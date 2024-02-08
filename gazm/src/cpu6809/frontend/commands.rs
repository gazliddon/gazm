use crate::frontend::{
    from_item_kid_tspan, parse_expr, AstNodeKind, CommandKind, Node, PResult, TSpan,
};

use super::NodeKind6809;

use unraveler::{cut, match_span as ms, preceded};

pub fn parse_set_dp(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(preceded(CommandKind::SetDp, cut(parse_expr)))(input)?;
    let node = from_item_kid_tspan(
        AstNodeKind::TargetSpecific(NodeKind6809::SetDp.into()),
        matched,
        sp,
    );
    Ok((rest, node))
}

pub fn parse_commands(input: TSpan) -> PResult<Node> {
    parse_set_dp(input)
}
