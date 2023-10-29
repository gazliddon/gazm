#![deny(unused_imports)]
use super::*;

use crate::item::{Item, Node};


use unraveler::match_span as ms;
use unraveler::{alt,many0,  pair, preceded, sep_list,};
use TokenKind::{Identifier, Colon };
use super::parse_line;

pub fn struct_entry(input: TSpan) -> PResult<[Node;2]> { 
    let (rest,(a,b)) = pair(parse_non_scoped_label, alt((parse_rmb, parse_rmd)))(input)?;
    Ok((rest,[a,b]))
}

pub fn struct_entries(input: TSpan) -> PResult<Vec<[Node;2]>> {
    let (rest,matched) = many0(parse_line(sep_list(struct_entry, Colon)))(input)?;
    let matched = matched.into_iter().flatten().collect();
    Ok((rest,matched))

}

pub fn parse_struct(input: TSpan) -> PResult<Node> {
    use IdentifierKind::Label;
    use CommandKind::Struct;

    let (rest, (sp, (label, list)))  = ms(pair(
        preceded(Struct, Identifier(Label)),
        parse_block(struct_entries),
    ))(input)?;

    let text = get_text(label);

    let list: Vec<_> = list
        .into_iter()
        .flatten()
        .collect();
    let node = Node::from_item_kids_tspan(Item::StructDef(text), &list, sp);
    Ok((rest, node))
}

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use super::*;
    use crate::item::LabelDefinition::Text;
    use pretty_assertions::assert_eq;
    use thin_vec::thin_vec;
    use unraveler::Collection;

    #[test]
    fn test_struct() {
        use Item::*;

        let text = r#"
        struct my_struct { test rmb 10 : spanner rmb 20 
        }"#;

        let sf = create_source_file(text);
        let tokens = to_tokens(&sf);

        let ts: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        println!("{:?}", ts);

        let span = make_tspan(&tokens, &sf);

        let (rest, matched) = parse_struct(span).unwrap();

        let sub_kinds : Vec<_> = matched.children.iter().map(|t| &t.item).collect();
        println!("Kids: {:?}",sub_kinds);

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

/*
struct Structy { test    rmb 10, spanner rmb 20 }
*/
