#![forbid(unused_imports)]
use super::{
    Assembler,
    scopetracker::ScopeTracker,
    traits::AssemblerCpuTrait,
};

/// Take the AST and work out the sizes of everything
/// Resolve labels where we can
use crate::{
    semantic::{Ast, AstNodeId, AstNodeRef},
    error::GResult,
    frontend::{Item, LabelDefinition, }, 
};

use crate::cpu6809::assembler::Compiler6809;

use std::{path::Path, marker::PhantomData};

/// Ast tree sizer
/// gets the size of everything
/// assigns values to labels that
/// are defined by value of PC
pub struct Sizer<'a, C = Compiler6809> 
where 
    C : AssemblerCpuTrait,
{
    pub tree: &'a Ast<C>,
    pub scopes: ScopeTracker,
    pub pc: usize,

    phantom: std::marker::PhantomData<&'a C>,
}

pub fn size<C: AssemblerCpuTrait>(asm: &mut Assembler<C>, ast_tree: &Ast<C>) -> GResult<()> {
    let _ = Sizer::<C>::try_new(ast_tree, asm)?;
    Ok(())
}

impl<'a, C> Sizer<'a,C> 
where 
    C : AssemblerCpuTrait,
{
    pub fn try_new(tree: &'a Ast<C>, asm: &mut Assembler<C>) -> GResult<Sizer<'a, C>> {
        let pc = 0;

        asm.set_pc_symbol(pc).expect("Can't set PC symbol");

        let root_id = asm.get_symbols().get_root_scope_id();

        let mut ret = Self {
            tree,
            scopes: ScopeTracker::new(root_id),
            pc,
            phantom: PhantomData
        };

        let id = ret.tree.as_ref().root().id();
        ret.size_node(asm, id)?;

        Ok(ret)
    }

    pub fn advance_pc(&mut self, val: usize) {
        assert!(self.pc < 65536);
        self.pc += val;
    }

    pub fn get_pc(&self) -> usize {
        self.pc
    }

    pub fn set_pc(&mut self, val: usize) {
        self.pc = val;
        assert!(self.pc < 65536);
    }

    fn size_node(&mut self, asm: &mut Assembler<C>, id: AstNodeId) -> GResult<()> {
        use Item::*;

        let node = self.get_node(id);
        let i = &node.value().item.clone();
        let current_scope_id = self.scopes.scope();

        asm.set_pc_symbol(self.get_pc())
            .expect("Can't set PC symbol value");

        match &i {
            MacroCallProcessed {
                scope_id, macro_id, ..
            } => {
                asm.eval_macro_args_node(*scope_id, id, self.tree);

                self.scopes.push(*scope_id);

                let m_node = self.get_node(*macro_id);
                let kids: Vec<_> = m_node.children().map(|n| n.id()).collect();
                for c in kids {
                    self.size_node(asm, c)?;
                }

                self.scopes.pop();
            }

            ScopeId(scope_id) => self.scopes.set_scope(*scope_id),

            GrabMem => {
                let args = asm.eval_n_args(node, 2, current_scope_id)?;
                let size = args[1];
                self.advance_pc(size as usize);
            }

            Org => {
                let pc = asm.eval_first_arg(node, current_scope_id)?.0 as usize;
                asm.add_fixup(id, Item::SetPc(pc), current_scope_id);
                self.set_pc(pc);
            }

            SetPc(val) => {
                self.set_pc(*val);
            }

            Put => {
                let (value, _) = asm.eval_first_arg(node, current_scope_id)?;
                let offset = (value - self.get_pc() as i64) as isize;
                asm.add_fixup(id, Item::SetPutOffset(offset), current_scope_id);
            }

            Rmb => {
                let (bytes, _) = asm.eval_first_arg(node, current_scope_id)?;

                if bytes < 0 {
                    return Err(asm
                        .make_user_error("Argument for RMB must be positive", node, true)
                        .into());
                };

                asm.add_fixup(id, Item::Skip(bytes as usize), current_scope_id);
                self.advance_pc(bytes as usize);
            }

            CpuSpecific(i) => { 
                C::size_node(self,asm,id,i.clone())?;
            },

            AssignmentFromPc(LabelDefinition::Scoped(symbol_id)) => {
                let pcv = if node.first_child().is_some() {
                    // If we have an arg then evaluate the arg
                    asm.eval_first_arg(node, current_scope_id)?.0
                } else {
                    // Otherwise it's just the current PC
                    self.get_pc() as i64
                };

                asm.set_symbol_value(*symbol_id, pcv as usize).unwrap();
            }

            TokenizedFile(..) => {
                for c in asm.get_node_children(node) {
                    self.size_node(asm, c)?;
                }
            }

            Fdb(num_of_words) => self.advance_pc(*num_of_words * 2),

            Fcb(num_of_bytes) => {
                self.advance_pc(*num_of_bytes);
            }

            Fcc(text) => {
                self.advance_pc(text.as_bytes().len());
            }

            Zmb => {
                let (v, _) = asm.eval_first_arg(node, current_scope_id)?;
                assert!(v >= 0);
                self.advance_pc(v as usize)
            }

            Zmd => {
                let (v, _) = asm.eval_first_arg(node, current_scope_id)?;
                assert!(v >= 0);
                self.advance_pc((v * 2) as usize)
            }

            Fill => {
                let (size, _val) = asm.eval_two_args(node, current_scope_id)?;
                assert!(size >= 0);
                self.advance_pc(size as usize);
            }


            IncBin(file_name) => {
                let r = self.get_binary_extents(asm, file_name, node)?;
                let new_item = IncBinResolved {
                    file: file_name.clone(),
                    r: r.clone(),
                };

                asm.add_fixup(id, new_item, current_scope_id);
                self.advance_pc(r.len())
            }

            PostFixExpr | WriteBin(..) | IncBinRef(..) | Assignment(..) | Comment(..)
            | StructDef(..) | MacroDef(..) | MacroCall(..) | Import => (),

            _ => {
                let msg = format!("Unable to size {i:?}");
                return Err(asm.make_user_error(msg, node, true).into());
            }
        };

        Ok(())
    }

    fn get_binary_extents<P: AsRef<Path>>(
        &self,
        asm: &Assembler<C>,
        file_name: P,
        node: AstNodeRef<C>,
    ) -> GResult<std::ops::Range<usize>> {
        use itertools::Itertools;

        let data_len = asm.get_file_size(&file_name)?;

        let mut r = 0..data_len;

        let current_scope_id = self.scopes.scope();

        if let Some((offset, size)) = node.children().collect_tuple() {
            let offset = asm.eval_node(offset, current_scope_id)?;
            let size = asm.eval_node(size, current_scope_id)?;
            let offset_usize = offset as usize;
            let size_usize = size as usize;
            let last = (offset_usize + size_usize) - 1;

            if !(r.contains(&offset_usize) && r.contains(&last)) {
                let msg =
                    format!("Trying to grab {offset:04X} {size:04X} from file size {data_len:X}");
                return Err(asm.make_user_error(msg, node, true).into());
            };

            r.start = offset_usize;
            r.end = offset_usize + size_usize;
        } else {
            panic!("Should not happen!")
        }

        Ok(r)
    }

    pub fn get_node(&self, id: AstNodeId) -> AstNodeRef<C> {
        self.tree.as_ref().get(id).expect("Can't fetch node")
    }
}
