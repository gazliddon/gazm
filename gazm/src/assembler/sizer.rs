#![forbid(unused_imports)]

use super::{plan::PlanEntry, scopetracker::ScopeTracker, Assembler};

/// Cap on `while` iterations so a condition that never becomes zero is a
/// clean error instead of an assembly-time hang. Bounded by the 64K
/// address space: a body that advances the PC cannot legitimately iterate
/// more than 65536 times, and a non-terminating condition hits the cap
/// before the PC-overflow assert.
const MAX_WHILE_ITERATIONS: u64 = 65_536;

/// Loop-control signal raised by a `break`/`continue` statement and
/// consumed by the innermost enclosing loop arm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoopSignal {
    Break,
    Continue,
}

/// Take the AST and work out the sizes of everything
/// Resolve labels where we can
use crate::{
    debug_mess,
    error::GResult,
    frontend::{AstNodeKind, AstNodeKindDiscriminants, LabelDefinition, MsgPart},
    gazmsymbols::SymbolScopeId,
    sections::SectionDescriptor,
    semantic::{Ast, AstNodeId, AstNodeRef},
    status_mess,
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
    /// `break`/`continue` raised by a body statement, pending consumption
    /// by the innermost enclosing loop arm. Left set after the walk, the
    /// statement was outside any loop and that is an error.
    loop_signal: Option<(LoopSignal, AstNodeId)>,
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
            loop_signal: None,
        };

        let id = ret.tree.as_ref().root().id();
        ret.size_node(asm, id)?;
        ret.check_section_bounds(asm)?;

        // Persist the final sections for the v4 metadata header: the sizer
        // tracks (start, end, max_size) per section; physical == logical
        // here (put offsets are not modelled in the sizer) and access is
        // ReadWrite unless a future CPU path says otherwise.
        let sections: Vec<_> = ret
            .sections
            .iter()
            .map(|(name, (start, end, _max))| {
                let range = *start..*end;
                SectionDescriptor::new(
                    name,
                    range.clone(),
                    range,
                    crate::assembler::AccessType::ReadWrite,
                )
            })
            .collect();
        asm.asm_out.sections = sections;

        // A loop-control signal left over after the whole walk means a
        // `break`/`continue` appeared outside any loop.
        if let Some((_, signal_node)) = ret.loop_signal {
            let node = ret.get_node(signal_node);
            return Err(asm
                .make_user_error("break/continue outside of a loop", node, true)
                .into());
        }

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

    /// Build an interpolated `log`/`assert` message by walking the parts
    /// and evaluating the `Value(i)` expression children. `value_base` is
    /// the child index of the first value expression (1 for `assert`,
    /// whose child 0 is the condition; 0 for `log`). Values are formatted
    /// as decimal.
    fn format_msg(
        &mut self,
        asm: &mut Assembler,
        parts: &[MsgPart],
        node: AstNodeRef,
        current_scope_id: u64,
        value_base: usize,
    ) -> GResult<String> {
        let mut text = String::new();
        for part in parts {
            match part {
                MsgPart::Text(t) => text.push_str(t),
                MsgPart::Value(i) => {
                    let child = node.children().nth(value_base + *i).ok_or_else(
                        || -> crate::error::GazmErrorKind {
                            let diag = asm.make_user_error("message value missing", node, true);
                            diag.into()
                        },
                    )?;
                    let value = asm.eval_node(child, current_scope_id)?;
                    text.push_str(&value.to_string());
                }
            }
        }
        Ok(text)
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
                // never has to re-derive the loop. `break` stops the whole
                // loop; `continue` skips to the next iteration.
                'iter: for iteration in 0..count {
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
                        match self.loop_signal.take() {
                            Some((LoopSignal::Break, _)) => {
                                if index_id.is_some() {
                                    self.bindings.pop();
                                }
                                break 'iter;
                            }
                            Some((LoopSignal::Continue, _)) => break,
                            None => {}
                        }
                    }
                    if index_id.is_some() {
                        self.bindings.pop();
                    }
                }
            }

            Break => {
                self.loop_signal = Some((LoopSignal::Break, id));
            }

            Continue => {
                self.loop_signal = Some((LoopSignal::Continue, id));
            }

            For { index } => {
                // Children: start expression, end expression, then body.
                let mut children = node.children();
                let start_node =
                    children
                        .next()
                        .ok_or_else(|| -> crate::error::GazmErrorKind {
                            let diag =
                                asm.make_user_error("for requires a start expression", node, true);
                            diag.into()
                        })?;
                let end_node = children
                    .next()
                    .ok_or_else(|| -> crate::error::GazmErrorKind {
                        let diag =
                            asm.make_user_error("for requires an end expression", node, true);
                        diag.into()
                    })?;
                let start = asm.eval_node(start_node, current_scope_id)?;
                let end = asm.eval_node(end_node, current_scope_id)?;
                let body: Vec<_> = children.map(|n| n.id()).collect();

                // Same index-symbol machinery as `repeat`: the scoping pass
                // created the symbol, we just set its value per iteration.
                let index_id = {
                    let reader = asm.get_symbols().get_reader(current_scope_id);
                    reader.get_symbol_info(index).ok().map(|si| si.symbol_id)
                };

                'iter: for value in start..end {
                    if let Some(index_id) = index_id {
                        asm.get_symbols_mut()
                            .set_value_for_id(index_id, value)
                            .map_err(|e| -> crate::error::GazmErrorKind {
                                asm.make_user_error(format!("for: {e}"), node, true).into()
                            })?;
                        self.bindings.push((index_id, value));
                    }
                    for c in &body {
                        self.size_node(asm, *c)?;
                        match self.loop_signal.take() {
                            Some((LoopSignal::Break, _)) => {
                                if index_id.is_some() {
                                    self.bindings.pop();
                                }
                                break 'iter;
                            }
                            Some((LoopSignal::Continue, _)) => break,
                            None => {}
                        }
                    }
                    if index_id.is_some() {
                        self.bindings.pop();
                    }
                }
            }

            If => {
                // First child is the condition expression; the then-branch
                // statements follow, optionally ending with an `Else` node
                // whose children are the else-branch. Only the taken branch
                // is walked, so the plan contains just those statements —
                // the compiler never sees the construct or the other branch.
                let mut children = node.children();
                let cond_node = children
                    .next()
                    .ok_or_else(|| -> crate::error::GazmErrorKind {
                        let diag =
                            asm.make_user_error("if requires a condition expression", node, true);
                        diag.into()
                    })?;
                let cond = asm.eval_node(cond_node, current_scope_id)?;

                if cond != 0 {
                    for c in children {
                        if matches!(c.value().item, AstNodeKind::Else) {
                            break;
                        }
                        self.size_node(asm, c.id())?;
                    }
                } else {
                    for c in children {
                        if matches!(c.value().item, AstNodeKind::Else) {
                            for ec in c.children() {
                                self.size_node(asm, ec.id())?;
                            }
                            break;
                        }
                    }
                }
            }

            While => {
                // First child is the condition; the rest is the body. The
                // condition is re-evaluated each iteration and the body
                // assembles while it is non-zero. Iterations are capped so
                // a non-terminating condition is a clean error, not a hang.
                let mut children = node.children();
                let cond_node = children
                    .next()
                    .ok_or_else(|| -> crate::error::GazmErrorKind {
                        let diag = asm.make_user_error(
                            "while requires a condition expression",
                            node,
                            true,
                        );
                        diag.into()
                    })?;
                let body: Vec<_> = children.map(|n| n.id()).collect();

                let mut iterations: u64 = 0;
                // `break` exits the loop; `continue` re-evaluates the
                // condition (the next iteration).
                'looping: loop {
                    // `*` in the condition means the current assembly PC.
                    asm.set_pc_symbol_internal(self.pc)?;
                    let cond = asm.eval_node(cond_node, current_scope_id)?;
                    if cond == 0 {
                        break;
                    }
                    iterations += 1;
                    if iterations > MAX_WHILE_ITERATIONS {
                        return Err(asm
                            .make_user_error(
                                format!(
                                    "while loop exceeded {MAX_WHILE_ITERATIONS} iterations \
                                     (condition never became zero?)"
                                ),
                                node,
                                true,
                            )
                            .into());
                    }
                    for c in &body {
                        self.size_node(asm, *c)?;
                        match self.loop_signal.take() {
                            Some((LoopSignal::Break, _)) => break 'looping,
                            Some((LoopSignal::Continue, _)) => break,
                            None => {}
                        }
                    }
                }
            }

            ScopeId(scope_id) => {
                self.scopes.set_scope(*scope_id);
                self.emit(id, i.clone());
            }

            // `assert <condition> [, message]`: evaluate at sizing time; a
            // false condition is a (non-fatal) error so several asserts
            // report together. No plan entry — the compiler never sees it,
            // exactly like if/while. Child 0 is the condition; the message
            // value expressions follow (MsgPart::Value indexes them from 1).
            Assert(msg) => {
                let cond_node =
                    node.first_child()
                        .ok_or_else(|| -> crate::error::GazmErrorKind {
                            let diag =
                                asm.make_user_error("assert requires a condition", node, true);
                            diag.into()
                        })?;
                asm.set_pc_symbol_internal(self.pc)?;
                let cond = asm.eval_node(cond_node, current_scope_id)?;
                if cond == 0 {
                    let text = self.format_msg(asm, msg, node, current_scope_id, 1)?;
                    let text = if text.is_empty() {
                        String::new()
                    } else {
                        format!(": {text}")
                    };
                    let diag = asm.make_user_error(format!("assertion failed{text}"), node, false);
                    if asm.asm_out.errors.push(diag) {
                        return Ok(());
                    }
                }
            }

            // `log <message>`: print during sizing. The message is a
            // sequence of text and `{expr}` parts (children are the value
            // expressions, MsgPart::Value indexes them from 0).
            Log(msg) => {
                asm.set_pc_symbol_internal(self.pc)?;
                let text = self.format_msg(asm, msg, node, current_scope_id, 0)?;
                if !text.is_empty() {
                    status_mess!("{text}");
                }
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

            ReserveBytes => {
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

            EmitWords(num_of_words) => {
                self.emit(id, i.clone());
                self.advance_pc(*num_of_words * 2);
            }

            EmitBytes(num_of_bytes) => {
                self.emit(id, i.clone());
                self.advance_pc(*num_of_bytes);
            }

            EmitString(text) => {
                self.emit(id, i.clone());
                self.advance_pc(text.len());
            }

            ZeroWords => {
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
/// plan-build time (`Org` -> `SetPc`, `ReserveBytes` -> `Skip`, ...) and walk
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
            | EmitWords
            | EmitBytes
            | EmitString
            | ZeroWords
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

    #[test]
    fn if_taken_assembles_then_branch() {
        let src = r#"
            org $1000
            if 1 { fcb 1 } else { fcb 2 }
        "#;
        assert_eq!(assemble_bytes("if_taken", src, 0x1000, 1), vec![1]);
    }

    #[test]
    fn if_not_taken_assembles_else_branch() {
        let src = r#"
            org $1000
            if 0 { fcb 1 } else { fcb 2 }
        "#;
        assert_eq!(assemble_bytes("if_else", src, 0x1000, 1), vec![2]);
    }

    #[test]
    fn if_without_else_skips_body() {
        let src = r#"
            org $1000
            if 0 { fcb 1 }
            fcb 9
        "#;
        assert_eq!(assemble_bytes("if_no_else", src, 0x1000, 1), vec![9]);
    }

    #[test]
    fn else_if_chain_takes_last_matching() {
        let src = r#"
            org $1000
            if 0 { fcb 1 } else if 0 { fcb 2 } else { fcb 3 }
        "#;
        assert_eq!(assemble_bytes("if_chain", src, 0x1000, 1), vec![3]);
    }

    #[test]
    fn while_loop_terminates_on_pc_condition() {
        // `*` is the current assembly PC; the loop writes 3 bytes then stops.
        let src = r#"
            org $1000
            while * < $1003 {
                fcb 0
            }
        "#;
        assert_eq!(assemble_bytes("while_pc", src, 0x1000, 3), vec![0, 0, 0]);
    }

    #[test]
    fn while_nonterminating_condition_is_an_error() {
        let src = r#"
            while 1 {
                fcb 0
            }
        "#;
        let path = std::env::temp_dir().join("gazm_while_infinite.gazm");
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
            "Expected while-loop cap error but assembly succeeded"
        );
    }

    #[test]
    fn if_inside_macro_uses_argument() {
        let src = r#"
            macro m(c) {
                if c { fcb 1 } else { fcb 2 }
            }

            org $1000
            m(0)
        "#;
        assert_eq!(assemble_bytes("if_macro", src, 0x1000, 1), vec![2]);
    }

    #[test]
    fn if_keywords_are_not_reserved() {
        // `if`/`else`/`while` stay usable as symbol names.
        let src = r#"
            IF: equ 1
            ELSE: equ 2
            WHILE: equ 3

            org $1000
            fcb IF + ELSE + WHILE
        "#;
        assert_eq!(
            assemble_bytes("if_keywords_symbols", src, 0x1000, 1),
            vec![6]
        );
    }

    #[test]
    fn comparison_operators_evaluate_to_zero_or_one() {
        let src = r#"
            org $1000
            fcb 3 == 3, 3 != 3, 2 < 1, 2 <= 2, 5 > 4, 5 >= 6
        "#;
        assert_eq!(
            assemble_bytes("comparison_ops", src, 0x1000, 6),
            vec![1, 0, 0, 1, 1, 0]
        );
    }

    #[test]
    fn logical_operators_use_nonzero_truthiness() {
        let src = r#"
            org $1000
            fcb 5 && 3, 0 && 3, 6 || 0, 0 || 0, 1 && 2 || 0
        "#;
        assert_eq!(
            assemble_bytes("logical_ops", src, 0x1000, 5),
            vec![1, 0, 1, 0, 1]
        );
    }

    #[test]
    fn logical_ops_bind_looser_than_comparisons() {
        // `&&`/`||` must not need parens: `3 > 2 && 1 < 2` is
        // `(3 > 2) && (1 < 2)`, not `3 > (2 && 1) < 2`.
        let src = r#"
            org $1000
            if 3 > 2 && 1 < 2 { fcb 1 } else { fcb 2 }
        "#;
        assert_eq!(assemble_bytes("logical_condition", src, 0x1000, 1), vec![1]);
    }

    #[test]
    fn break_exits_repeat_early() {
        let src = r#"
            org $1000
            repeat 10, i {
                if i == 3 { break }
                fcb i
            }
        "#;
        assert_eq!(
            assemble_bytes("break_repeat", src, 0x1000, 3),
            vec![0, 1, 2]
        );
    }

    #[test]
    fn continue_skips_iteration() {
        let src = r#"
            org $1000
            repeat 5, i {
                if i == 2 { continue }
                fcb i
            }
        "#;
        assert_eq!(
            assemble_bytes("continue_repeat", src, 0x1000, 4),
            vec![0, 1, 3, 4]
        );
    }

    #[test]
    fn break_exits_while() {
        let src = r#"
            org $1000
            while * < $1006 {
                fcb 1
                if * == $1002 { break }
            }
        "#;
        assert_eq!(assemble_bytes("break_while", src, 0x1000, 2), vec![1, 1]);
    }

    #[test]
    fn break_outside_loop_is_an_error() {
        let src = r#"
            org $1000
            break
        "#;
        let path = std::env::temp_dir().join("gazm_break_outside.gazm");
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
            "Expected break-outside-loop error but assembly succeeded"
        );
    }

    #[test]
    fn for_loop_emits_range() {
        let src = r#"
            org $1000
            for i in 0..4 { fcb i }
        "#;
        assert_eq!(
            assemble_bytes("for_range", src, 0x1000, 4),
            vec![0, 1, 2, 3]
        );
    }

    #[test]
    fn for_loop_with_expression_bounds() {
        let src = r#"
            org $1000
            for i in 2..5 { fcb i*2 }
        "#;
        assert_eq!(assemble_bytes("for_expr", src, 0x1000, 3), vec![4, 6, 8]);
    }

    #[test]
    fn for_loop_inside_macro() {
        let src = r#"
            macro table(n) {
                for i in 0..n { fcb i }
            }

            org $1000
            table(3)
        "#;
        assert_eq!(assemble_bytes("for_macro", src, 0x1000, 3), vec![0, 1, 2]);
    }

    #[test]
    fn break_continue_keywords_are_not_reserved() {
        let src = r#"
            BREAK: equ 1
            CONTINUE: equ 2

            org $1000
            fcb BREAK + CONTINUE
        "#;
        assert_eq!(
            assemble_bytes("break_keywords_symbols", src, 0x1000, 1),
            vec![3]
        );
    }

    #[test]
    fn dot_labels_still_lex() {
        // Leading-dot labels (.COAST style) survive the dot-free identifier
        // change; `..` is now the range operator.
        let src = r#"
            .IF: equ 5

            org $1000
            fcb .IF
        "#;
        assert_eq!(assemble_bytes("dot_labels", src, 0x1000, 1), vec![5]);
    }

    #[test]
    fn struct_fields_resolve_scoped() {
        let src = r#"
            struct proc {
                link : word
                addr : word
                time : byte
                cod : byte[4]
            }

            org $1000
            fdb proc::link, proc::addr
            fcb proc::time, proc::cod, sizeof(proc)
        "#;
        // link=0 addr=2 time=4 cod=5 sizeof=9
        assert_eq!(
            assemble_bytes("struct_scoped", src, 0x1000, 7),
            vec![0x00, 0x00, 0x00, 0x02, 4, 5, 9]
        );
    }

    #[test]
    fn sizeof_returns_struct_size_in_expressions() {
        let src = r#"
            struct proc {
                link : word
                addr : word
                time : byte
                cod : byte[4]
            }

            org $1000
            fcb sizeof(proc)
            fdb sizeof(proc) * 85
        "#;
        // sizeof = 9; table of 85 entries = 765
        assert_eq!(
            assemble_bytes("struct_sizeof", src, 0x1000, 3),
            vec![9, 0x02, 0xFD] // 765 = 0x02FD
        );
    }

    #[test]
    fn sizeof_unknown_struct_is_an_error() {
        let src = r#"
            org $1000
            fcb sizeof(nope)
        "#;
        let err = assemble_error("sizeof_unknown", src);
        assert!(
            err.contains("Unknown struct nope"),
            "expected an unknown-struct error, got: {err}"
        );
    }

    #[test]
    fn struct_accepts_comma_form_for_back_compat() {
        let src = r#"
            struct proc { link : word, addr : word, time : byte }

            org $1000
            fcb proc::link, proc::addr, proc::time
        "#;
        assert_eq!(
            assemble_bytes("struct_comma", src, 0x1000, 3),
            vec![0, 2, 4]
        );
    }

    #[test]
    fn struct_flat_names_are_gone() {
        // The old `Name.field` form no longer resolves — fields are
        // `Name::field` only.
        let src = r#"
            struct proc { link : word, addr : word }

            org $1000
            fcb proc.link
        "#;
        let path = std::env::temp_dir().join("gazm_struct_flat_gone.gazm");
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
            "Expected flat struct name to be unresolved but assembly succeeded"
        );
    }

    /// Assemble `src` and return the formatted error message (the test
    /// expects assembly to fail).
    fn assemble_error(name: &str, src: &str) -> String {
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
        format!("{res:?}")
    }

    #[test]
    fn top_level_scopes_are_open() {
        // A symbol in another top-level scope resolves without `import`.
        let src = r#"
            scope core
            SLEEP: equ $1000

            scope main
            org $1000
            fdb SLEEP
        "#;
        assert_eq!(
            assemble_bytes("open_scopes", src, 0x1000, 2),
            vec![0x10, 0x00]
        );
    }

    #[test]
    fn ambiguous_top_level_symbol_is_an_error() {
        let src = r#"
            scope a
            FOO: equ 1
            scope b
            FOO: equ 2

            scope main
            org $1000
            fcb FOO
        "#;
        let err = assemble_error("ambiguous_symbol", src);
        assert!(
            err.contains("Ambiguous symbol FOO"),
            "expected ambiguity error, got: {err}"
        );
    }

    #[test]
    fn missing_symbol_hints_at_scope() {
        let src = r#"
            struct proc { time : byte }
            org $1000
            fcb proc
        "#;
        let err = assemble_error("scope_hint", src);
        assert!(
            err.contains("did you mean the scope `proc`"),
            "expected scope hint, got: {err}"
        );
    }

    #[test]
    fn local_symbol_coexists_with_struct_scope() {
        let src = r#"
            struct proc { time : byte }
            proc: equ 5

            org $1000
            fcb proc, proc::time
        "#;
        // bare `proc` is the symbol (5); `proc::time` is the struct field (0).
        assert_eq!(
            assemble_bytes("symbol_scope_coexist", src, 0x1000, 2),
            vec![5, 0]
        );
    }

    #[test]
    fn scoped_access_from_within_top_level_scope() {
        // `scope` directives create top-level (root-child) scopes. From
        // inside one, a scoped path resolves chain-first, falling back to
        // the open top-level scopes: the struct's `proc` is found there.
        let src = r#"
            struct proc { time : byte }

            scope main
            org $1000
            fcb proc::time
        "#;
        assert_eq!(assemble_bytes("scope_open_scoped", src, 0x1000, 1), vec![0]);
    }

    #[test]
    fn scope_directive_shares_name_with_struct_is_an_error() {
        // `scope proc` and `struct proc` both create a top-level scope named
        // `proc` (same parent), so they merge; the struct field and the
        // label then collide in one scope.
        let src = r#"
            struct proc { time : byte }

            scope main
            scope proc
            time: equ 7

            org $1000
            fcb proc::time
        "#;
        let err = assemble_error("scope_struct_name_clash", src);
        assert!(
            err.contains("AlreadyDefined"),
            "expected a redefinition error, got: {err}"
        );
    }

    #[test]
    fn sin_table_emits_rounded_bytes() {
        // The motivating use case for compile-time floats: generate a sin
        // table at assembly time. Floats never reach the target — each
        // entry is rounded to an integer byte.
        let src = r#"
            org $1000
            for i in 0..4 {
                fcb round(sin(i * 2 * 3.14159 / 4) * 127)
            }
        "#;
        // sin(0)=0, sin(pi/2)=1, sin(pi)=0, sin(3pi/2)=-1 -> 0, 127, 0, -127
        assert_eq!(
            assemble_bytes("sin_table", src, 0x1000, 4),
            vec![0x00, 0x7F, 0x00, 0x81]
        );
    }

    #[test]
    fn float_arithmetic_promotes_and_rounds() {
        let src = r#"
            org $1000
            fcb round(1.5 + 1.5), round(10 / 4.0), round(-2.5)
        "#;
        // -2.5 rounds to -3, emitted as byte 0xFD
        assert_eq!(
            assemble_bytes("float_arith", src, 0x1000, 3),
            vec![3, 3, 0xFD]
        );
    }

    #[test]
    fn float_comparison_in_condition() {
        let src = r#"
            org $1000
            if 1.5 > 1.0 {
                fcb 1
            } else {
                fcb 0
            }
        "#;
        assert_eq!(assemble_bytes("float_cond", src, 0x1000, 1), vec![1]);
    }

    #[test]
    fn float_result_without_conversion_is_an_error() {
        let src = r#"
            org $1000
            fcb 1.5
        "#;
        let err = assemble_error("float_unconverted", src);
        assert!(
            err.contains("use round()"),
            "expected a float-conversion error, got: {err}"
        );
    }

    #[test]
    fn float_bitwise_is_an_error() {
        let src = r#"
            org $1000
            fcb 1.5 & 2
        "#;
        let err = assemble_error("float_bitwise", src);
        assert!(
            err.contains("require integer operands"),
            "expected a float bitwise error, got: {err}"
        );
    }

    #[test]
    fn unknown_function_is_an_error() {
        let src = r#"
            org $1000
            fcb foo(1)
        "#;
        let err = assemble_error("unknown_fn", src);
        assert!(
            err.contains("Unknown function foo"),
            "expected an unknown-function error, got: {err}"
        );
    }

    #[test]
    fn assert_passes_and_fails_with_message() {
        let ok_src = r#"
            org $1000
            assert 1 == 1
            fcb 7
        "#;
        assert_eq!(assemble_bytes("assert_ok", ok_src, 0x1000, 1), vec![7]);

        let bad_src = r#"
            org $1000
            assert 1 == 2, "expected one"
            fcb 7
        "#;
        let err = assemble_error("assert_fail", bad_src);
        assert!(
            err.contains("assertion failed: expected one"),
            "expected the assert message in the error, got: {err}"
        );
    }

    #[test]
    fn assert_checks_layout() {
        // Labels get their PC values as the sizer walks in source order,
        // so an assert checks what is already laid out — place it after
        // the things it checks.
        let src = r#"
            org $1000
            PTAB: rmb 5
            TABEND:
            assert TABEND - PTAB == 5
            fcb 9
        "#;
        assert_eq!(assemble_bytes("assert_layout", src, 0x1005, 1), vec![9]);
    }

    #[test]
    fn log_prints_text_and_values_during_sizing() {
        let src = r#"
            org $1000
            log "process table ready"
            log {12345}
            log "table: " {6 * 7} " bytes"
            fcb 1
        "#;
        let path = std::env::temp_dir().join("gazm_log.gazm");
        std::fs::write(&path, src).unwrap();

        let opts = Opts {
            project_file: path.clone(),
            assemble_dir: Some(std::env::temp_dir()),
            verbose: crate::messages::Verbosity::Normal,
            ..Default::default()
        };
        // The CLI normally initialises messaging; do it here so the log
        // lines are routed into the capture.
        crate::messages::init(&opts, None);
        let mut asm = Assembler::new(opts);
        let (res, out, _) = crate::messages::capture(|| asm.assemble());
        let _ = std::fs::remove_file(&path);

        assert!(res.is_ok(), "Assembly failed: {:?}", res.err());
        assert!(
            out.contains("process table ready"),
            "text log missing from output: {out}"
        );
        assert!(
            out.contains("12345"),
            "value log missing from output: {out}"
        );
        assert!(
            out.contains("table: 42 bytes"),
            "interpolated log missing from output: {out}"
        );
    }

    #[test]
    fn assert_message_interpolates_values() {
        let src = r#"
            org $1000
            assert 1 == 2, "got " {1 + 1} ", expected " {2}
            fcb 7
        "#;
        let err = assemble_error("assert_interp", src);
        assert!(
            err.contains("assertion failed: got 2, expected 2"),
            "expected interpolated assert message, got: {err}"
        );
    }

    #[test]
    fn math_builtins() {
        let src = r#"
            org $1000
            fcb abs(-3), min(2, 9), max(2, 9), hi($1234), lo($1234)
            fcb round(sqrt(16.0)), round(floor(3.9)), round(ceil(3.1))
        "#;
        assert_eq!(
            assemble_bytes("math_builtins", src, 0x1000, 8),
            vec![3, 2, 9, 0x12, 0x34, 4, 3, 4]
        );
    }

    #[test]
    fn repeat_index_name_can_be_reused_in_one_scope() {
        let src = r#"
            org $1000
            repeat 2, li {
                fcb li
            }
            repeat 2, li {
                fcb li + 1
            }
        "#;
        // 0,1 then 1,2
        assert_eq!(
            assemble_bytes("repeat_reuse", src, 0x1000, 4),
            vec![0, 1, 1, 2]
        );
    }
}
