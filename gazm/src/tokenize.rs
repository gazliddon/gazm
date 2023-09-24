/// Parses text into a load of structured tokens
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{anychar, line_ending, multispace0, char as nom_char},
    combinator::{cut, not, opt, recognize},
    multi::many0,
    sequence::{pair, preceded, terminated, tuple},
};

use std::path::{Path, PathBuf};
use tryvial::try_block;

use crate::{
    opts::Opts,
    error::{GResult, IResult, ParseError},
    item::{Item, Node},
    parse6809::opcodes::parse_opcode,

    parse::{
        commands::parse_command,
        comments::{parse_comment, parse_star_comment},
        labels::parse_label,
        locate::span_to_pos,
        locate::Span,
        macros::{get_macro_def, get_scope_block, parse_macro_call},
        structs::{get_struct, parse_struct_definition},
        util::{parse_assignment, ws},
    },

};

use grl_sources::Position;

fn get_line_cut(input: Span) -> IResult<Span> {
    cut(get_line)(input)
}

fn get_line(input: Span) -> IResult<Span> {
    ws(terminated(
        recognize(many0(is_not("\r\n"))),
        opt(line_ending),
    ))(input)
}

fn parse_comments(stars: bool, input: Span) -> IResult<Node> {
    if stars {
        if let Ok((rest, comment)) = parse_star_comment(input) {
            return Ok((rest, comment));
        }
    }
    parse_comment(input)
}

fn parse_trailing_line_text<'a>(opts: &Opts, input: Span<'a>) -> IResult<'a, Node> {
    if let Ok((rest, matched)) = parse_comments(opts.star_comments, input) {
        Ok((rest, matched))
    } else if opts.trailing_comments {
        let node = Node::from_item_span(Item::Comment(input.to_string()), input);
        Ok((input, node))
    } else {
        let message = "Unexpected characters";
        let pe = ParseError::new(message.to_string(), &input, false);
        Err(nom::Err::Error(pe))
    }
}

fn parse_label_not_macro(input: Span) -> IResult<Node> {
    let (_, _) = not(parse_macro_call)(input)?;
    ws(parse_label)(input)
}

fn mk_pc_equate(node: &Node) -> Node {
    use Item::{AssignmentFromPc, Label, LocalAssignmentFromPc, LocalLabel};
    let pos = node.ctx.clone();

    match &node.item {
        Label(label_def) => Node::new(AssignmentFromPc(label_def.clone()), pos),
        LocalLabel(label_def) => Node::new(LocalAssignmentFromPc(label_def.clone()), pos),
        _ => panic!("shouldn't happen"),
    }
}

#[derive(Default)]
pub struct Tokens {
    pub tokens: Vec<Node>,
    pub opts: Opts,
    pub parse_errors: Vec<ParseError>,
    pub includes: Vec<(Position, PathBuf)>,
    docs: DocTracker,
}

#[derive(Default)]
struct DocTracker {
    doc_lines: Vec<String>,
}

impl DocTracker {
    pub fn has_docs(&self) -> bool {
        !self.doc_lines.is_empty()
    }

    pub fn add_doc_line(&mut self, doc: &str) {
        self.doc_lines.push(doc.to_string())
    }
    pub fn flush_docs(&mut self) -> String {
        let ret = self.doc_lines.join("\n");
        *self = Default::default();
        ret
    }
}

fn parse_pc_label(input: Span) -> IResult<Node> {
    let (rest, matched) = parse_label_not_macro(input)?;
    let node = mk_pc_equate(&matched);
    Ok((rest, node))
}

impl Tokens {
    pub fn from_text(opts: &Opts, text: Span) -> GResult<Self> {
        let mut tokens = Self {
            opts: opts.clone(),
            ..Default::default()
        };
        tokens.parse_to_tokens(text).map(|_| tokens)
    }

    // Add a node with no doc
    fn add_node(&mut self, node: Node) {
        let doc = self.docs.flush_docs();

        if !doc.is_empty() {
            // TODO - do something about this
        }

        if let Item::Include(name) = &node.item {
            self.add_include_with_pos(node.ctx.clone(), name.clone())
        }

        self.tokens.push(node);
    }

    fn add_node_with_doc(&mut self, mut node: Node) {
        let doc = self.docs.flush_docs();
        if !doc.is_empty() {
            let doc_node = Node::new(Item::Doc(doc), node.ctx.clone());
            node.add_child(doc_node)
        }
        self.add_node(node)
    }

    fn trailing_text<'a>(&'a mut self, input: Span<'a>) -> IResult<()> {
        if input.is_empty() {
            Ok((input, ()))
        } else {
            let (rest, node) = parse_trailing_line_text(&self.opts, input)?;
            self.add_node(node);
            Ok((rest, ()))
        }
    }

    fn tokenize_line<'a>(&'a mut self, input: Span<'a>) -> IResult<()> {
        let rest = if input.is_empty() && self.opts.encode_blank_lines {
            let node = Node::from_item_span(Item::BlankLine, input);
            self.add_node(node);
            return Ok((input, ()));
        } else if let Ok((rest, doc)) = get_doc_line(input) {
            self.docs.add_doc_line(&doc);
            rest
        } else if let Ok((rest, node)) = parse_comments(self.opts.star_comments, input) {
            self.add_node(node);
            rest
        }
        // If we find an equate, parse and return
        else if let Ok((rest, (equate, doc))) =
            ws(tuple((parse_assignment, opt(parse_doc_line))))(input)
        {
            // Lol: if we have a doc line at the end of this line
            // then add it to the docs
            if let Some(Node {
                item: Item::Doc(doc),
                ..
            }) = doc
            {
                self.docs.add_doc_line(&doc);
            }
            self.add_node_with_doc(equate);
            rest
        }
        // if this is an opcode parse and return
        else if let Ok((rest, (opt_label, node, doc))) = ws(tuple((
            opt(parse_pc_label),
            alt((parse_command, parse_opcode, parse_macro_call)),
            opt(parse_doc_line),
        )))(input)
        {
            // Lol: if we have a doc line at the end of this line
            // then add it to the docs
            if let Some(Node {
                item: Item::Doc(doc),
                ..
            }) = doc
            {
                self.docs.add_doc_line(&doc);
            }

            if let Some(label) = opt_label {
                self.add_node_with_doc(label);
            }

            self.add_node(node);
            rest
        }
        // If this just a label
        else if let Ok((rest, (label, doc))) =
            ws(tuple((parse_pc_label, opt(parse_doc_line))))(input)
        {
            if let Some(Node {
                item: Item::Doc(doc),
                ..
            }) = doc
            {
                self.docs.add_doc_line(&doc);
            }
            self.add_node_with_doc(label);
            rest
        } else {
            return self.trailing_text(input);
        };

        self.trailing_text(rest)
    }

    fn take_tokens(self) -> Vec<Node> {
        self.tokens
    }

    fn add_include_with_pos<P: AsRef<Path>>(&mut self, pos: Position, file: P) {
        self.includes.push((pos, file.as_ref().into()));
    }

    fn parse_to_tokens(&mut self, input: Span) -> GResult<()> {
        let mut source = input;

        while !source.is_empty() {
            if let Ok((_rest, (_name, _body))) = get_scope_block(source) {
                panic!();
                // let tok_result = Tokens::new(body, self.opts.clone())?;
                // self.add_includes(&tok_result.includes);
                // let toks = tok_result.to_tokens();
                // let scope_def = Node::new_with_children(
                //     Item::Scope2(name.to_string()),
                //     toks,
                //     span_to_pos(body),
                // );
                // self.add_node(scope_def);
                // source = rest;
                // continue;
            }

            if let Ok((rest, (name, params, body))) = get_macro_def(source) {
                let macro_tokes = Tokens::from_text(&self.opts, body)?.take_tokens();

                let pos = span_to_pos(body);
                let name = name.to_string();
                let params = params.iter().map(ToString::to_string).collect();

                let macro_def =
                    Node::new_with_children(Item::MacroDef(name, params), &macro_tokes, pos);
                self.add_node(macro_def);
                source = rest;
                continue;
            }

            let res: Result<(), ParseError> = try_block! {
                if let Ok((rest, _)) = get_struct(source) {
                    let (_, matched) = parse_struct_definition(source)?;
                    self.add_node(matched);
                    source = rest;
                } else {
                    let (rest, line) = get_line_cut(source)?;
                    source = rest;
                    self.tokenize_line(line)?;
                }
            };

            match res {
                Ok(..) => (),
                Err(pe) => {
                    self.parse_errors.push(pe);
                }
            };
        }

        Ok(())
    }
}

pub fn get_doc_line(input: Span) -> IResult<Span> {
    let rest = input;
    let (rest, matched) = get_line_cut(rest)?;
    let (_, matched) = preceded(tuple(( multispace0, tag(";;;"),opt(nom_char(' '))  )), recognize(many0(anychar)))(matched)?;
    Ok((rest, matched))
}

pub fn parse_doc_line(input: Span) -> IResult<Node> {
    use Item::*;
    let (rest, matched) = get_doc_line(input)?;
    let node = Node::from_item_span(Doc(matched.to_string()), input);
    Ok((rest, node))
}
