#![forbid(unused_imports)]
use std::path::Path;

use crate::{
    assembler::Assembler,
    ast::{Ast, AstNodeId, AstNodeRef},
    binary::BinaryError,
    debug_mess,
    error::{GResult, GazmErrorKind, UserError},
    gazmsymbols::SymbolScopeId,
    info_mess,
    item::{self, Item},
    item6809::{
        self, AddrModeParseType, IndexParseType,
        MC6809::{self, OpCode, SetDp},
    },
    regutils::*,
    scopetracker::ScopeTracker,
    status_mess,
};

use emu6809::isa;
use grl_sources::ItemType;

pub struct Compiler<'a> {
    tree: &'a Ast,
    scopes: ScopeTracker,
    pc_symbol_id: SymbolScopeId,
}

impl<'a> Compiler<'a> {
    pub fn new(tree: &'a Ast, current_scope_id: u64, pc_symbol_id: SymbolScopeId) -> GResult<Self> {
        let ret = Self {
            tree,
            scopes: ScopeTracker::new(current_scope_id),
            pc_symbol_id,
        };
        Ok(ret)
    }

    pub fn compile_root(&mut self, asm: &mut Assembler) -> GResult<()> {
        let scope_id = asm.get_symbols().get_root_scope_id();
        self.scopes.set_scope(scope_id);
        self.compile_node_error(asm, self.tree.as_ref().root().id())
    }
}

impl<'a> Compiler<'a> {
    fn get_node_id_item(&self, asm: &Assembler, id: AstNodeId) -> (AstNodeId, Item) {
        let node = self.tree.as_ref().get(id).unwrap();
        let this_i = &node.value().item;
        let i = asm.get_fixup_or_default(id, this_i, self.scopes.scope());
        (node.id(), i)
    }

    // fn get_node_item_ref(&self, asm: &Assembler, id: AstNodeId) -> (AstNodeRef, Item) {
    //     let node = self.tree.as_ref().get(id).unwrap();
    //     let this_i = &node.value().item;
    //     let i = asm.get_fixup_or_default(id, this_i, self.scopes.scope());
    //     (node, i)
    // }

    fn get_node(&self, id: AstNodeId) -> AstNodeRef {
        let node = self.tree.as_ref().get(id).unwrap();
        node
    }

    fn binary_error(
        &self,
        asm: &mut Assembler,
        id: AstNodeId,
        e: crate::binary::BinaryError,
    ) -> GazmErrorKind {
        let n = self.get_node(id);
        let info = &asm.get_source_info(&n.value().pos).unwrap();
        let msg = e.to_string();
        UserError::from_text(msg, info, true).into()
    }

    fn binary_error_map<T>(
        &self,
        asm: &mut Assembler,
        id: AstNodeId,
        e: Result<T, BinaryError>,
    ) -> Result<T, GazmErrorKind> {
        e.map_err(|e| self.binary_error(asm, id, e))
    }

    fn relative_error(
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
        UserError::from_text(msg, info, true).into()
    }

    fn compile_indexed(
        &mut self,
        asm: &mut Assembler,
        id: AstNodeId,
        imode: IndexParseType,
        indirect: bool,
    ) -> GResult<()> {
        use item6809::IndexParseType::*;
        let idx_byte = imode.get_index_byte(indirect);

        self.write_byte(idx_byte, asm, id)?;

        let node = self.get_node(id);

        match imode {
            PCOffset | ConstantOffset(..) => {
                panic!("Should not happen")
            }

            ExtendedIndirect => {
                let (val, _) = asm.eval_first_arg(node, self.scopes.scope())?;

                let res = asm.binary_mut().write_uword_check_size(val);
                self.binary_error_map(asm, id, res)?;
            }

            ConstantWordOffset(_, val) | PcOffsetWord(val) => {
                let res = asm.binary_mut().write_iword_check_size(val as i64);
                self.binary_error_map(asm, id, res)?;
            }

            ConstantByteOffset(_, val) | PcOffsetByte(val) => {
                let res = asm.binary_mut().write_ibyte_check_size(val as i64);
                self.binary_error_map(asm, id, res)?;
            }
            _ => (),
        }

        Ok(())
    }

    /// Adds a mapping of this source file fragment to a physicl and logical range of memory
    /// ( physical range, logical_range )
    fn add_mapping(
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
    fn grab_mem(&self, asm: &mut Assembler, id: AstNodeId) -> GResult<()> {
        let node = self.get_node(id);
        let args = asm.eval_n_args(node, 2, self.scopes.scope())?;
        let source = args[0];
        let size = args[1];

        let bytes_ret = asm
            .binary()
            .get_bytes(source as usize, size as usize)
            .map(|n| n.to_vec());

        let bytes = bytes_ret.map_err(|e| self.binary_error(asm, id, e))?;

        asm.binary_mut()
            .write_bytes(&bytes)
            .map_err(|e| self.binary_error(asm, id, e))?;

        Ok(())
    }

    /// Add a binary to write
    fn add_binary_to_write<P: AsRef<Path>>(
        &self,
        asm: &mut Assembler,
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

    fn inc_bin_ref<P: AsRef<Path>>(&self, asm: &mut Assembler, file_name: P) -> GResult<()> {
        use crate::binary::BinRef;

        let file = file_name.as_ref().to_path_buf();

        let (.., data) = asm.read_binary(&file_name)?;

        let dest = asm.binary().get_write_location().physical;

        let bin_ref = BinRef {
            file: file.clone(),
            start: 0,
            size: data.len(),
            dest,
        };

        asm.binary_mut().add_bin_reference(&bin_ref, &data);

        info_mess!(
            "Adding binary reference {} for {:05X} - {:05X}",
            file.to_string_lossy(),
            dest,
            dest + data.len()
        );

        Ok(())
    }

    fn write_word(&mut self, val: u16, asm: &mut Assembler, id: AstNodeId) -> GResult<()> {
        let ret = asm.binary_mut().write_word(val);
        self.binary_error_map(asm, id, ret)?;
        Ok(())
    }

    fn write_byte(&mut self, val: u8, asm: &mut Assembler, id: AstNodeId) -> GResult<()> {
        let ret = asm.binary_mut().write_byte(val);
        self.binary_error_map(asm, id, ret)?;
        Ok(())
    }

    fn write_byte_check_size(
        &mut self,
        val: i64,
        asm: &mut Assembler,
        id: AstNodeId,
    ) -> GResult<()> {
        let ret = asm.binary_mut().write_byte_check_size(val);
        self.binary_error_map(asm, id, ret)?;
        Ok(())
    }
    fn write_word_check_size(
        &mut self,
        val: i64,
        asm: &mut Assembler,
        id: AstNodeId,
    ) -> GResult<()> {
        let ret = asm.binary_mut().write_word_check_size(val);
        self.binary_error_map(asm, id, ret)?;
        Ok(())
    }

    fn write_byte_word_size(
        &mut self,
        val: i64,
        asm: &mut Assembler,
        id: AstNodeId,
    ) -> GResult<()> {
        let ret = asm.binary_mut().write_word_check_size(val);
        self.binary_error_map(asm, id, ret)?;
        Ok(())
    }

    /// Compile an opcode
    fn compile_opcode(
        &mut self,
        asm: &mut Assembler,
        id: AstNodeId,
        ins: &isa::Instruction,
        amode: AddrModeParseType,
    ) -> GResult<()> {
        use isa::AddrModeEnum::*;

        let pc = asm.binary().get_write_address();
        let ins_amode = ins.addr_mode;
        let current_scope_id = self.scopes.scope();

        if ins.opcode > 0xff {
            self.write_word(ins.opcode as u16, asm, id)
        } else {
            self.write_byte(ins.opcode as u8, asm, id)
        }?;

        let node = self.get_node(id);

        match ins_amode {
            Indexed => {
                if let AddrModeParseType::Indexed(imode, indirect) = amode {
                    self.compile_indexed(asm, id, imode, indirect)?;
                }
            }

            Immediate8 => {
                let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
                self.write_byte_check_size(arg, asm, id)?
            }

            Direct => {
                let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
                self.write_byte_check_size(arg & 0xff, asm, id)?
            }

            Extended | Immediate16 => {
                let (arg, _) = asm.eval_first_arg(node, current_scope_id)?;
                self.write_word_check_size(arg, asm, id)?;
            }

            Relative => {
                use crate::binary::BinaryError::*;
                let (arg, arg_id) = asm.eval_first_arg(node, current_scope_id)?;
                let arg_n = self.get_node(arg_id);
                let val = arg - (pc as i64 + ins.size as i64);
                // offset is from PC after Instruction and operand has been fetched
                let res = asm
                    .asm_out
                    .binary
                    .write_ibyte_check_size(val)
                    .map_err(|x| match x {
                        DoesNotFit { .. } => self.relative_error(asm, id, val, 8),
                        DoesNotMatchReference { .. } => self.binary_error(asm, id, x),
                        _ => asm.user_error(format!("{x:?}"), arg_n, false).into(),
                    });

                match &res {
                    Ok(_) => (),
                    Err(_) => {
                        if asm.opts.ignore_relative_offset_errors {
                            // messages::warning("Skipping writing relative offset");
                            let res = asm.binary_mut().write_ibyte_check_size(0);
                            self.binary_error_map(asm, id, res)?;
                        } else {
                            res?;
                        }
                    }
                }
            }

            Relative16 => {
                use crate::binary::BinaryError::*;

                let (arg, arg_id) = asm.eval_first_arg(node, current_scope_id)?;

                let arg_n = self.get_node(arg_id);

                let val = (arg - (pc as i64 + ins.size as i64)) & 0xffff;
                // offset is from PC after Instruction and operand has been fetched
                let res = asm.binary_mut().write_word_check_size(val);

                res.map_err(|x| match x {
                    DoesNotFit { .. } => self.relative_error(asm, id, val, 16),
                    DoesNotMatchReference { .. } => self.binary_error(asm, id, x),
                    _ => asm.user_error(format!("{x:?}"), arg_n, true).into(),
                })?;
            }

            Inherent => {}

            RegisterPair => {
                if let AddrModeParseType::RegisterPair(a, b) = amode {
                    let val = reg_pair_to_flags(a, b);
                    self.write_byte(val, asm, id)?;
                } else {
                    panic!("Whut!")
                }
            }

            RegisterSet => {
                let rset = &node.first_child().unwrap().value().item;
                if let Item::Cpu(MC6809::RegisterSet(regs)) = rset {
                    let flags = registers_to_flags(regs);
                    self.write_byte(flags, asm, id)?;
                } else {
                    panic!("Whut!")
                }
            }
        };

        // Add memory to source code mapping for this opcode
        let (phys_range, range) = asm.binary().range_to_write_address(pc);
        self.add_mapping(asm, phys_range, range, id, ItemType::OpCode);
        Ok(())
    }

    fn incbin_resolved<P: AsRef<Path>>(
        &self,
        asm: &mut Assembler,
        id: AstNodeId,
        file: P,
        r: &std::ops::Range<usize>,
    ) -> GResult<()> {
        status_mess!(
            "Including Binary {} :  offset: {:04X} len: {:04X}",
            file.as_ref().to_string_lossy(),
            r.start,
            r.len()
        );

        let (.., bin) = asm.read_binary_chunk(file, r.clone())?;

        for val in bin {
            asm.binary_mut()
                .write_byte(val)
                .map_err(|e| self.binary_error(asm, id, e))?;
        }
        Ok(())
    }

    fn compile_children(&mut self, asm: &mut Assembler, id: AstNodeId) -> GResult<()> {
        let node = self.get_node(id);
        let kids: Vec<_> = node.children().map(|n| n.id()).collect();
        for c in kids {
            self.compile_node_error(asm, c)?;
        }
        Ok(())
    }

    fn compile_node_error(&mut self, asm: &mut Assembler, id: AstNodeId) -> GResult<()> {
        use item::Item::*;

        let (node_id, i) = self.get_node_id_item(asm, id);

        let mut pc = asm.binary().get_write_address();
        let mut do_source_mapping = asm.opts.lst_file.is_some();
        let current_scope_id = self.scopes.scope();

        asm.set_symbol_value(self.pc_symbol_id, pc)
            .expect("Can't set PC symbol value");

        match i {
            ScopeId(scope_id) => self.scopes.set_scope(scope_id),

            GrabMem => self.grab_mem(asm, id)?,

            WriteBin(file_name) => self.add_binary_to_write(asm, id, &file_name)?,

            IncBinRef(file_name) => {
                self.inc_bin_ref(asm, &file_name)?;
            }

            IncBinResolved { file, r } => {
                self.incbin_resolved(asm, id, &file, &r)?;
            }

            Skip(skip) => {
                asm.binary_mut().skip(skip);
            }

            SetPc(new_pc) => {
                asm.binary_mut().set_write_address(new_pc, 0);

                pc = new_pc;
                debug_mess!("Set PC to {:02X}", pc);
            }

            SetPutOffset(offset) => {
                debug_mess!("Set put offset to {}", offset);
                asm.binary_mut().set_write_offset(offset);
            }

            Cpu(OpCode(_, ins, amode)) => {
                self.compile_opcode(asm, id, &ins, amode)?;
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
                    let e = asm.binary_mut().write_word_check_size(x);
                    self.binary_error_map(asm, id, e)?;
                }

                let (phys_range, range) = asm.binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, id, ItemType::Command);
            }

            Fcb(..) => {
                let node = self.get_node(node_id);
                for n in node.children() {
                    let x = asm.eval_node(n, current_scope_id)?;
                    let e = asm.binary_mut().write_byte_check_size(x);
                    self.binary_error_map(asm, id, e)?;
                }
                let (phys_range, range) = asm.binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, id, ItemType::Command);
            }

            Fcc(text) => {
                for c in text.as_bytes() {
                    let e = asm.binary_mut().write_byte(*c);
                    self.binary_error_map(asm, id, e)?;
                }
                let (phys_range, range) = asm.binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, id, ItemType::Command);
            }

            Zmb => {
                let node = self.get_node(node_id);
                let (bytes, _) = asm.eval_first_arg(node, current_scope_id)?;
                for _ in 0..bytes {
                    let e = asm.binary_mut().write_byte(0);
                    self.binary_error_map(asm, id, e)?;
                }
                let (phys_range, range) = asm.binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, id, ItemType::Command);
            }

            Zmd => {
                let node = self.get_node(node_id);
                let (words, _) = asm.eval_first_arg(node, current_scope_id)?;
                for _ in 0..words {
                    let e = asm.binary_mut().write_word(0);
                    self.binary_error_map(asm, id, e)?;
                }

                let (phys_range, range) = asm.binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, id, ItemType::Command);
            }

            Fill => {
                let node = self.get_node(node_id);
                let (size, byte) = asm.eval_two_args(node, current_scope_id)?;

                for _ in 0..size {
                    let e = asm.binary_mut().write_ubyte_check_size(byte);
                    self.binary_error_map(asm, id, e)?;
                }

                let (phys_range, range) = asm.binary().range_to_write_address(pc);
                self.add_mapping(asm, phys_range, range, id, ItemType::Command);
            }

            Exec => {
                let node = self.get_node(node_id);
                let (exec_addr, _) = asm.eval_first_arg(node, current_scope_id)?;
                asm.asm_out.exec_addr = Some(exec_addr as usize);
            }

            IncBin(..) | Org | AssignmentFromPc(..) | Assignment(..) | Comment(..) | Rmb
            | StructDef(..) | MacroDef(..) | MacroCall(..) | Cpu(SetDp) | Import => (),
            _ => {
                panic!("Can't compile {i:?}");
            }
        }

        if do_source_mapping {
            let node = self.get_node(node_id);
            asm.add_source_mapping(&node.value().pos, pc);
        }

        Ok(())
    }
}
