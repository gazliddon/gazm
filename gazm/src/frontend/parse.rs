// #![deny(unused_imports)]
use super::{
    parse_command, parse_equate, parse_label, parse_line, parse_macro_call, parse_macro_def,
    parse_multi_opcode_vec, parse_struct, split_at_next_line, to_pos, utils::mk_pc_equate,
    FrontEndError, FrontEndErrorKind, Item, Node, PResult, TSpan,
};

use itertools::{Itertools, Position};
use thin_vec::ThinVec;
use unraveler::{all, alt, many0, map, Collection, Severity};

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

pub fn parse_pc_equate(input: TSpan) -> PResult<Node> {
    map(parse_label, |n| mk_pc_equate(&n))(input)
}
/// Parse the next chunk of valid source
pub fn parse_next_source_chunk(input: TSpan) -> PResult<Vec<Node>> {
    let (rest, matched) = alt((
        parse_single_line,
        map(parse_macro_def, |n| vec![n]),
        map(parse_struct, |n| vec![n]),
        map(parse_pc_equate, |n| vec![n]),
    ))(input)?;
    Ok((rest, matched))
}

/// Parse as many chunks of source that I can
pub fn parse_source_chunks(input: TSpan) -> PResult<Vec<Node>> {
    let mut nodes = NodeCollector::new(input);
    let (rest, matched) = many0(parse_next_source_chunk)(input)?;
    nodes.add_vec(matched.into_iter().flatten().collect_vec());
    Ok((rest, nodes.nodes.into()))
}

/// Parse all of this span
/// until we have too many errors or have parsed everything
pub fn parse_all_with_resume(mut input: TSpan) -> PResult<Vec<Node>> {
    let mut ret = vec![];
    let mut errors = vec![];
    let max_errors = input.extra().opts.max_errors;

    while input.length() == 0 || errors.len() > max_errors {
        let parse_result = parse_next_source_chunk(input);

        match parse_result {
            Err(e) => {
                errors.push(e);
                let (_, next_line) = split_at_next_line(input)?;
                input = next_line;
            }
            Ok((rest, matched)) => {
                input = rest;
                ret.extend(matched);
            }
        }
    }

    if errors.is_empty() {
        Ok((input, ret))
    } else {
        Err(FrontEndError::new(
            input,
            FrontEndErrorKind::TooManyErrors(errors),
            Severity::Fatal,
        ))
    }
}
