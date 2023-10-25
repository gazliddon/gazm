use std::path::PathBuf;

use super::{
    make_tspan, match_span as ms, parse_command, parse_expr, parse_label, parse_macro_def,
    parse_opcode, parse_struct, to_pos, FrontEndError, FrontEndErrorKind, PResult, TSpan, Token,
    TokenKind,
};

use crate::{
    frontend::{get_str, get_text, CommandKind, parse_macro_call},
    item::{Item, LabelDefinition, Node},
    opts::Opts,
};

use grl_sources::{AsmSource, SourceFile};

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

use strum::EnumCount;
use termimad::crossterm::style::Stylize;
use termimad::minimad::Col;
use unraveler::{cut, sep_list, sep_pair, tag, Collection, Splitter};

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
        let node = parse_span(spam)?;

        Ok(Parsed {
            node,
            includes: Default::default(),
            request: self,
        })
    }
}

fn parse_multi_opcode(input: TSpan) -> PResult<Node> {
    use super::match_span as ms;
    use crate::item::Item;
    use TokenKind::Colon;
    let (rest, (sp, matched)) = ms(sep_list(parse_opcode, Colon))(input)?;
    Ok((rest, Node::block(matched.into(), sp)))
}

fn parse_macro_code_def(_input: TSpan) -> PResult<Node> {
    panic!()
}

use thin_vec::ThinVec;

struct Nodes {
    nodes: ThinVec<Node>,
}

impl Nodes {
    pub fn new() -> Self {
        Self {
            nodes: thin_vec::ThinVec::with_capacity(4096),
        }
    }

    pub fn add(&mut self, n: Node) {
        if n.item == Item::Block {
            for i in n.children {
                self.add(i)
            }
        } else {
            println!("Added: {:?}",n.item );
            self.nodes.push(n)
        }
    }

    pub fn add_vec(&mut self, nodes: Vec<Node>) {
        self.nodes.reserve(nodes.len());
        self.nodes.extend(nodes.into_iter())
    }
}

impl Into<ThinVec<Node>> for Nodes {
    fn into(self) -> ThinVec<Node> {
        self.nodes
    }
}

impl Into<Vec<Node>> for Nodes {
    fn into(self) -> Vec<Node> {
        self.nodes.to_vec()
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

pub fn take_line(full_span: TSpan) -> (TSpan, TSpan) {
    let f = || {
        for i in 1..full_span.length() {
            let (rest, matched) = full_span.split(i).expect("That's bad");
            let mpos = &matched.first().unwrap().extra.pos;
            let rpos = &rest.first().unwrap().extra.pos;
            if mpos.line != rpos.line {
                return (rest, matched);
            }
        }

        full_span.split(full_span.length()).unwrap()
    };

    let (rest, matched) = match full_span.length() {
        0 => (full_span,full_span),
        1 => full_span.split(1).unwrap(),
        _ => f(),
    };

    (rest, matched)
}

pub fn parse_line<P>(input: TSpan, mut p: P) -> PResult<Node>
where
    P: FnMut(TSpan) -> PResult<Node> + Copy,
{
    let (rest, line) = take_line(input);
    let (_, matched) = p(line)?;
    Ok((rest, matched))
}

pub fn parse_span(full_span: TSpan) -> Result<Node, FrontEndError> {
    use unraveler::alt;
    let mut input = full_span;
    let mut nodes = Nodes::new();

    while !input.is_empty() {
        let (rest, matched) = alt((
            |i| parse_line(i, parse_macro_call),
            |i| parse_line(i, parse_equate),
            |i| parse_line(i, parse_command),
            |i| parse_line(i, parse_multi_opcode),

            parse_struct,
            parse_label,
        ))(input)?;
        nodes.add(matched);
        input = rest;
    }

    let node = Node::block(nodes.into(), full_span);
    Ok(node)
}
