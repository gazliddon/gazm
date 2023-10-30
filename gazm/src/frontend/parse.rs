#![deny(unused_imports)]

use grl_sources::SourceFile;
use std::path::PathBuf;
use unraveler::{alt, Collection};

use super::{
    make_tspan, parse_command,  parse_label, parse_line, parse_macro_call,
    parse_struct, FrontEndError, 
    PResult, TSpan, Token, 
    parse_macro_def,
    parse_multi_opcode,
    parse_equate,

};

use crate::{
    item::{Item,  Node},
    opts::Opts,
};

#[derive(Debug, Clone)]
pub struct ParseTask {
    opts: Opts,
    source_file: SourceFile,
}

#[derive(Debug, Clone)]
pub struct Parsed {
    pub node: Node,
    pub includes: Vec<PathBuf>,
    pub request: ParseTask,
}

impl ParseTask {
    pub fn from_text(opts: &Opts, text: &str) -> Self {
        let source_file = SourceFile::new("NO FILE", text, 0);
        Self::from_source(opts, &source_file)
    }

    pub fn from_source(opts: &Opts, source_file: &SourceFile) -> Self {
        Self {
            opts: opts.clone(),
            source_file: source_file.clone(),
        }
    }

    fn tokenize(&self) -> Vec<Token> {
        super::to_tokens_filter(&self.source_file, |k| k.is_comment())
    }
}

impl TryInto<Parsed> for ParseTask {
    type Error = FrontEndError;

    fn try_into(self) -> Result<Parsed, Self::Error> {
        let tokens = self.tokenize();
        let spam = make_tspan(&tokens, &self.source_file);
        let (_, node) = parse_span(spam)?;

        Ok(Parsed {
            node,
            includes: Default::default(),
            request: self,
        })
    }
}

fn parse_macro_code_def(_input: TSpan) -> PResult<Node> {
    panic!()
}

use thin_vec::ThinVec;

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

pub fn parse_span<'a>(full_span: TSpan<'a>) -> PResult<'a, Node> {
    let mut input = full_span;
    let mut nodes = NodeCollector::new(full_span);

    while !input.is_empty() {
        let (rest, matched) = alt((
            parse_macro_def,
            parse_struct,
            parse_single_line,
            parse_label,
        ))(input)?;
        nodes.add(matched);
        input = rest;
    }

    Ok((input, nodes.into_block()))
}
