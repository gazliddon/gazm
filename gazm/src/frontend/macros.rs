#![deny(unused_imports)]
use crate::item::{Node, Item::{ MacroCall, MacroDef } };

use unraveler::{preceded, sep_list, tuple, Parser};

use super::{
    get_text, match_span as ms, parse_block, parse_bracketed, parse_non_scoped_label,
    CommandKind, FrontEndError,
    IdentifierKind::Label,
    PResult, TSpan,
    TokenKind::{Comma, Identifier, self},
};


pub fn parse_macro_call<'a, E, P: Parser<TSpan<'a>, Node, FrontEndError>>(
    input: TSpan<'a>,
    p: P,
) -> PResult<Node> {
    let (rest, (sp, (label, args, _body))) = ms(
        tuple((
            Identifier(Label),
            parse_bracketed(sep_list( parse_non_scoped_label, Comma )),p),
        ),
    )(input)?;

    let node = Node::from_item_kids_tspan(MacroCall(get_text(label)), &args, sp);

    Ok((rest, node))
}

pub fn parse_macro_def<'a, E, P: Parser<TSpan<'a>, Node, FrontEndError>>(
    input: TSpan<'a>,
    p: P,
) -> PResult<Node> {

    let (rest, (sp, (label, args, body))) = ms(preceded(
        CommandKind::Macro,
        tuple((
            Identifier(Label),
            parse_bracketed(sep_list(TokenKind::Identifier(Label), Comma)),
            parse_block(p),
        )),
    ))(input)?;

    let v : thin_vec::ThinVec<_> = args.as_slice().into_iter().map(|sp| get_text(*sp).to_owned()).collect();

    let node = Node::from_item_kid_tspan(MacroDef(get_text(label),v), body,sp);
    Ok((rest, node))
}

#[cfg(test)]
mod test {}

/*
macro X() {

}
*/
