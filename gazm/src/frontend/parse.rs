// #![deny(unused_imports)]
use super::{
    parse_command, parse_equate, parse_label, parse_line, parse_macro_call, parse_macro_def,
    parse_multi_opcode_vec, parse_struct, utils::mk_pc_equate, Item, Node, PResult, TSpan,
};

use itertools::Itertools;
use thin_vec::ThinVec;
use unraveler::{all, alt, many0, map, Collection};

struct NodeCollector<'a> {
    nodes: ThinVec<Node>,
    span: TSpan<'a>,
}

impl<'a> NodeCollector<'a> {
    pub fn new(sp: TSpan<'a>) -> Self {
        Self {
            nodes: thin_vec::ThinVec::with_capacity(4096),
            span: sp,
        }
    }

    pub fn add(&mut self, n: Node) {
        if n.item == Item::Block {
            for i in n.children {
                self.add(i)
            }
        } else {
            self.nodes.push(n)
        }
    }

    pub fn add_vec(&mut self, nodes: Vec<Node>) {
        self.nodes.reserve(nodes.len());
        self.nodes.extend(nodes)
    }

    pub fn into_block(self) -> Node {
        Node::block(self.nodes, self.span)
    }
}

pub fn parse_single_line(input: TSpan) -> PResult<Vec<Node>> {
    parse_line(alt((
        map(parse_macro_call, |n| vec![n]),
        map(parse_equate, |n| vec![n]),
        map(parse_command, |n| vec![n]),
        parse_multi_opcode_vec,
    )))(input)
}

pub fn parse_span_vec(input: TSpan) -> PResult<Vec<Node>> {
    let (rest, matched) = parse_span(input)?;
    Ok((rest, matched.children.to_vec()))
}

pub fn parse_pc_equate(input: TSpan) -> PResult<Node> {
    map(parse_label, |n| mk_pc_equate(&n))(input)
}

pub fn parse_span(input: TSpan) -> PResult<Node> {
    let mut nodes = NodeCollector::new(input);

    // Many 0 will never fail :(

    let (rest, matched) = many0(alt((
        parse_single_line,
        map(parse_macro_def, |n| vec![n]),
        map(parse_struct, |n| vec![n]),
        map(parse_pc_equate, |n| vec![n]),
    )))(input)?;

    if !rest.is_empty() {
        println!("Unconsumed!");
        let v = rest.iter().map(|t| (t.kind, t.extra)).collect_vec();
        println!("{:#?}", v);
    }

    let matched = matched.into_iter().flatten().collect();
    nodes.add_vec(matched);

    Ok((rest, nodes.into_block()))
}
