//! Shared operand-parsing scaffolding for the CPU backends.
//!
//! The 6809/6800 frontends parse an opcode's operand into a parse-level
//! addressing mode, then resolve the instruction row through the table.
//! The pieces below are identical across those backends; each supplies
//! its own mode type (via `Into<AstNodeKind>`) and its resolution
//! closure. The genuinely CPU-specific grammar — indexed postbytes,
//! register lists — stays in the backend as the `parse_arg` hook.

use super::{
    err_fatal, from_item_tspan, parse_expr, AstNodeKind, FrontEndError, FrontEndErrorKind, Node,
    PResult, TSpan, TokenKind,
};
use unraveler::{match_span as ms, preceded, Collection};

/// Parse `[prefix] <expr>` into an operand node carrying `amode`.
/// The immediates (`#expr`), forced direct (`<expr`), forced extended
/// (`>expr`) and plain expression forms all share this shape.
pub fn parse_prefixed_operand<'a, A>(
    input: TSpan<'a>,
    prefix: Option<TokenKind>,
    amode: A,
) -> PResult<'a, Node>
where
    A: Into<AstNodeKind>,
{
    let (rest, (sp, matched)) = match prefix {
        Some(prefix) => ms(preceded(prefix, parse_expr))(input)?,
        None => ms(parse_expr)(input)?,
    };
    let node = from_item_tspan(amode, sp).with_child(matched);
    Ok((rest, node))
}

/// An opcode with no operand: resolve the inherent row, or error with
/// `unsupported` when the mnemonic has no inherent form.
pub fn parse_inherent<'a, R, E>(
    rest: TSpan<'a>,
    sp: TSpan<'a>,
    get_inherent: impl FnOnce() -> Option<R>,
    make_item: impl FnOnce(R) -> AstNodeKind,
    unsupported: E,
) -> PResult<'a, Node>
where
    E: Into<FrontEndErrorKind>,
{
    match get_inherent() {
        Some(ins) => {
            let oc = make_item(ins);
            Ok((rest, from_item_tspan(oc, sp)))
        }
        None => err_fatal(sp, unsupported),
    }
}

/// Parse an operand, resolve it to an instruction row, and build the
/// `OpCode` node. `parse_arg` handles the backend's operand grammar
/// (immediate / indexed / register forms); `resolve` extracts the
/// parse-level mode from the argument node and resolves the row id,
/// erroring with the backend's own kinds (via `fatal(sp, kind)`);
/// `make_item` builds the per-CPU `OpCode` kind.
pub fn parse_opcode_operand<'a, A, R>(
    rest: TSpan<'a>,
    sp: TSpan<'a>,
    parse_arg: impl FnOnce(TSpan<'a>) -> PResult<'a, Node>,
    resolve: impl FnOnce(&Node) -> Result<(A, R), FrontEndError>,
    make_item: impl FnOnce(A, R) -> AstNodeKind,
) -> PResult<'a, Node> {
    let (rest, arg) = parse_arg(rest)?;
    let (amode, ins) = resolve(&arg)?;
    let item = make_item(amode, ins);
    let node = from_item_tspan(item, sp).take_others_children(arg);
    Ok((rest, node))
}
