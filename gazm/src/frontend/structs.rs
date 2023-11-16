#![deny(unused_imports)]

// use super::*;
use super::{
    get_text, parse_block, parse_expr, CommandKind, IdentifierKind, PResult, TSpan,
    TokenKind::{Colon, Comma, Identifier},
    item::{Item, Node, StructMemberType},
    parse_label,
};

use unraveler::{map, match_span as ms, opt, pair, preceded, sep_list0, succeeded, tag, tuple};

use CommandKind::Struct;
use IdentifierKind::Label;

pub fn parse_struct_arg_type(input: TSpan) -> PResult<(TSpan, StructMemberType)> {
    let as_arg_type =
        |i| -> StructMemberType { get_text(i).to_string().parse::<StructMemberType>().unwrap() };
    ms(map(tag(Identifier(Label)), as_arg_type))(input)
}
pub fn parse_array_def(input: TSpan) -> PResult<Node> {
    use super::TokenKind::*;
    let (rest, (_, matched, _)) =
        tuple((OpenSquareBracket, parse_expr, CloseSquareBracket))(input)?;

    Ok((rest, matched))
}

pub fn parse_struct_entry(input: TSpan) -> PResult<Node> {
    let (rest, (name, _, (entry_span, entry_type), (array_def_sp, array))) = tuple((
        parse_label,
        Colon,
        parse_struct_arg_type,
        ms(opt(parse_array_def)),
    ))(input)?;

    let size = entry_type.to_size_item();

    let kids = [
        array.unwrap_or(Node::from_num_tspan(1, array_def_sp)),
        Node::from_item_tspan(Item::Mul, array_def_sp),
        Node::from_item_tspan(size, entry_span),
    ];

    let expr = Node::from_item_kids_tspan(Item::Expr, &kids, entry_span);
    let node = Node::from_item_kid_tspan(Item::StructEntry(name.to_string()), expr, input);

    Ok((rest, node))
}

pub fn parse_struct(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (label, entries))) = ms(pair(
        preceded(Struct, Identifier(Label)),
        parse_block(succeeded(sep_list0(parse_struct_entry, Comma), opt(Comma))),
    ))(input)?;

    let text = get_text(label);

    let node = Node::from_item_kids_tspan(Item::StructDef(text), &entries, sp);

    Ok((rest, node))
}

// Always compile so I get IDE errors
#[allow(unused_imports)]
mod test {
    use super::*;
    use crate::frontend::*;

    // Only include dev deps if test cfg
    #[cfg(test)]
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

    use LabelDefinition::Text;
    use grl_sources::SourceFile;
    use thin_vec::thin_vec;
    use unraveler::Collection;

    #[test]
    fn test_struct() {
        use Item::*;

        let text = r#"
        struct my_struct 
        { test : byte, spanner : byte,
        book  : word
        }"#;

        let sf = create_source_file(text);
        let tokens = to_tokens_no_comment(&sf);

        let ts: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        println!("{:?}", ts);

        let span = make_tspan(&tokens, &sf);

        let (rest, matched) = parse_struct(span).unwrap();

        let sub_kinds: Vec<_> = matched.children.iter().map(|t| &t.item).collect();
        println!("Kids: {:?}", sub_kinds);

        let items = get_items(&matched);
        let desired = (
            StructDef("my_struct".to_owned()),
            thin_vec![
                StructEntry("test".into()),
                StructEntry("spanner".into()),
                StructEntry("book".into()),
            ],
        );

        assert_eq!(items, desired);

        assert!(rest.is_empty());
    }
}
