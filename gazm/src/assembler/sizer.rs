#![forbid(unused_imports)]

use super::{plan::PlanEntry, scopetracker::ScopeTracker, Assembler};

/// Take the AST and work out the sizes of everything
/// Resolve labels where we can
use crate::{
    debug_mess,
    error::GResult,
    frontend::{AstNodeKind, AstNodeKindDiscriminants, LabelDefinition},
    gazmsymbols::SymbolScopeId,
    semantic::{Ast, AstNodeId, AstNodeRef},
};

// use crate::cpu6809::Compiler6809;

/// Ast tree sizer
/// gets the size of everything
/// assigns values to labels that
/// are defined by value of PC
/// emits a [`PlanEntry`] per statement for the compiler to replay
pub struct Sizer<'a> {
    pub tree: &'a Ast,
    pub scopes: ScopeTracker,
    pub pc: usize,
    pub sections: std::collections::HashMap<String, (usize, usize, Option<usize>)>,
    pub current_section: Option<String>,
    /// The linear walk plan handed to the compiler.
    pub plan: Vec<PlanEntry>,
    /// Loop-index symbol values in effect for statements we are about to emit
    /// (one entry per enclosing `repeat`).
    bindings: Vec<(SymbolScopeId, i64)>,
    /// One-slot handoff from the CPU-specific sizers: when an instruction's
    /// encoding is decided at size time (extended -> direct, indexed offset
    /// width, ...) they record the replacement here and the shared
    /// `TargetSpecific` arm turns it into the plan entry kind.
    pending_cpu_fixup: Option<(AstNodeId, AstNodeKind)>,
}

/// Walk the tree, compute layout, and return the linear walk plan for the
/// compiler to replay. Symbol values (labels, PC symbol, macro parameters,
/// loop indices) are set as side effects and persist for the compiler.
pub fn size(asm: &mut Assembler, ast_tree: &Ast) -> GResult<Vec<PlanEntry>> {
    let sizer = Sizer::try_new(ast_tree, asm)?;
    Ok(sizer.plan)
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
            plan: Vec::new(),
            bindings: Vec::new(),
            pending_cpu_fixup: None,
        };

        let id = ret.tree.as_ref().root().id();
        ret.size_node(asm, id)?;
        ret.check_section_bounds(asm)?;

        Ok(ret)
    }

    /// Record one statement in the plan. `kind` is the *final* node kind
    /// (fixups such as `Org` -> `SetPc` are applied here, at plan-build time);
    /// the entry's PC is the current PC, i.e. the start of the statement.
    fn emit(&mut self, id: AstNodeId, kind: AstNodeKind) {
        self.emit_with_pc(id, kind, self.pc);
    }

    /// Like [`Self::emit`] but with an explicit PC, for the `TargetSpecific`
    /// arm where the CPU-specific sizer has already advanced the PC while
    /// sizing the instruction.
    fn emit_with_pc(&mut self, id: AstNodeId, kind: AstNodeKind, pc: usize) {
        self.plan.push(PlanEntry {
            scope_id: self.scopes.scope(),
            node_id: id,
            kind,
            pc,
            bindings: self.bindings.clone(),
        });
    }

    /// Hand the replacement encoding of a `TargetSpecific` node from a
    /// CPU-specific sizer back to the shared sizer. Only one instruction is
    /// being sized at a time, so a single slot suffices; it is consumed by
    /// the `TargetSpecific` arm of `size_node`.
    pub(crate) fn set_node_fixup<I: Into<AstNodeKind>>(&mut self, id: AstNodeId, kind: I) {
        self.pending_cpu_fixup = Some((id, kind.into()));
    }

    fn take_node_fixup(&mut self, id: AstNodeId) -> Option<AstNodeKind> {
        match &self.pending_cpu_fixup {
            Some((fixed_id, _)) if *fixed_id == id => {
                self.pending_cpu_fixup.take().map(|(_, kind)| kind)
            }
            _ => None,
        }
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
                // Record the call itself: pass 2 re-runs `eval_macro_args`
                // here so parameters whose arguments reference forward labels
                // (unevaluable at size time) get their values now that the
                // whole layout is known. The body statements are separate
                // plan entries that follow.
                self.emit(id, i.clone());
                asm.eval_macro_args_node(*scope_id, id, self.tree);

                self.scopes.push(*scope_id);

                let m_node = self.get_node(*macro_id);
                let children: Vec<_> = m_node.children().map(|n| n.id()).collect();
                for c in children {
                    self.size_node(asm, c)?;
                }

                self.scopes.pop();
            }

            Repeat { index } => {
                // First child is the count expression; the rest is the body.
                let mut children = node.children();
                let count_node =
                    children
                        .next()
                        .ok_or_else(|| -> crate::error::GazmErrorKind {
                            let diag = asm.make_user_error(
                                "repeat requires a count expression",
                                node,
                                true,
                            );
                            diag.into()
                        })?;
                let count = asm.eval_node(count_node, current_scope_id)?;
                let body: Vec<_> = children.map(|n| n.id()).collect();

                // The scoping pass already resolved the index name to a real
                // symbol (if the body referenced it); find its id once, then
                // just set its value per iteration. When the index is
                // declared but unused there is nothing to bind.
                let index_id = index.as_deref().and_then(|name| {
                    let reader = asm.get_symbols().get_reader(current_scope_id);
                    reader.get_symbol_info(name).ok().map(|si| si.symbol_id)
                });

                // Each body statement is recorded in the plan once per
                // iteration, carrying the index binding so the compiler
                // never has to re-derive the loop.
                for iteration in 0..count {
                    if let Some(index_id) = index_id {
                        asm.get_symbols_mut()
                            .set_value_for_id(index_id, iteration)
                            .map_err(|e| -> crate::error::GazmErrorKind {
                                asm.make_user_error(format!("repeat: {e}"), node, true)
                                    .into()
                            })?;
                        self.bindings.push((index_id, iteration));
                    }
                    for c in &body {
                        self.size_node(asm, *c)?;
                    }
                    if index_id.is_some() {
                        self.bindings.pop();
                    }
                }
            }

            ScopeId(scope_id) => {
                self.scopes.set_scope(*scope_id);
                self.emit(id, i.clone());
            }

            GrabMem => {
                let args = asm.eval_n_args(node, 2, current_scope_id)?;
                let size = args[1];
                self.emit(id, i.clone());
                self.advance_pc(size as usize);
            }

            Org => {
                let pc = asm.eval_first_arg(node, current_scope_id)?.0 as usize;
                self.emit(id, AstNodeKind::SetPc(pc));
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
                    self.emit(id, AstNodeKind::SetPc(start));
                    self.set_pc(start);
                } else if let Some(state) = self.sections.get_mut(name) {
                    let pc = state.1;
                    self.current_section = Some(name.clone());
                    self.emit(id, AstNodeKind::SetPc(pc));
                    self.set_pc(pc);
                } else {
                    return Err(asm
                        .make_user_error(format!("Unknown section '{name}'"), node, true)
                        .into());
                }
            }

            SetPc(val) => {
                self.emit(id, i.clone());
                self.set_pc(*val);
            }

            Put => {
                let (value, _) = asm.eval_first_arg(node, current_scope_id)?;
                let offset = (value - self.get_pc() as i64) as isize;
                self.emit(id, AstNodeKind::SetPutOffset(offset));
            }

            Rmb => {
                let (bytes, _) = asm.eval_first_arg(node, current_scope_id)?;

                if bytes < 0 {
                    return Err(asm
                        .make_user_error("Argument for RMB must be positive", node, true)
                        .into());
                };

                self.emit(id, AstNodeKind::Skip(bytes as usize));
                self.advance_pc(bytes as usize);
            }

            TargetSpecific(node_kind) => {
                // Capture the PC before sizing: the CPU-specific sizer
                // advances it while working out the instruction size.
                let pc = self.pc;
                asm.size_node(self, id, node_kind.clone(), current_scope_id)?;
                // The CPU-specific sizer may have decided on a different
                // encoding (e.g. extended -> direct); use that as the final
                // kind if it did.
                let kind = self.take_node_fixup(id).unwrap_or_else(|| i.clone());
                self.emit_with_pc(id, kind, pc);
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
                self.emit(id, i.clone());
            }

            TokenizedFile(..) => {
                self.emit(id, i.clone());
                for c in asm.get_node_children(node) {
                    self.size_node(asm, c)?;
                }
            }

            Block => {
                self.emit(id, i.clone());
                for c in asm.get_node_children(node) {
                    self.size_node(asm, c)?;
                }
            }

            Fdb(num_of_words) => {
                self.emit(id, i.clone());
                self.advance_pc(*num_of_words * 2);
            }

            Fcb(num_of_bytes) => {
                self.emit(id, i.clone());
                self.advance_pc(*num_of_bytes);
            }

            Fcc(text) => {
                self.emit(id, i.clone());
                self.advance_pc(text.len());
            }

            Zmb => {
                let (v, _) = asm.eval_first_arg(node, current_scope_id)?;
                assert!(v >= 0);
                self.emit(id, i.clone());
                self.advance_pc(v as usize)
            }

            Zmd => {
                let (v, _) = asm.eval_first_arg(node, current_scope_id)?;
                assert!(v >= 0);
                self.emit(id, i.clone());
                self.advance_pc((v * 2) as usize)
            }

            Fill => {
                let (size, _val) = asm.eval_two_args(node, current_scope_id)?;
                assert!(size >= 0);
                self.emit(id, i.clone());
                self.advance_pc(size as usize);
            }

            IncBin(file_name) => {
                let r = asm.get_binary_extents(asm, file_name, node, current_scope_id)?;
                let new_item = IncBinResolved {
                    file: file_name.clone(),
                    r: r.clone(),
                };

                self.emit(id, new_item);
                self.advance_pc(r.len())
            }

            // Statements with compile-time effects (or none at all) are still
            // recorded in the plan so the compiler processes them in order
            // and the source map is unchanged.
            WriteBin(..) | IncBinRef(..) | Assignment(..) | Comment(..) | StructDef(..)
            | MacroDef(..) | MacroCall(..) | Import | Exec => {
                self.emit(id, i.clone());
            }

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

/// True if `size_node` records a [`PlanEntry`] for this node kind — i.e. the
/// kind can appear in the walk plan. Kinds that are replaced by fixups at
/// plan-build time (`Org` -> `SetPc`, `Rmb` -> `Skip`, ...) and walk
/// scaffolding (`MacroCallProcessed`, `Repeat`) are *not* plan kinds.
///
/// Mirrors the `match` in `Sizer::size_node` — keep the two in sync. The
/// coverage test in `mod.rs` asserts this set equals the set the compiler
/// can replay, so a kind added to one pass but not the other fails loudly
/// instead of emitting wrong bytes.
///
/// Note: this works at discriminant granularity, so `AssignmentFromPc` counts
/// as a plan kind even though the real match only has an arm for the `Scoped`
/// label variant (unscoped labels are resolved by the semantic pass before
/// assembly and error here).
pub(crate) fn sizer_emits(kind: &AstNodeKindDiscriminants) -> bool {
    use AstNodeKindDiscriminants::*;
    matches!(
        kind,
        MacroCallProcessed
            | ScopeId
            | GrabMem
            | SetPc
            | SetPutOffset
            | Skip
            | TargetSpecific
            | AssignmentFromPc
            | TokenizedFile
            | Block
            | Fdb
            | Fcb
            | Fcc
            | Zmb
            | Zmd
            | Fill
            | IncBinResolved
            | WriteBin
            | IncBinRef
            | Exec
            | Assignment
            | Comment
            | StructDef
            | MacroDef
            | MacroCall
            | Import
    )
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

    /// Assemble `src` as a Cpu6809 project and return the assembled bytes at
    /// `addr`. `name` must be unique per test — tests run in parallel and
    /// share the temp dir.
    fn assemble_bytes(name: &str, src: &str, addr: usize, count: usize) -> Vec<u8> {
        let path = std::env::temp_dir().join(format!("gazm_{name}.gazm"));
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

        asm.get_binary()
            .get_bytes(addr, count)
            .expect("Can't read bytes")
            .to_vec()
    }

    #[test]
    fn repeat_in_macro_emits_indexed_table() {
        let src = r#"
            macro gen_table(base, count) {
                repeat count, i {
                    fdb base + i*2
                }
            }

            org $1000
            start: gen_table($10, 4)
        "#;

        // 4 x fdb at $1000: 0x10, 0x12, 0x14, 0x16
        assert_eq!(
            assemble_bytes("repeat_macro_table", src, 0x1000, 8),
            vec![0x00, 0x10, 0x00, 0x12, 0x00, 0x14, 0x00, 0x16]
        );
    }

    #[test]
    fn repeat_without_index_still_iterates() {
        let src = r#"
            org $1000
            repeat 3 {
                fcb 7
            }
        "#;

        assert_eq!(
            assemble_bytes("repeat_no_index", src, 0x1000, 3),
            vec![7, 7, 7]
        );
    }

    #[test]
    fn repeat_keyword_is_not_reserved() {
        // `repeat` must stay usable as a symbol name (robotron defines
        // REPEAT as data), and a command word followed by a colon is a
        // label definition.
        let src = r#"
            REPEAT: equ $C0
            FCB REPEAT+6
            FDB: equ $1234

            org $1000
            fcb REPEAT+6
            fdb FDB
        "#;

        assert_eq!(
            assemble_bytes("repeat_keyword_symbol", src, 0x1000, 3),
            vec![0xC6, 0x12, 0x34]
        );
    }

    #[test]
    fn command_words_can_be_labels_and_symbols() {
        let src = r#"
            FDB: equ $1234
            ORG: equ $5678
            FCB: equ 2

            org $1000
            fdb FDB, ORG
            fcb FCB
        "#;

        assert_eq!(
            assemble_bytes("command_words_symbols", src, 0x1000, 5),
            vec![0x12, 0x34, 0x56, 0x78, 0x02]
        );
    }
}
