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
    pub sections: std::collections::HashMap<String, (usize, usize, Option<usize>)>,
    pub current_section: Option<String>,
}

pub fn size(asm: &mut Assembler, ast_tree: &Ast) -> GResult<()> {
    let _ = Sizer::try_new(ast_tree, asm)?;
    Ok(())
}

impl<'a> Sizer<'a> {
    pub fn try_new(tree: &'a Ast, asm: &mut Assembler) -> GResult<Sizer<'a>> {
        let pc = 0;

        asm.set_pc_symbol_internal(pc)?;

        let root_id = asm.get_symbols().get_root_scope_id();

        let mut ret = Self {
            tree,
            scopes: ScopeTracker::new(root_id),
            pc,
            sections: Default::default(),
            current_section: None,
        };

        let id = ret.tree.as_ref().root().id();
        ret.size_node(asm, id)?;
        ret.check_section_bounds(asm)?;

        Ok(ret)
    }

    pub fn check_section_bounds(&self, asm: &mut Assembler) -> GResult<()> {
        for (name, (start, pc, max_size)) in &self.sections {
            if let Some(max) = max_size {
                let used = pc.saturating_sub(*start);
                if used > *max {
                    let root_node = self.tree.as_ref().root();
                    let msg = format!(
                        "Section '{name}' overflowed by {} bytes (allocated {used} bytes, maximum size {max} bytes)",
                        used - max
                    );
                    return Err(asm.make_user_error(msg, root_node, true).into());
                }
            }
        }
        Ok(())
    }

    pub fn advance_pc(&mut self, val: usize) {
        assert!(self.pc + val <= 65536);
        self.pc += val;
        if let Some(cur_name) = &self.current_section {
            if let Some(state) = self.sections.get_mut(cur_name) {
                state.1 = self.pc;
            }
        }
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

        asm.set_pc_symbol_internal(self.get_pc())?;

        match &i {
            MacroCallProcessed {
                scope_id, macro_id, ..
            } => {
                asm.eval_macro_args_node(*scope_id, id, self.tree);

                self.scopes.push(*scope_id);

                let m_node = self.get_node(*macro_id);
                let children: Vec<_> = m_node.children().map(|n| n.id()).collect();
                for c in children {
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

            Section(name) => {
                let children: Vec<_> = node.children().collect();
                if let Some(cur) = &self.current_section {
                    if let Some(state) = self.sections.get_mut(cur) {
                        state.1 = self.pc;
                    }
                }

                if !children.is_empty() {
                    let start = asm.eval_node(children[0], current_scope_id)? as usize;
                    let size = if children.len() > 1 {
                        Some(asm.eval_node(children[1], current_scope_id)? as usize)
                    } else {
                        None
                    };

                    self.sections.insert(name.clone(), (start, start, size));
                    self.current_section = Some(name.clone());
                    self.set_pc(start);
                    asm.add_fixup(id, AstNodeKind::SetPc(start), current_scope_id);
                } else if let Some(state) = self.sections.get_mut(name) {
                    let pc = state.1;
                    self.current_section = Some(name.clone());
                    self.set_pc(pc);
                    asm.add_fixup(id, AstNodeKind::SetPc(pc), current_scope_id);
                } else {
                    return Err(asm
                        .make_user_error(format!("Unknown section '{name}'"), node, true)
                        .into());
                }
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

            TargetSpecific(node_kind) => {
                asm.size_node(self, id, node_kind.clone(), current_scope_id)?;
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

                asm.set_symbol_value_internal(*symbol_id, pcv as usize)?;
            }

            TokenizedFile(..) => {
                for c in asm.get_node_children(node) {
                    self.size_node(asm, c)?;
                }
            }

            Block => {
                for c in asm.get_node_children(node) {
                    self.size_node(asm, c)?;
                }
            }

            Fdb(num_of_words) => self.advance_pc(*num_of_words * 2),

            Fcb(num_of_bytes) => {
                self.advance_pc(*num_of_bytes);
            }

            Fcc(text) => {
                self.advance_pc(text.len());
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

    pub fn get_node(&self, id: AstNodeId) -> AstNodeRef<'a> {
        self.tree.as_ref().get(id).expect("Can't fetch node")
    }
}

#[cfg(test)]
mod tests {
    use crate::assembler::Assembler;
    use crate::opts::Opts;

    #[test]
    fn test_sections_independent_location_counters() {
        let src = r#"
            section rom_01, start = $1000, size = $1000
ENTRY:      nop
            nop
            section dp_ram, start = $9800, size = $100
VAR1:       rmb 2
VAR2:       rmb 1
            section rom_01
CONT:       nop
        "#;
        let path = std::env::temp_dir().join("gazm_section_test.gazm");
        std::fs::write(&path, src).unwrap();

        let opts = Opts {
            project_file: path.clone(),
            assemble_dir: Some(std::env::temp_dir()),
            ..Default::default()
        };
        let mut asm = Assembler::new(opts);
        let res = asm.assemble();
        let _ = std::fs::remove_file(&path);
        assert!(res.is_ok(), "Assembly failed: {:?}", res.err());

        let syms = asm.get_symbols();
        let root_scope = syms.get_root_scope_id();
        let entry_val = syms.get_symbol_info("ENTRY", root_scope).unwrap().value;
        let var1_val = syms.get_symbol_info("VAR1", root_scope).unwrap().value;
        let var2_val = syms.get_symbol_info("VAR2", root_scope).unwrap().value;
        let cont_val = syms.get_symbol_info("CONT", root_scope).unwrap().value;

        assert_eq!(entry_val, Some(0x1000));
        assert_eq!(var1_val, Some(0x9800));
        assert_eq!(var2_val, Some(0x9802));
        assert_eq!(cont_val, Some(0x1002));
    }

    #[test]
    fn test_section_overflow() {
        let src = r#"
            section small_sec, start = $1000, size = 4
            nop
            nop
            nop
            nop
            nop
        "#;
        let path = std::env::temp_dir().join("gazm_section_overflow_test.gazm");
        std::fs::write(&path, src).unwrap();

        let opts = Opts {
            project_file: path.clone(),
            assemble_dir: Some(std::env::temp_dir()),
            ..Default::default()
        };
        let mut asm = Assembler::new(opts);
        let res = asm.assemble();
        let _ = std::fs::remove_file(&path);
        assert!(
            res.is_err(),
            "Expected overflow error but assembly succeeded"
        );
    }
}
