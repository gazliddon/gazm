#![deny(unused_imports)]

use unraveler::{many0, match_span as ms, pair, preceded, sep_list0, tuple};


use super::{
    AstNodeKind::{MacroCall, MacroDef},
    TokenKind::Comma,
    *,
};

impl GazmParser
{
    pub fn parse_macro_call(input: TSpan) -> PResult<Node> {
        let (rest, (sp, (label, args))) =
            ms(pair(get_label_string, parse_bracketed(Self::parse_expr_list0)))(input)?;

        let node = from_item_kids_tspan(MacroCall(label), &args, sp);
        Ok((rest, node))
    }

    pub fn parse_macro_def(input: TSpan) -> PResult<Node> {
        let (rest, (sp, (label, args, body))) = ms(preceded(
            CommandKind::Macro,
            tuple((
                get_label_string,
                parse_macrodef_args,
                parse_block(many0(Self::parse_next_source_chunk)),
            )),
        ))(input)?;

        let body: Vec<Node> = body.into_iter().flatten().collect();

        let node = from_item_kids_tspan(MacroDef(label, args.into()), &body, sp);
        Ok((rest, node))
    }
}

pub fn is_parsing_macro_def(i: TSpan) -> bool {
    i.extra().is_parsing_macro_def
}

pub fn set_parsing_macro(i: TSpan, v: bool) -> TSpan {
    i.lift_extra(|e| ParseContext {
        is_parsing_macro_def: v,
        ..e
    })
}

fn parse_macrodef_args(input: TSpan) -> PResult<Vec<String>> {
    parse_bracketed(sep_list0(get_label_string, Comma))(input)
}

// #[allow(dead_code)]
mod test {
    // use crate::{cpu6809::frontend::MC6809, frontend::*, opts::Opts};
    // use thin_vec::ThinVec;
    // use unraveler::all;

    // #[test]
    // fn parse_args() {
    //     let opts = Opts::default();
    //     let text = "(ax,ab,ac)";
    //     println!("Testing macro args: {text}");
    //     let sf = create_source_file(text);
    //     let tokens = to_tokens_no_comment(&sf);
    //     let t: Vec<_> = tokens.iter().map(|t| t.kind).collect();
    //     println!("Toks : {:?}", t);
    //     let input = make_tspan(&tokens, &sf, &opts);
    //     let (_r, m) = super::parse_macrodef_args(input).expect("Can't parse args");
    //     println!("parsed : {:?}", m);
    // }

    // fn as_args<const N: usize>(args: [&str; N]) -> ThinVec<String> {
    //     args.into_iter().map(String::from).collect()
    // }

    // fn as_label(txt: &str) -> Item<MC6809> {
    //     use Item::*;
    //     use LabelDefinition::*;
    //     Label(Text(txt.to_owned()))
    // }

    // #[test]
    // fn test_parse_macro_def() {
    //     use itertools::Itertools;
    //     let text = r#"
// macro MKPROB(process,object_pic,collion_vec,blip) {
    // fdb    MPROB
    // FDB    process,object_pic,collion_vec,blip
// }
    //     "#;

    //     let opts = Opts::default();

    //     let sf = create_source_file(text);
    //     let tokens = to_tokens_no_comment(&sf);
    //     let input = make_tspan(&tokens, &sf, &opts);

    //     let res = super::parse_macro_def(input);

    //     match res {
    //         Err(ref e) => {
    //             println!("len is {}", text.len());
    //             println!("Error is {:?}", e);
    //             let t = &text[e.position.range()];
    //             println!("TEXT: {t}");
    //         }
    //         _ => (),
    //     }

    //     let (_rest, matched) = res.expect("Can't parse macro def!");

    //     let it = NodeIter::new(&matched).map(|n| &n.node.item);

    //     let x = it.collect_vec();
    //     println!("{:?}", x);

    //     use Item::*;

    //     let desired = vec![
    //         MacroDef(
    //             "MKPROB".into(),
    //             as_args(["process", "object_pic", "collion_vec", "blip"]),
    //         ),
    //         Fdb(1),
    //         Expr,
    //         as_label("MPROB"),
    //         Fdb(4),
    //         Expr,
    //         as_label("process"),
    //         Expr,
    //         as_label("object_pic"),
    //         Expr,
    //         as_label("collion_vec"),
    //         Expr,
    //         as_label("blip"),
    //     ];

    //     let it = NodeIter::new(&matched);
    //     let got = it.map(|n| n.node.item.clone()).collect_vec();

    //     assert_eq!(got, desired);
    // }

    // fn text_macro_call(text: &str, _desired: &[Item<MC6809>]) {
    //     println!("Testing macro call : {text}");
    //     let opts = Opts::default();
    //     let sf = create_source_file(text);
    //     let tokens = to_tokens_no_comment(&sf);
    //     let input = make_tspan(&tokens, &sf, &opts);

    //     let (_, matched) = all(Self::parse_macro_call)(input).expect("Doesn't parse");
    //     println!(
    //         "{:?} {:?}",
    //         matched.item,
    //         matched
    //             .children
    //             .iter()
    //             .map(|n| n.item.clone())
    //             .collect::<Vec<_>>()
    //     );

    //     let items = matched
    //         .children
    //         .iter()
    //         .map(|n| n.item.clone())
    //         .collect::<Vec<_>>();
    //     assert_eq!(&items, _desired)
    // }

    // #[test]
    // fn test_macro_call() {
    //     use Item::Num;
    //     use ParsedFrom::Hexadecimal;
    //     let data = [
    //         (
    //             "SLEEP($60, $70)",
    //             vec![Num(0x60, Hexadecimal), Num(0x70, Hexadecimal)],
    //         ),
    //         ("SLEEP()", vec![]),
    //         ("SLEEP((20))", vec![Item::Expr]),
    //     ];

    //     for (text, desired) in data {
    //         text_macro_call(text, &desired)
    //     }
    // }
}
