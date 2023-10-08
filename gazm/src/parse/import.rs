#![forbid(unused_imports)]
use super::{
    labels::get_scoped_label,
    locate::{span_to_pos, Span},
};

use crate::{
    error::IResult,
    item::{Item, Node},
    gazmsymbols::ScopedName,
};

fn parse_braced_imports(_input: Span) -> IResult<Node> { 
    panic!()
}

pub fn parse_import_arg(input: Span) -> IResult<Node> {
    let (rest, matched) = get_scoped_label(input)?;
    let scoped_name = ScopedName::new(&matched);

    assert!(scoped_name.is_abs());

    let raw_imports = vec![matched];

    let imports: Vec<_> = raw_imports
        .into_iter()
        .map(|i| {
            let item = Item::Label(crate::item::LabelDefinition::TextScoped(i.to_string()));
            Node::from_item_span(item, i)
        })
        .collect();
    let node = Node::new_with_children(Item::Import, &imports, span_to_pos(matched));
    Ok((rest, node))
}

