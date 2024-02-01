use crate::frontend::{CommandKind, AstNodeKind, PResult, TSpan};

use super::super::{from_item_kid_tspan, parse_expr, Node};

use super::NodeKind6809;

use unraveler::{cut, match_span as ms, preceded};

pub fn parse_set_dp(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(preceded(CommandKind::SetDp, cut(parse_expr)))(input)?;
    let node = from_item_kid_tspan(AstNodeKind::CpuSpecific(NodeKind6809::SetDp), matched, sp);
    Ok((rest, node))
}

pub fn parse_commands(input: TSpan) -> PResult<Node> {
    parse_set_dp(input)
}
