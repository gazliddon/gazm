#![deny(unused_imports)]
use super::{
    parse_command, parse_equate, parse_label, parse_line, parse_macro_call, parse_macro_def,
    parse_multi_opcode, parse_struct, PResult, TSpan,
};

use crate::{item::{Item, Node}, frontend::get_text};
use thin_vec::ThinVec;
use unraveler::{alt, many0};

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

pub fn parse_single_line(input: TSpan) -> PResult<Node> {
    parse_line(alt((
        parse_macro_call,
        parse_equate,
        parse_command,
        parse_multi_opcode,
    )))(input)
}

pub fn parse_span(input: TSpan) -> PResult<Node> {
    let mut nodes = NodeCollector::new(input);

    println!("to test {}", get_text(input));

    let (rest, matched) = many0(alt((
        parse_macro_def,
        parse_struct,
        parse_single_line,
        parse_label,
    )))(input)?;

    nodes.add_vec(matched);

    Ok((rest, nodes.into_block()))
}
