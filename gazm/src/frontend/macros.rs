#![deny(unused_imports)]

use unraveler::{match_span as ms, pair, preceded, sep_list0, tuple};


use super::{
    IdentifierKind::Label,
    Item::{MacroCall, MacroDef},
    TokenKind::{Comma, Identifier},
    *,
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

fn parse_macrodef_args(input: TSpan) -> PResult<Vec<String>> {
    parse_bracketed(sep_list0(get_label_text, Comma))(input)
}

pub fn parse_macro_def(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (label, args, body))) = ms(preceded(
        CommandKind::Macro,
        tuple((
            Identifier(Label),
            parse_macrodef_args,
            parse_block(parse_source_chunks),
        )),
    ))(input)?;

    let node = Node::from_item_kids_tspan(MacroDef(get_text(label), args.into()), &body, sp);
    Ok((rest, node))
}

#[allow(unused_imports)]
mod test {
    use super::*;
    use crate::{cli::parse_command_line, frontend::*};

    use Item::Num;
    use ParsedFrom::Hex;

    use crate::opts::Opts;
    use grl_eval::ExprItem::Expr;
    use grl_sources::{grl_utils::Stack, SourceFile};
    use itertools::Itertools;
    use termimad::crossterm::style::Stylize;
    use thin_vec::ThinVec;
    use tower_lsp::lsp_types::{ClientInfo, CompletionItemCapability, DeleteFilesParams};
    use unraveler::{all, cut, Collection, Parser};

    ////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn parse_args() {
        let opts = Opts::default();
        let text = "(ax,ab,ac)";
        println!("Testing macro args: {text}");
        let sf = create_source_file(text);
        let tokens = to_tokens_no_comment(&sf);
        let t: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        println!("Toks : {:?}", t);
        let input = make_tspan(&tokens, &sf, &opts);
        let (_r, m) = super::parse_macrodef_args(input).expect("Can't parse args");
        println!("parsed : {:?}", m);
    }

    fn as_args<const N: usize>(args: [&str; N]) -> ThinVec<String> {
        args.into_iter().map(String::from).collect()
    }

    fn as_label(txt: &str) -> Item {
        use Item::*;
        use LabelDefinition::*;
        Label(Text(txt.to_owned()))
    }

    #[test]
    fn test_parse_macro_def() {
        use thin_vec::thin_vec;
        let text = r#"
macro MKPROB(process,object_pic,collion_vec,blip) {
    fdb    MPROB
    FDB    process,object_pic,collion_vec,blip
}
        "#;

        let opts = Opts::default();

        let sf = create_source_file(text);
        let tokens = to_tokens_no_comment(&sf);
        let input = make_tspan(&tokens, &sf, &opts);

        let res = super::parse_macro_def(input);

        match res {
            Err(ref e) => {
                println!("len is {}", text.len());
                println!("Error is {:?}", e);
                let t = &text[e.position.range()];
                println!("TEXT: {t}");
            }
            _ => (),
        }

        let (_rest, matched) = res.expect("Can't parse macro def!");

        let it = NodeIter::new(&matched).map(|n| &n.node.item);

        let x = it.collect_vec();
        println!("{:?}", x);

        use Item::*;

        let desired = vec![
            MacroDef(
                "MKPROB".into(),
                as_args(["process", "object_pic", "collion_vec", "blip"]),
            ),
            Fdb(1),
            Expr,
            as_label("MPROB"),
            Fdb(4),
            Expr,
            as_label("process"),
            Expr,
            as_label("object_pic"),
            Expr,
            as_label("collion_vec"),
            Expr,
            as_label("blip"),
        ];

        let it = NodeIter::new(&matched);
        let got = it.map(|n| n.node.item.clone()).collect_vec();

        assert_eq!(got, desired);
    }

    fn text_macro_call(text: &str, _desired: &[Item]) {
        println!("Testing macro call : {text}");
        let opts = Opts::default();
        let sf = create_source_file(text);
        let tokens = to_tokens_no_comment(&sf);
        let input = make_tspan(&tokens, &sf, &opts);

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
            ("SLEEP((20))", vec![Item::Expr]),
        ];

        for (text, desired) in data {
            text_macro_call(text, &desired)
        }
    }
}
