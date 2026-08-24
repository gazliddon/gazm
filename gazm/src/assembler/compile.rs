#![forbid(unused_imports)]
use std::path::Path;

use super::{binary::BinaryError, plan::PlanEntry, Assembler};

use crate::frontend::{AstNodeKind, AstNodeKindDiscriminants};
use crate::{
    debug_mess,
    error::{Diagnostic, GResult, GazmErrorKind},
    info_mess,
    semantic::{Ast, AstNodeId, AstNodeRef},
};

use grl_sources::ItemType;

pub struct Compiler<'a> {
    tree: &'a Ast,
}

/// Replay the walk plan produced by the sizer: emit the bytes for each
/// statement in order. The plan already carries the resolved scope, the
/// final node kind (fixups applied at plan-build time) and the computed PC,
/// so this pass holds no layout state of its own — it just checks the binary
/// write address matches the planned PC, applies any loop-index bindings, and
/// emits.
pub fn compile(asm: &mut Assembler, tree: &Ast, plan: &[PlanEntry]) -> GResult<()> {
    let mut compiler = Compiler { tree };
    for entry in plan {
        compiler.compile_entry(asm, entry)?;
    }
    Ok(())
}

impl<'a> Compiler<'a> {
    fn compile_entry(&mut self, asm: &mut Assembler, entry: &PlanEntry) -> GResult<()> {
        use AstNodeKind::*;

        let node_id = entry.node_id;
        let current_scope_id = entry.scope_id;

        // Internal invariant: the layout the sizer computed must agree with
        // the binary's write address. Any drift here means sizing and
        // emission disagreed, which would otherwise silently corrupt output.
        assert_eq!(
            asm.get_binary().get_write_address(),
            entry.pc,
            "layout drift: plan expects PC ${:04X} but the binary write address is ${:04X}",
            entry.pc,
            asm.get_binary().get_write_address()
        );

        asm.set_pc_symbol_internal(entry.pc)?;

        // Loop index bindings from enclosing `repeat`s (usually empty).
        for (symbol_id, value) in &entry.bindings {
            asm.get_symbols_mut()
                .set_value_for_id(*symbol_id, *value)
                .map_err(|e| -> crate::error::GazmErrorKind {
                    let node = self.get_node(node_id);
                    asm.make_user_error(format!("repeat: {e}"), node, true)
                        .into()
                })?;
        }

        let mut pc = entry.pc;
        let mut do_source_mapping = true;

        match &entry.kind {
            MacroCallProcessed { .. } => {
                // Re-evaluate the macro arguments now that the whole layout
                // is known; parameters whose arguments reference forward
                // labels could not be evaluated at size time. The body
                // statements are separate plan entries that follow.
                do_source_mapping = false;
                let node = self.get_node(node_id);
                let ret = asm.eval_macro_args(current_scope_id, node);

                if !ret {
                    let pos = &node.value().pos;
                    let si = asm.get_source_info(pos).unwrap();
                    return Err(Diagnostic::from_text(
                        "Couldn't evaluate all macro args",
                        &si,
                        true,
                    )
                    .into());
                }
            }

            ScopeId(..) => (),

            GrabMem => self.grab_mem(asm, node_id, current_scope_id)?,

            WriteBin(file_name) => {
                self.add_binary_to_write(asm, node_id, file_name, current_scope_id)?;
            }

            IncBinRef(file_name) => {
                self.inc_bin_ref(asm, file_name, node_id, current_scope_id)?;
            }

            IncBinResolved { file, r } => {
                self.incbin_resolved(asm, node_id, file, r)?;
            }

            Skip(skip) => {
                asm.get_binary_mut().skip(*skip);
            }

            SetPc(new_pc) => {
                asm.get_binary_mut().set_write_address(*new_pc, 0);

                pc = *new_pc;
                debug_mess!("Set PC to {:02X}", pc);
            }

            SetPutOffset(offset) => {
                debug_mess!("Set put offset to {}", offset);
                asm.get_binary_mut().set_write_offset(*offset);
            }

            TokenizedFile(..) | Block => (),

            Fdb(..) => {
                let node = self.get_node(node_id);

                for n in node.children() {
                    let x = asm.eval_node(n, current_scope_id)?;
                    let e = asm.get_binary_mut().write_word_check_size(x);
                    self.binary_error_map(asm, node_id, e)?;
                }

                let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, node_id, ItemType::Command);
            }

            Fcb(..) => {
                let node = self.get_node(node_id);
                for n in node.children() {
                    let x = asm.eval_node(n, current_scope_id)?;
                    let e = asm.get_binary_mut().write_byte_check_size(x);
                    self.binary_error_map(asm, node_id, e)?;
                }
                let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, node_id, ItemType::Command);
            }

            Fcc(text) => {
                for c in text.as_bytes() {
                    let e = asm.get_binary_mut().write_byte(*c);
                    self.binary_error_map(asm, node_id, e)?;
                }
                let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, node_id, ItemType::Command);
            }

            Zmb => {
                let node = self.get_node(node_id);
                let (bytes, _) = asm.eval_first_arg(node, current_scope_id)?;
                for _ in 0..bytes {
                    let e = asm.get_binary_mut().write_byte(0);
                    self.binary_error_map(asm, node_id, e)?;
                }
                let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, node_id, ItemType::Command);
            }

            Zmd => {
                let node = self.get_node(node_id);
                let (words, _) = asm.eval_first_arg(node, current_scope_id)?;
                for _ in 0..words {
                    let e = asm.get_binary_mut().write_word(0);
                    self.binary_error_map(asm, node_id, e)?;
                }

                let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, node_id, ItemType::Command);
            }

            Fill => {
                let node = self.get_node(node_id);
                let (size, byte) = asm.eval_two_args(node, current_scope_id)?;

                for _ in 0..size {
                    let e = asm.get_binary_mut().write_ubyte_check_size(byte);
                    self.binary_error_map(asm, node_id, e)?;
                }

                let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, node_id, ItemType::Command);
            }

            Exec => {
                let node = self.get_node(node_id);
                let (exec_addr, _) = asm.eval_first_arg(node, current_scope_id)?;
                asm.asm_out.exec_addr = Some(exec_addr as usize);
            }

            TargetSpecific(node_kind) => {
                let node = self.get_node(node_id);

                asm.compile_node(node, node_kind.clone(), current_scope_id)?;
            }

            AssignmentFromPc(..) | Assignment(..) | Comment(..) | StructDef(..) | MacroDef(..)
            | MacroCall(..) | Import => (),

            _ => {
                panic!("Can't compile {:?}", entry.kind);
            }
        }

        if do_source_mapping {
            self.add_source_mapping(asm, node_id, pc);
        }
        Ok(())
    }

    pub fn get_node(&self, id: AstNodeId) -> AstNodeRef<'_> {
        let node = self.tree.as_ref().get(id).unwrap();
        node
    }

    pub fn binary_error(
        &self,
        asm: &mut Assembler,
        id: AstNodeId,
        e: BinaryError,
    ) -> GazmErrorKind {
        let n = self.get_node(id);
        let info = &asm.get_source_info(&n.value().pos).unwrap();
        let msg = e.to_string();
        Diagnostic::from_text(msg, info, true).into()
    }

    pub fn binary_error_map<T>(
        &self,
        asm: &mut Assembler,
        id: AstNodeId,
        e: Result<T, BinaryError>,
    ) -> Result<T, GazmErrorKind> {
        if !asm.opts.error_mismatches {
            if let Err(BinaryError::DoesNotMatchReference(_r)) = &e {}
        }

        e.map_err(|e| self.binary_error(asm, id, e))
    }

    pub fn relative_error(
        &self,
        asm: &Assembler,
        id: AstNodeId,
        val: i64,
        bits: usize,
    ) -> GazmErrorKind {
        let n = self.get_node(id);
        let p = 1 << (bits - 1);

        let message = if val < 0 {
            format!("Branch out of range by {} bytes ({val})", (p + val).abs())
        } else {
            format!("Branch out of range by {} bytes ({val})", val - (p - 1))
        };

        let info = &asm.get_source_info(&n.value().pos).unwrap();
        let msg = message;
        Diagnostic::from_text(msg, info, true).into()
    }

    /// Adds a mapping of this source file fragment to a physicl and logical range of memory
    /// ( physical range, logical_range )
    pub fn add_mapping(
        &self,
        asm: &mut Assembler,
        phys_range: std::ops::Range<usize>,
        range: std::ops::Range<usize>,
        id: AstNodeId,
        i: ItemType,
    ) {
        let pos = self.get_node(id).value().pos;
        asm.asm_out
            .source_map
            .add_mapping(phys_range, range, &pos, i);
    }

    /// Grab memory and copy it the PC
    fn grab_mem(&self, asm: &mut Assembler, id: AstNodeId, current_scope_id: u64) -> GResult<()> {
        let node = self.get_node(id);
        let args = asm.eval_n_args(node, 2, current_scope_id)?;
        let source = args[0];
        let size = args[1];

        let bytes = asm
            .get_binary()
            .get_bytes(source as usize, size as usize)
            .map(|n| n.to_vec())
            .map_err(|e| self.binary_error(asm, id, e))?;

        let ret = asm.get_binary_mut().write_bytes(&bytes);

        self.binary_error_map(asm, id, ret)?;
        Ok(())
    }

    /// Add a binary to write
    fn add_binary_to_write<P: AsRef<Path>>(
        &self,
        asm: &mut Assembler,
        id: AstNodeId,
        path: P,
        current_scope_id: u64,
    ) -> GResult<()> {
        let node = self.get_node(id);
        let (physical_address, count) = asm.eval_two_args(node, current_scope_id)?;

        asm.add_bin_to_write(
            &path,
            physical_address as usize..(physical_address + count) as usize,
        )?;

        Ok(())
    }

    fn inc_bin_ref<P: AsRef<Path>>(
        &self,
        asm: &mut Assembler,
        file_name: P,
        node_id: AstNodeId,
        current_scope_id: u64,
    ) -> GResult<()> {
        use crate::assembler::binary::BinRef;
        let file = file_name.as_ref().to_path_buf();
        let (.., data) = asm.read_binary_file(&file_name)?;

        let node = self.get_node(node_id);

        let mut result = asm.eval_all_args(node, current_scope_id)?;

        if result.len() == 1 {
            result.push(data.len() as i64)
        }

        let dest = result[0] as usize;
        let size = result[1] as usize;

        assert!(size <= data.len());

        let bin_ref = BinRef {
            file: file.clone(),
            start: 0,
            size,
            dest,
        };

        asm.get_binary_mut().add_bin_reference(&bin_ref, &data);

        info_mess!(
            "Adding binary reference {} for ${:04x} - ${:04x}",
            file.to_string_lossy(),
            dest,
            (dest + data.len()) - 1
        );

        Ok(())
    }

    // pub fn write_word(&mut self, val: u16, asm: &mut Assembler, node: AstNodeRef) -> GResult<()> {
    //     let ret = asm.get_binary_mut().write_word(val);
    //     self.binary_error_map(asm, node.id(), ret)?;
    //     Ok(())
    // }

    // pub fn write_byte(&mut self, val: u8, asm: &mut Assembler, node: AstNodeRef) -> GResult<()> {
    //     let ret = asm.get_binary_mut().write_byte(val);
    //     self.binary_error_map(asm, node.id(), ret)?;
    //     Ok(())
    // }

    // pub fn write_byte_check_size(
    //     &mut self,
    //     val: i64,
    //     asm: &mut Assembler,
    //     id: AstNodeId,
    // ) -> GResult<()> {
    //     let ret = asm.get_binary_mut().write_byte_check_size(val);
    //     self.binary_error_map(asm, id, ret)?;
    //     Ok(())
    // }

    // pub fn write_word_check_size(
    //     &mut self,
    //     val: i64,
    //     asm: &mut Assembler,
    //     id: AstNodeId,
    // ) -> GResult<()> {
    //     let ret = asm.get_binary_mut().write_word_check_size(val);
    //     self.binary_error_map(asm, id, ret)?;
    //     Ok(())
    // }

    fn _write_byte_word_size(
        &mut self,
        val: i64,
        asm: &mut Assembler,
        id: AstNodeId,
    ) -> GResult<()> {
        let ret = asm.get_binary_mut().write_word_check_size(val);
        self.binary_error_map(asm, id, ret)?;
        Ok(())
    }

    fn incbin_resolved<P: AsRef<Path>>(
        &self,
        asm: &mut Assembler,
        id: AstNodeId,
        file: P,
        r: &std::ops::Range<usize>,
    ) -> GResult<()> {
        debug_mess!(
            "Including Binary {} :  offset: {:04X} len: {:04X}",
            file.as_ref().to_string_lossy(),
            r.start,
            r.len()
        );

        let (.., bin) = asm.read_binary_file_chunk(file, r.clone())?;

        for val in bin {
            let ret = asm.get_binary_mut().write_byte(val);

            self.binary_error_map(asm, id, ret)?;
        }
        Ok(())
    }

    fn add_source_mapping(&self, asm: &mut Assembler, id: AstNodeId, addr: usize) {
        let node = self.get_node(id);
        // TODO Fix this fucker!
        let kind: ItemType = ItemType::OpCode;

        asm.add_source_mapping(&node.value().pos, addr, kind);
    }
}

/// True if `compile_entry` has an arm for this node kind (including its
/// explicit no-op arm); false if it hits the `panic!("Can't compile")`
/// catch-all.
///
/// Mirrors the `match` in `Compiler::compile_entry` — keep the two in sync.
/// The coverage test in `mod.rs` asserts this set equals the set of kinds the
/// sizer can put in the plan, so a kind added to one pass but not the other
/// fails loudly instead of emitting wrong bytes.
pub(crate) fn compiler_handles(kind: &AstNodeKindDiscriminants) -> bool {
    use AstNodeKindDiscriminants::*;
    matches!(
        kind,
        MacroCallProcessed
            | ScopeId
            | GrabMem
            | WriteBin
            | IncBinRef
            | IncBinResolved
            | Skip
            | SetPc
            | SetPutOffset
            | TokenizedFile
            | Block
            | Fdb
            | Fcb
            | Fcc
            | Zmb
            | Zmd
            | Fill
            | Exec
            | TargetSpecific
            | AssignmentFromPc
            | Assignment
            | Comment
            | StructDef
            | MacroDef
            | MacroCall
            | Import
    )
}
