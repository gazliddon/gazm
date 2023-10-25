#![deny(unused_imports)]
use crate::item::{
    Item::{MacroCall, MacroDef},
    Node,
};

use unraveler::{pair, preceded, sep_list0, tuple, Parser};

use super::{
    get_text, match_span as ms, parse_block, parse_bracketed, parse_expr_list0, CommandKind,
    FrontEndError,
    IdentifierKind::Label,
    PResult, TSpan,
    TokenKind::{self, Comma, Identifier},
};


pub fn parse_macro_call(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (label, args))) = ms(pair(
        Identifier(Label),
        parse_bracketed(parse_expr_list0),
    ))(input)?;
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
            parse_bracketed(sep_list0(TokenKind::Identifier(Label), Comma)),
            parse_block(p),
        )),
    ))(input)?;

    let v: thin_vec::ThinVec<_> = args
        .as_slice()
        .into_iter()
        .map(|sp| get_text(*sp).to_owned())
        .collect();

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
    use unraveler::{Collection, Parser,cut,all};

    fn text_macro_call(text : &str, _desired: &[Item]) {
        println!("Testing macro call : {text}");
        let sf = create_source_file(text);
        let tokens = to_tokens(&sf);
        let input = make_tspan(&tokens, &sf);

        let (_,matched) = all(parse_macro_call)(input).expect("Doesn't parse");
        println!("{:?} {:?}", matched.item, matched.children.iter().map(|n| n.item.clone()).collect::<Vec<_>>());
        let items = matched.children.iter().map(|n| n.item.clone()).collect::<Vec<_>>();
        assert_eq!(&items,_desired)
    }

    #[test]
    fn test_macro_call() {
        let data = [
            ( "SLEEP($60, $70)", vec![Num(0x60,Hex),Num(0x70,Hex)] ),
            ( "SLEEP()", vec![] ),
            ( "SLEEP((20))", vec![Expr] ),
        ];

        for (text,desired) in data {
            text_macro_call(text, &desired)
        }
    }
}

/*
macro X() {

}
*/
