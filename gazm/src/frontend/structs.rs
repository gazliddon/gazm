#![deny(unused_imports)]
// use super::*;
use super::{
    get_text, parse_block, parse_line, parse_non_scoped_label, parse_rmb, parse_rmd, CommandKind,
    IdentifierKind, PResult, TSpan,
    TokenKind::{Colon, Identifier},
};

use crate::item::{Item, Node};
use unraveler::match_span as ms;
use unraveler::{alt, many0, pair, preceded, sep_list};
use CommandKind::Struct;
use IdentifierKind::Label;

pub fn struct_entry(input: TSpan) -> PResult<[Node; 2]> {
    let (rest, (a, b)) = pair(parse_non_scoped_label, alt((parse_rmb, parse_rmd)))(input)?;
    Ok((rest, [a, b]))
}

pub fn struct_entries(input: TSpan) -> PResult<Vec<[Node; 2]>> {
    let (rest, matched) = parse_line(many0(sep_list(struct_entry, Colon)))(input)?;
    let matched = matched.into_iter().flatten().collect();
    Ok((rest, matched))
}

pub fn parse_struct(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (label, list))) = ms(pair(
        preceded(Struct, Identifier(Label)),
        parse_block(struct_entries),
    ))(input)?;

    let text = get_text(label);

    let list: Vec<_> = list.into_iter().flatten().collect();
    let node = Node::from_item_kids_tspan(Item::StructDef(text), &list, sp);
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

    use crate::item::LabelDefinition::Text;
    use grl_sources::SourceFile;
    use thin_vec::thin_vec;
    use unraveler::Collection;

    #[test]
    fn test_struct() {
        use Item::*;

        let text = r#"
        struct my_struct 
        { test rmb 10 : spanner rmb 20 }"#;

        let sf = create_source_file(text);
        let tokens = to_tokens(&sf);

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
                Label(Text("test".into())),
                Rmb,
                Label(Text("spanner".into())),
                Rmb
            ],
        );

        assert_eq!(items, desired);
        assert!(rest.is_empty());
    }
}
