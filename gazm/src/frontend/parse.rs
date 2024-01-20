use crate::error::{ NewErrorCollector, ErrorCollectorTrait };

// #![deny(unused_imports)]
use super::{
    get_text, parse_command, parse_equate, parse_label, parse_line, parse_macro_call,
    parse_macro_def, parse_struct, split_at_next_line, utils::mk_pc_equate,
    FrontEndError, FrontEndErrorKind, Item, Node, PResult, TSpan,
};

// TODO: Remove6809
use crate::cpu6809::frontend::parse_multi_opcode_vec;

use itertools::Itertools;
use num_traits::Float;
use thin_vec::ThinVec;
use unraveler::{alt, many0, map, Collection, ParseError, ParseErrorKind, Severity};


struct NodeCollector<'a> {
    nodes: ThinVec<Node>,
    _span: TSpan<'a>,
}

impl<'a> NodeCollector<'a> {
    pub fn new(sp: TSpan<'a>) -> Self {
        Self {
            nodes: thin_vec::ThinVec::with_capacity(4096),
            _span: sp,
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
        for n in nodes {
            self.add(n)
        }
    }
}

// I need isolate parse_command
// and parse opcode

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

fn as_vec(n: Node) -> Vec<Node> {
    vec![n]
}

/// Parse the next chunk of valid source
pub fn parse_next_source_chunk(input: TSpan) -> PResult<Vec<Node>> {
    use FrontEndErrorKind::*;

    // If we can't parse this chunk we need to xform the ParseError into
    // something relevant upstream
    let err_map = |e: FrontEndError| match &e.kind {
        ParseError(ParseErrorKind::NoMatch) => FrontEndError {
            kind: FrontEndErrorKind::Unexpected,
            ..e
        },
        _ => e,
    };

    let (rest, matched) = alt((
        parse_single_line,
        map(parse_macro_def, as_vec),
        map(parse_struct, as_vec),
        map(parse_pc_equate, as_vec),
    ))(input)
    .map_err(err_map)?;

    Ok((rest, matched))
}

/// Parse all of this span
/// until we have too many errors or have parsed everything
pub fn parse_all_with_resume(mut input: TSpan) -> Result<(TSpan,Vec<Node>), NewErrorCollector<FrontEndError>> {
    let mut ret = vec![];
    let max_errors = input.extra().opts.max_errors;
    let mut errors :NewErrorCollector<FrontEndError> = NewErrorCollector::new(max_errors);

    while !errors.is_over_max_errors() && input.length() > 0 {
        let parse_result = parse_next_source_chunk(input);

        match parse_result {
            Err(e) => {
                errors.add(e);
                let (next_line, _this_line) = split_at_next_line(input).expect("Can't split!");
                input = next_line;
            }

            Ok((rest, matched)) => {
                ret.extend(matched);
                input = rest;
            }
        };
    }

    if !errors.has_errors() {
        Ok((input, ret))
    } else {
        Err(errors)
    }
}
