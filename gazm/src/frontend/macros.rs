#![deny(unused_imports)]
use crate::item::{Node, Item::MacroCall };

use unraveler::{preceded, sep_list, tuple, Parser};

use super::{
    concat, get_text, match_span as ms, parse_block, parse_bracketed, parse_non_scoped_label,
    CommandKind, FrontEndError,
    IdentifierKind::Label,
    PResult, TSpan,
    TokenKind::{Comma, Identifier},
};

pub fn parse_macro_def<'a, E, P: Parser<TSpan<'a>, Node, FrontEndError>>(
    input: TSpan<'a>,
    p: P,
) -> PResult<Node> {
    let (rest, (sp, (label, args, body))) = ms(preceded(
        CommandKind::Macro,
        tuple((
            Identifier(Label),
            parse_bracketed(sep_list(parse_non_scoped_label, Comma)),
            parse_block(p),
        )),
    ))(input)?;

    let node = Node::from_item_kids_tspan(MacroCall(get_text(label)), &concat((body, args)), sp);
    Ok((rest, node))
}

#[cfg(test)]
mod test {}

/*
macro X() {

}
*/
