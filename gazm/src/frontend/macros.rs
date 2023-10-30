#![deny(unused_imports)]
use crate::item::{
    Item::{MacroCall, MacroDef},
    Node,
};

use unraveler::{pair, preceded, sep_list0, tuple};

use super::{
    get_text, parse_block, parse_bracketed, parse_expr_list0, CommandKind,
    IdentifierKind::Label,
    PResult, TSpan,
    TokenKind::{self, Comma, Identifier},
    OriginalSource,
    parse_span,
};
use unraveler::match_span as ms;

pub fn parse_macro_call(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (label, args))) =
        ms(pair(Identifier(Label), parse_bracketed(parse_expr_list0)))(input)?;
    let node = Node::from_item_kids_tspan(MacroCall(get_text(label)), &args, sp);
    Ok((rest, node))
}

pub fn is_parsing_macro_def(i: TSpan) -> bool {
    i.extra().is_parsing_macro_def
}

pub fn set_parsing_macro(i: TSpan, v: bool) -> TSpan {
    i.lift_extra(|e| OriginalSource { is_parsing_macro_def: v, ..e })
}

pub fn parse_macro_def(input: TSpan) -> PResult<Node> {
    if is_parsing_macro_def(input) {
        panic!("Need an error message for trying to parse a mdef in a mdef")
    } else {
    parse_macro_def_with_body(set_parsing_macro(input, true), parse_span)
        .map(|(r, m)| (set_parsing_macro(r, false), m))
    }
}

pub fn parse_macro_def_with_body<P>(input: TSpan, p: P) -> PResult<Node>
where
    P: Fn(TSpan) -> PResult<Node> + Copy,
{
    let (rest, (sp, (label, args, body))) = ms(preceded(
        CommandKind::Macro,
        tuple((
            Identifier(Label),
            parse_bracketed(sep_list0(TokenKind::Identifier(Label), Comma)),
            parse_block(p),
        )),
    ))(input)?;

    let v: thin_vec::ThinVec<_> = args.into_iter().map(|sp| get_text(sp).to_owned()).collect();

    let node = Node::from_item_kid_tspan(MacroDef(get_text(label), v), body, sp);
    Ok((rest, node))
}

#[allow(unused_imports)]
#[cfg(test)]
mod test {
    use crate::{
        cli::parse_command_line,
        frontend::*,
        item::{
            Item::{self, *},
            LabelDefinition, Node,
            ParsedFrom::*,
        },
        item6809::MC6809,
    };
    use grl_sources::SourceFile;
    use pretty_assertions::{assert_eq, assert_ne};
    use thin_vec::ThinVec;
    use unraveler::{all, cut, Collection, Parser};

    fn text_macro_call(text: &str, _desired: &[Item]) {
        println!("Testing macro call : {text}");
        let sf = create_source_file(text);
        let tokens = to_tokens(&sf);
        let input = make_tspan(&tokens, &sf);

        let (_, matched) = all(parse_macro_call)(input).expect("Doesn't parse");
        println!(
            "{:?} {:?}",
            matched.item,
            matched
                .children
                .iter()
                .map(|n| n.item.clone())
                .collect::<Vec<_>>()
        );
        let items = matched
            .children
            .iter()
            .map(|n| n.item.clone())
            .collect::<Vec<_>>();
        assert_eq!(&items, _desired)
    }

    #[test]
    fn test_macro_call() {
        let data = [
            ("SLEEP($60, $70)", vec![Num(0x60, Hex), Num(0x70, Hex)]),
            ("SLEEP()", vec![]),
            ("SLEEP((20))", vec![Expr]),
        ];

        for (text, desired) in data {
            text_macro_call(text, &desired)
        }
    }
}

/*
macro X() {

}
*/
