#![forbid(unused_imports)]

use super::{scopetracker::ScopeTracker, Assembler};

/// Take the AST and work out the sizes of everything
/// Resolve labels where we can
use crate::{
    debug_mess,
    error::GResult,
    frontend::{AstNodeKind, LabelDefinition},
    semantic::{Ast, AstNodeId, AstNodeRef},
};

// use crate::cpu6809::Compiler6809;

/// Ast tree sizer
/// gets the size of everything
/// assigns values to labels that
/// are defined by value of PC
pub struct Sizer<'a> {
    pub tree: &'a Ast,
    pub scopes: ScopeTracker,
    pub pc: usize,
}

pub fn size(asm: &mut Assembler, ast_tree: &Ast) -> GResult<()> {
    let _ = Sizer::try_new(ast_tree, asm)?;
    Ok(())
}

impl<'a> Sizer<'a> {
    pub fn try_new(tree: &'a Ast, asm: &mut Assembler) -> GResult<Sizer<'a>> {
        let pc = 0;

        asm.set_pc_symbol(pc).expect("Can't set PC symbol");

        let root_id = asm.get_symbols().get_root_scope_id();

        let mut ret = Self {
            tree,
            scopes: ScopeTracker::new(root_id),
            pc,
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

    fn size_node(&mut self, asm: &mut Assembler, id: AstNodeId) -> GResult<()> {
        use AstNodeKind::*;

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
                asm.add_fixup(id, AstNodeKind::SetPc(pc), current_scope_id);
                self.set_pc(pc);
            }

            SetPc(val) => {
                self.set_pc(*val);
            }

            Put => {
                let (value, _) = asm.eval_first_arg(node, current_scope_id)?;
                let offset = (value - self.get_pc() as i64) as isize;
                asm.add_fixup(id, AstNodeKind::SetPutOffset(offset), current_scope_id);
            }

            Rmb => {
                let (bytes, _) = asm.eval_first_arg(node, current_scope_id)?;

                if bytes < 0 {
                    return Err(asm
                        .make_user_error("Argument for RMB must be positive", node, true)
                        .into());
                };

                asm.add_fixup(id, AstNodeKind::Skip(bytes as usize), current_scope_id);
                self.advance_pc(bytes as usize);
            }

            TargetSpecific(_i) => {
                // C::size_node(self, asm, id, i.clone())?;
                todo!()
            }

            AssignmentFromPc(LabelDefinition::Scoped(symbol_id)) => {
                let pcv = if node.first_child().is_some() {
                    // If we have an arg then evaluate the arg
                    asm.eval_first_arg(node, current_scope_id)?.0
                } else {
                    // Otherwise it's just the current PC
                    self.get_pc() as i64
                };

                let sym = asm
                    .get_symbols()
                    .get_symbol_info_from_id(*symbol_id)
                    .unwrap();
                debug_mess!("Assigning {} = ${:04x}", sym.name(), pcv);

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
                let r = asm.get_binary_extents(asm, file_name, node, current_scope_id)?;
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

    pub fn get_node(&self, id: AstNodeId) -> AstNodeRef {
        self.tree.as_ref().get(id).expect("Can't fetch node")
    }
}
