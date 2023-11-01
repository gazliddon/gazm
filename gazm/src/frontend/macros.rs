#![deny(unused_imports)]

use crate::item::{
    Item::{MacroCall, MacroDef2},
    Node,
};

use unraveler::{match_span as ms, pair, preceded, sep_list0, tuple};

use super::{
    get_text, parse_block, parse_bracketed, parse_expr_list0, parse_non_scoped_label, parse_span,
    CommandKind,
    IdentifierKind::Label,
    OriginalSource, PResult, TSpan,
    TokenKind::{Comma, Identifier},
};

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
    i.lift_extra(|e| OriginalSource {
        is_parsing_macro_def: v,
        ..e
    })
}

fn parse_macro_args(input: TSpan) -> PResult<Vec<Node>> {
    parse_bracketed(sep_list0(parse_non_scoped_label, Comma))(input)
}

pub fn parse_macro_def(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (label, _args, body))) = ms(preceded(
        CommandKind::Macro,
        tuple((Identifier(Label), parse_macro_args, parse_block(parse_span))),
    ))(input)?;

    let node = Node::from_item_kid_tspan(MacroDef2(get_text(label)), body, sp);
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

    #[test]
    fn parse_args() {
        let text = "(ax,ab,ac)";
        println!("Testing macro args: {text}");
        let sf = create_source_file(text);
        let tokens = to_tokens_no_comment(&sf);
        let t: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        println!("Toks : {:?}", t);
        let input = make_tspan(&tokens, &sf);
        let (_r, m) = super::parse_macro_args(input).expect("Can't parse args");

        let t: Vec<_> = m.into_iter().map(|n| n.item).collect();
        println!("parsed : {:?}", t);
    }

    #[test]
    fn test_parse_macro_def() {
        let text = r#"macro label(ax,bx,cx) { }"#;

        println!("Testing macro def: {text}");
        let sf = create_source_file(text);
        let tokens = to_tokens_no_comment(&sf);

        let t: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        println!("Toks : {:?}", t);

        let input = make_tspan(&tokens, &sf);
        let (_rest, matched) = super::parse_macro_def(input).expect("Can't parse macro def");

        let t: Vec<_> = matched.children.iter().map(|n| &n.item).collect();
        println!("Node : {:?} {:?}", &matched.item, t);
        panic!()
    }

    fn text_macro_call(text: &str, _desired: &[Item]) {
        println!("Testing macro call : {text}");
        let sf = create_source_file(text);
        let tokens = to_tokens_no_comment(&sf);
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

