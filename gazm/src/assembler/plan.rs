#![forbid(unused_imports)]

use crate::{frontend::AstNodeKind, gazmsymbols::SymbolScopeId, semantic::AstNodeId};

/// One statement of the assembled program, in final walk order.
///
/// The sizer walks the AST once and records, per statement, everything the
/// compiler needs to emit bytes: the scope it was assembled in, the AST node
/// it came from (for children, source positions and diagnostics), the *final*
/// node kind (fixups such as `Org` -> `SetPc` are applied here, at plan-build
/// time, instead of being carried as a side table), and the PC the sizer
/// computed for it.
///
/// Control flow (macro expansion, `repeat` iteration) is already expanded:
/// the plan is linear, and each body statement appears once per iteration.
/// `bindings` carries the loop-index symbol values that must be in effect
/// while the statement is processed — one entry per enclosing `repeat`, so
/// nested loops stay correct without the compiler re-deriving anything.
#[derive(Debug, Clone)]
pub struct PlanEntry {
    /// Scope this statement was assembled in.
    pub scope_id: u64,
    /// The AST node this statement came from.
    pub node_id: AstNodeId,
    /// The final node kind, with fixups already applied.
    pub kind: AstNodeKind,
    /// The PC (logical) the sizer computed for the start of this statement.
    /// The compiler asserts the binary's write address matches before
    /// processing, so any drift between sizing and emission is caught
    /// immediately instead of silently corrupting the output.
    pub pc: usize,
    /// Symbol values to set before processing, in application order.
    /// Empty for statements outside any `repeat`.
    pub bindings: Vec<(SymbolScopeId, i64)>,
}
