use super::*;

use crate::item::{Item, LabelDefinition, Node};
use grl_sources::Position;
use thin_vec::thin_vec;

use super::match_span as ms;

use unraveler::{
    all, alt, cut, is_a, many0, many1, many_until, not, opt, pair, preceded, sep_list, sep_pair,
    succeeded, tuple, until, wrapped_cut, Collection, ParseError, ParseErrorKind, Parser, Severity,
};

pub fn struct_entry(input: TSpan) -> PResult<(Node, Node)> {
    pair(parse_non_scoped_label, alt((parse_rmb, parse_rmd)))(input)
}

pub fn parse_struct(input: TSpan) -> PResult<Node> {
    use {
        IdentifierKind::Label,
        TokenKind::{Comma, Identifier},
    };

    let body = succeeded(sep_list(struct_entry, Comma), opt(Comma));

    let parsed = ms(pair(
        preceded(CommandKind::Struct, Identifier(Label)),
        parse_block(body),
    ))(input);

    let (rest, (sp, (label, list))) = parsed?;

    let text = get_text(label);
    let list: Vec<_> = list.into_iter().map(|(a, b)| [a, b]).flatten().collect();
    let node = Node::new_with_children(Item::StructDef(text), &list, to_pos(sp));
    Ok((rest, node))
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use IdentifierKind::Label;
    use LabelDefinition::*;

    #[test]
    fn test_struct() {
        use Item::*;

        let text = "struct my_struct { test rmb 10, spanner rmb 20 }";
        let sf = create_source_file(text);
        let tokes = to_tokens(&sf);

        let ts: Vec<_> = tokes.iter().map(|t| t.kind).collect();
        println!("{:?}", ts);

        let span = TSpan::from_slice(&tokes,Default::default());

        let (rest, matched) = parse_struct(span).unwrap();

        let items = get_items(&matched);
        let desired = (
            StructDef("my_struct".to_owned()),
            thin_vec![
                Label(Text("test".into())),
                Rmb,
                Label(Text("spanner".into())),
                Rmb
            ],
        );

        assert_eq!(items, desired);
        assert!(rest.is_empty())
    }
}

/*
struct Structy { test    rmb 10, spanner rmb 20 }
*/
