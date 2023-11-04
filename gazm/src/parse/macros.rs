#![deny(unused_imports)]
use crate::{
    error::IResult,
    item::Node,
};

use super::{
    labels::get_just_label,
    util::{get_block, sep_list0, wrapped_chars, ws},
    locate::{matched_span, span_to_pos, Span},
};

use grl_sources::Position;

use nom::{
    bytes::complete::tag,
    character::complete::multispace1,
    multi::separated_list0,
    sequence::separated_pair,
};

////////////////////////////////////////////////////////////////////////////////
// Macros
/// Gets the strings for a macro definition
/// returns (name, array of args, macro body)
pub fn get_macro_def(input: Span<'_>) -> IResult<(Span, Vec<Span>, Span)> {
    let rest = input;
    let (rest, (_, name)) = ws(separated_pair(tag("macro"), multispace1, get_just_label))(rest)?;
    let (rest, params) = wrapped_chars('(', sep_list0(get_just_label), ')')(rest)?;
    let (rest, body) = get_block(rest)?;
    Ok((rest, (name, params, body)))
}

pub fn get_scope_block(input: Span<'_>) -> IResult<(Span, Span)> {
    let rest = input;
    let (rest, (_, name)) = ws(separated_pair(tag("scope2"), multispace1, get_just_label))(rest)?;
    let (rest, body) = get_block(rest)?;
    Ok((rest, (name, body)))
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroCall {
    pub name: Position,
    pub args: Vec<Position>,
}

pub fn parse_macro_call(input: Span) -> IResult<Node> {
    use crate::item::Item;
    use crate::parse::expr::parse_expr;
    let sep = ws(tag(","));

    let rest = input;
    let (rest, name) = get_just_label(rest)?;
    let (rest, args) = ws(wrapped_chars(
        '(',
        ws(separated_list0(sep, parse_expr)),
        ')',
    ))(rest)?;

    let matched_span = span_to_pos(matched_span(input, rest));
    let node = Node::new_with_children(Item::MacroCall(name.to_string()), &args, matched_span);

    Ok((rest, node))
}

#[allow(unused_imports)]
#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    #[test]
    fn test_scopes() {
        let body = "scope2 background {a} aka kj akj a";
        let sp = Span::new_extra(body, grl_sources::AsmSource::FromStr);

        if let Ok((rest, (name, body))) = get_scope_block(sp) {
            assert_eq!(&name.to_string(), "background");
            assert_eq!(&body.to_string(), "a");
            assert_eq!(&rest.to_string(), "aka kj akj a");
        } else {
            assert!(false)
        }
    }
}
