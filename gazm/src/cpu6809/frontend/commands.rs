use crate::frontend::{CommandKind, GazmParser, Item, PResult, TSpan};

use super::super::assembler::Assembler6809;
use super::MC6809;

type Node = crate::frontend::Node<MC6809>;

use unraveler::{cut, preceded, match_span as ms};

impl GazmParser<Assembler6809> {
    pub fn parse_set_dp(input: TSpan) -> PResult<Node> {
        let (rest, (sp, matched)) = ms(preceded(CommandKind::SetDp, cut(Self::parse_expr)))(input)?;
        let node = Self::from_item_kid_tspan(Item::CpuSpecific(MC6809::SetDp), matched, sp);
        Ok((rest, node))
    }

    pub fn parse_commands(input: TSpan) -> PResult<Node> {
        Self::parse_set_dp(input)
    }
}
