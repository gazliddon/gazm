use super::*;

use crate::{
    item::{Item, LabelDefinition, Node},
    parse::{macros::MacroCall, util::sep_list1},
};
use grl_sources::Position;
use thin_vec::thin_vec;

use super::match_span as ms;

use unraveler::{
    all, alt, cut, is_a, many0, many1, many_until, not, opt, pair, preceded, sep_list, sep_pair,
    succeeded, tuple, until, wrapped_cut, Collection, ParseError, ParseErrorKind, Parser, Severity,
};

pub fn parse_macro_def<'a, P: Parser<TSpan<'a>, Node, MyError>>(
    input: TSpan<'a>,
    p: P,
) -> PResult<Node> {
    use {
        IdentifierKind::Label,
        TokenKind::{Comma, Identifier}
    };

    let (rest, (sp, (label, args, body))) = ms(preceded(
        CommandKind::Macro,
        tuple((
            Identifier(Label),
            parse_bracketed(sep_list(parse_non_scoped_label, Comma)),
            parse_block(p),
        )),
    ))(input)?;

    let node = Node::new_with_children(
        Item::MacroCall(get_text(label)),
        &concat((body, args)),
        to_pos(sp),
    );
    Ok((rest, node))
}

#[cfg(test)]
mod test {}

/*
macro X() {

}
*/
