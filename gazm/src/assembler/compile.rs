#![forbid(unused_imports)]
use std::path::Path;

use super::{binary::BinaryError, scopetracker::ScopeTracker, Assembler, AssemblerCpuTrait};

use crate::frontend::AstNodeKind;
use crate::{
    debug_mess,
    error::{GResult, GazmErrorKind, UserError},
    info_mess,
    semantic::{Ast, AstNodeId, AstNodeRef},
};

use grl_sources::ItemType;

pub struct Compiler<'a, C>
where
    C: AssemblerCpuTrait,
{
    tree: &'a Ast<C>,
    pub scopes: ScopeTracker,
    pub compiler: C,
}

pub fn compile<C>(asm: &mut Assembler<C>, tree: &Ast<C>) -> GResult<()>
where
    C: AssemblerCpuTrait,
{
    let root_id = asm.get_symbols().get_root_scope_id();
    let mut compiler = Compiler::new(tree, root_id)?;
    compiler.compile_root(asm)?;
    Ok(())
}

impl<'a, C> Compiler<'a, C>
where
    C: AssemblerCpuTrait,
{
    pub fn new(tree: &'a Ast<C>, current_scope_id: u64) -> GResult<Self> {
        Ok(Self {
            tree,
            scopes: ScopeTracker::new(current_scope_id),
            compiler: C::new(),
        })
    }

    pub fn compile_root(&mut self, asm: &mut Assembler<C>) -> GResult<()> {
        let scope_id = asm.get_symbols().get_root_scope_id();
        self.scopes.set_scope(scope_id);
        asm.set_pc_symbol(0).expect("Can't set pc symbol");

        self.compile_node_error(asm, self.tree.as_ref().root().id())
    }

    fn get_node_id_item(
        &self,
        asm: &Assembler<C>,
        id: AstNodeId,
    ) -> (AstNodeId, AstNodeKind<C::NodeKind>) {
        let node = self.tree.as_ref().get(id).unwrap();
        let this_i = &node.value().item;
        let i = asm.get_fixup_or_default(id, this_i, self.scopes.scope());
        (node.id(), i)
    }

    pub fn get_node(&self, id: AstNodeId) -> AstNodeRef<C> {
        let node = self.tree.as_ref().get(id).unwrap();
        node
    }

    pub fn binary_error(
        &self,
        asm: &mut Assembler<C>,
        id: AstNodeId,
        e: BinaryError,
    ) -> GazmErrorKind {
        let n = self.get_node(id);
        let info = &asm.get_source_info(&n.value().pos).unwrap();
        let msg = e.to_string();
        UserError::from_text(msg, info, true).into()
    }

    pub fn binary_error_map<T>(
        &self,
        asm: &mut Assembler<C>,
        id: AstNodeId,
        e: Result<T, BinaryError>,
    ) -> Result<T, GazmErrorKind> {
        if !asm.opts.error_mismatches {
            if let Err(BinaryError::DoesNotMatchReference(_r)) = &e {
            }
        }

        let ret = e.map_err(|e| self.binary_error(asm, id, e));

        ret
    }

    pub fn relative_error(
        &self,
        asm: &Assembler<C>,
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
        UserError::from_text(msg, info, true).into()
    }

    /// Adds a mapping of this source file fragment to a physicl and logical range of memory
    /// ( physical range, logical_range )
    pub fn add_mapping(
        &self,
        asm: &mut Assembler<C>,
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
    fn grab_mem(&self, asm: &mut Assembler<C>, id: AstNodeId) -> GResult<()> {
        let node = self.get_node(id);
        let args = asm.eval_n_args(node, 2, self.scopes.scope())?;
        let source = args[0];
        let size = args[1];

        let bytes = asm
            .get_binary()
            .get_bytes(source as usize, size as usize)
            .map(|n| n.to_vec()).map_err(|e| self.binary_error(asm, id, e))?;

        let ret = asm.get_binary_mut().write_bytes(&bytes);

        self.binary_error_map(asm, id, ret)?;
        Ok(())
    }

    /// Add a binary to write
    fn add_binary_to_write<P: AsRef<Path>>(
        &self,
        asm: &mut Assembler<C>,
        id: AstNodeId,
        path: P,
    ) -> GResult<()> {
        let current_scope_id = self.scopes.scope();
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
        asm: &mut Assembler<C>,
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

    pub fn write_word(&mut self, val: u16, asm: &mut Assembler<C>, id: AstNodeId) -> GResult<()> {
        let ret = asm.get_binary_mut().write_word(val);
        self.binary_error_map(asm, id, ret)?;
        Ok(())
    }

    pub fn write_byte(&mut self, val: u8, asm: &mut Assembler<C>, id: AstNodeId) -> GResult<()> {
        let ret = asm.get_binary_mut().write_byte(val);
        self.binary_error_map(asm, id, ret)?;
        Ok(())
    }

    pub fn write_byte_check_size(
        &mut self,
        val: i64,
        asm: &mut Assembler<C>,
        id: AstNodeId,
    ) -> GResult<()> {
        let ret = asm.get_binary_mut().write_byte_check_size(val);
        self.binary_error_map(asm, id, ret)?;
        Ok(())
    }

    pub fn write_word_check_size(
        &mut self,
        val: i64,
        asm: &mut Assembler<C>,
        id: AstNodeId,
    ) -> GResult<()> {
        let ret = asm.get_binary_mut().write_word_check_size(val);
        self.binary_error_map(asm, id, ret)?;
        Ok(())
    }

    fn _write_byte_word_size(
        &mut self,
        val: i64,
        asm: &mut Assembler<C>,
        id: AstNodeId,
    ) -> GResult<()> {
        let ret = asm.get_binary_mut().write_word_check_size(val);
        self.binary_error_map(asm, id, ret)?;
        Ok(())
    }

    fn incbin_resolved<P: AsRef<Path>>(
        &self,
        asm: &mut Assembler<C>,
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

    fn compile_children(&mut self, asm: &mut Assembler<C>, id: AstNodeId) -> GResult<()> {
        let node = self.get_node(id);
        let kids: Vec<_> = node.children().map(|n| n.id()).collect();
        for c in kids {
            self.compile_node_error(asm, c)?;
        }
        Ok(())
    }

    fn add_source_mapping(&self, asm: &mut Assembler<C>, id: AstNodeId, addr: usize) {
        let node = self.get_node(id);
        let _i = node.value().item.clone();
        // TODO Fix this fucker!
        let kind: ItemType = ItemType::OpCode;

        asm.add_source_mapping(&node.value().pos, addr, kind);
    }

    fn compile_node_error(&mut self, asm: &mut Assembler<C>, id: AstNodeId) -> GResult<()> {
        use AstNodeKind::*;

        let (node_id, i) = self.get_node_id_item(asm, id);

        let mut pc = asm.get_binary().get_write_address();

        let mut do_source_mapping = true;
        let current_scope_id = self.scopes.scope();

        asm.set_pc_symbol(pc).expect("Can't set PC symbol value");

        match i {
            ScopeId(scope_id) => self.scopes.set_scope(scope_id),

            GrabMem => self.grab_mem(asm, id)?,

            WriteBin(file_name) => self.add_binary_to_write(asm, id, &file_name)?,

            IncBinRef(file_name) => {
                self.inc_bin_ref(asm, &file_name, id, current_scope_id)?;
            }

            IncBinResolved { file, r } => {
                self.incbin_resolved(asm, id, &file, &r)?;
            }

            Skip(skip) => {
                asm.get_binary_mut().skip(skip);
            }

            SetPc(new_pc) => {
                asm.get_binary_mut().set_write_address(new_pc, 0);

                pc = new_pc;
                debug_mess!("Set PC to {:02X}", pc);
            }

            SetPutOffset(offset) => {
                debug_mess!("Set put offset to {}", offset);
                asm.get_binary_mut().set_write_offset(offset);
            }

            MacroCallProcessed {
                scope_id, macro_id, ..
            } => {
                let node = self.get_node(node_id);
                do_source_mapping = false;
                let ret = asm.eval_macro_args(scope_id, node);

                if !ret {
                    let pos = &node.value().pos;
                    let si = asm.get_source_info(pos).unwrap();
                    return Err(UserError::from_text(
                        "Couldn't evaluate all macro args",
                        &si,
                        true,
                    )
                    .into());
                }

                self.scopes.push(scope_id);

                {
                    let m_node = self.get_node(macro_id);
                    let kids: Vec<_> = m_node.children().map(|n| n.id()).collect();

                    for c_node in kids {
                        self.compile_node_error(asm, c_node)?;
                    }
                }

                self.scopes.pop();
            }

            TokenizedFile(..) => {
                self.compile_children(asm, id)?;
            }

            Fdb(..) => {
                let node = self.get_node(node_id);

                for n in node.children() {
                    let x = asm.eval_node(n, current_scope_id)?;
                    let e = asm.get_binary_mut().write_word_check_size(x);
                    self.binary_error_map(asm, id, e)?;
                }

                let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, id, ItemType::Command);
            }

            Fcb(..) => {
                let node = self.get_node(node_id);
                for n in node.children() {
                    let x = asm.eval_node(n, current_scope_id)?;
                    let e = asm.get_binary_mut().write_byte_check_size(x);
                    self.binary_error_map(asm, id, e)?;
                }
                let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, id, ItemType::Command);
            }

            Fcc(text) => {
                for c in text.as_bytes() {
                    let e = asm.get_binary_mut().write_byte(*c);
                    self.binary_error_map(asm, id, e)?;
                }
                let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, id, ItemType::Command);
            }

            Zmb => {
                let node = self.get_node(node_id);
                let (bytes, _) = asm.eval_first_arg(node, current_scope_id)?;
                for _ in 0..bytes {
                    let e = asm.get_binary_mut().write_byte(0);
                    self.binary_error_map(asm, id, e)?;
                }
                let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, id, ItemType::Command);
            }

            Zmd => {
                let node = self.get_node(node_id);
                let (words, _) = asm.eval_first_arg(node, current_scope_id)?;
                for _ in 0..words {
                    let e = asm.get_binary_mut().write_word(0);
                    self.binary_error_map(asm, id, e)?;
                }

                let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, id, ItemType::Command);
            }

            Fill => {
                let node = self.get_node(node_id);
                let (size, byte) = asm.eval_two_args(node, current_scope_id)?;

                for _ in 0..size {
                    let e = asm.get_binary_mut().write_ubyte_check_size(byte);
                    self.binary_error_map(asm, id, e)?;
                }

                let (phys_range, range) = asm.get_binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, id, ItemType::Command);
            }

            Exec => {
                let node = self.get_node(node_id);
                let (exec_addr, _) = asm.eval_first_arg(node, current_scope_id)?;
                asm.asm_out.exec_addr = Some(exec_addr as usize);
            }

            IncBin(..) | Org | AssignmentFromPc(..) | Assignment(..) | Comment(..) | Rmb
            | StructDef(..) | MacroDef(..) | MacroCall(..) | Import => (),

            CpuSpecific(node_kind) => {
                C::compile_node(self, asm, id, node_kind)?;
            }

            _ => {
                panic!("Can't compile {i:?}");
            }
        }

        if do_source_mapping {
            self.add_source_mapping(asm, node_id, pc);
        }

        Ok(())
    }
}
