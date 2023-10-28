#![deny(unused_imports)]
use std::path::PathBuf;


use super::{
    make_tspan, parse_command, parse_expr, parse_label, 
    parse_line,
    parse_multi_opcode, parse_struct, FrontEndError, PResult, TSpan, Token,
    TokenKind,
};

use crate::{
    frontend::parse_macro_call,
    item::{Item, LabelDefinition, Node},
    opts::Opts,
};

use grl_sources::SourceFile;

use unraveler::match_span as ms;
use unraveler::alt;

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

use unraveler::{sep_pair, tag, Collection};

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
        let (_,node) = parse_span(spam)?;

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


fn get_label_definition(item: &Item) -> Option<LabelDefinition> {
    match item {
        Item::Label(l) | Item::LocalLabel(l) => Some(l.clone()),
        _ => None,
    }
}

fn parse_equate(input: TSpan) -> PResult<Node> {
    use super::CommandKind::Equ;
    use Item::Assignment;
    let command: TokenKind = Equ.into();
    let (rest, (sp, (label, expr))) = ms(sep_pair(parse_label, tag(command), parse_expr))(input)?;
    let lab_def = get_label_definition(&label.item).expect("This should be a label kind!");
    let node = Node::from_item_kid_tspan(Assignment(lab_def), expr, sp);
    Ok((rest, node))
}

pub fn parse_single_line(input: TSpan) -> PResult<Node> {
    parse_line(alt((
        parse_macro_call,
        parse_equate,
        parse_command,
        parse_multi_opcode,
    )))(input)
}

pub fn parse_span(full_span: TSpan) -> PResult<Node> {
    let mut input = full_span;

    let mut nodes = NodeCollector::new(full_span);

    while !input.is_empty() {
        let (rest, matched) = alt((parse_struct,parse_single_line, parse_label))(input)?;
        nodes.add(matched);
        input = rest;
    }

    Ok((input, nodes.into_block() ))
}
