use crate::{
    commands, comments,
    ctx::Opts,
    gazm::Gazm,
    item::{Item, Node},
    labels::parse_label,
    locate::{matched_span, span_to_pos},
    macros::{get_macro_def, parse_macro_call},
    messages::messages,
    opcodes,
    structs::{get_struct, parse_struct_definition},
    util::{self, ws},
};

use crate::error::{GResult, GazmError};
use std::path::{Path, PathBuf};

use nom::{
    branch::alt,
    bytes::complete::is_not,
    character::complete::{line_ending, multispace0},
    combinator::{all_consuming, cut, not, opt, recognize},
    multi::many0,
    sequence::{preceded, terminated},
};
use petgraph::visit::GraphRef;
use emu::utils::sources::Position;

use crate::error::{ErrorCollector, IResult, ParseError, UserError};
use crate::locate::Span;
use emu::utils::sources::AsmSource;

fn get_line(input: Span) -> IResult<Span> {
    let (rest, line) = cut(preceded(
        multispace0,
        terminated(recognize(many0(is_not("\r\n"))), opt(line_ending)),
    ))(input)?;

    Ok((rest, line))
}

fn parse_comments(stars: bool, input: Span) -> IResult<Node> {
    if stars {
        if let Ok((rest, comment)) = comments::parse_star_comment(input) {
            return Ok((rest, comment));
        }
    }
    comments::parse_comment(input)
}

fn parse_trailing_line_text<'a>(opts: &Opts, input: Span<'a>) -> IResult<'a, Node> {
    if let Ok((rest, matched)) = parse_comments(opts.star_comments, input) {
        return Ok((rest, matched));
    } else {
        if opts.trailing_comments {
            let node = Node::from_item_span(Item::Comment(input.to_string()), input);
            Ok((input, node))
        } else {
            let message = "Unexpected characters";
            let pe = ParseError::new(message.to_string(), &input, false);
            return Err(nom::Err::Error(pe));
        }
    }
}

fn parse_label_not_macro(input: Span) -> IResult<Node> {
    let (_, _) = not(parse_macro_call)(input)?;
    parse_label(input)
}

pub fn tokenize_file_from_str<P>(
    file: P,
    input: &str,
    ctx: &mut crate::ctx::Context,
    opts: Opts,
) -> GResult<Node>
where
    P: AsRef<Path>,
{
    let pb: PathBuf = file.as_ref().into();
    let span = Span::new_extra(input, AsmSource::FromStr);
    let mut tokes = Tokens::new(ctx, &opts);
    tokes.add_tokens(span)?;
    let tokes = tokes.to_tokens();
    let item = Item::TokenizedFile(pb.clone(), pb);
    let file_node = Node::from_item_span(item, span).with_children(tokes);
    Ok(file_node)
}

fn mk_pc_equate(node: Node) -> Node {
    use Item::*;
    let pos = node.ctx().clone();

    match &node.item {
        Label(name) => Node::from_item(AssignmentFromPc(name.clone()), pos),
        LocalLabel(name) => Node::from_item(LocalAssignmentFromPc(name.clone()), pos),
        _ => panic!("shouldn't happen"),
    }
}

pub struct Tokens<'a> {
    tokens: Vec<Node>,
    ctx: &'a mut crate::ctx::Context,
    opts: Opts,
}

impl<'a> Tokens<'a> {
    pub fn new(ctx: &'a mut crate::ctx::Context, opts: &Opts) -> Self {
        Self {
            tokens: vec![],
            ctx,
            opts: opts.clone(),
        }
    }

    fn add_some_node(&mut self, node: Option<Node>) {
        if let Some(node) = node {
            self.add_node(node)
        }
    }

    fn add_node(&mut self, node: Node) {
        self.tokens.push(node)
    }

    fn add_comment(&mut self, text: Span) {
        let node = Node::from_item_span(Item::Comment(text.to_string()), text);
        self.add_node(node)
    }

    fn trailing_text(&mut self, input: Span<'a>) -> IResult<()> {
        if !input.is_empty() {
            let (rest, node) = parse_trailing_line_text(&self.opts, input)?;
            self.add_node(node);
            Ok((rest, ()))
        } else {
            Ok((input, ()))
        }
    }

    fn tokenize_line(&mut self, line: Span<'a>) -> IResult<()> {
        use commands::parse_command;
        use opcodes::parse_opcode;
        use util::parse_assignment;

        let mut input = line;

        if line.is_empty() && self.opts.encode_blank_lines {
            let node = Node::from_item_span(Item::BlankLine, line);
            self.add_node(node);
            return Ok((input,()))
        }

        if let Ok((rest, node)) = parse_comments(self.opts.star_comments, input) {
            self.add_node(node);
            return self.trailing_text(rest);
        }

        // If we find an equate, parse and return
        if let Ok((rest, equate)) = ws(parse_assignment)(input) {
            self.add_node(equate);
            return self.trailing_text(rest);
        }

        // If this is a label, add the label and carry on
        if let Ok((rest, label)) = ws(parse_label_not_macro)(input) {
            let node = mk_pc_equate(label);
            self.add_node(node);
            input = rest;
        }

        // if this is an opcode parse and return
        if let Ok((rest, body)) = 
            alt((ws(parse_command), ws(parse_opcode), ws(parse_macro_call)))(input)
        {
            self.add_node(body);
            return self.trailing_text(rest);
        }

        Ok((input, ()))
    }

    pub fn to_tokens(self) -> Vec<Node> {
        self.tokens
    }

    pub fn add_tokens(&mut self, input: Span<'a>) -> GResult<()> {
        use crate::macros::MacroCall;

        // let ret = Node::from_item_span(Item::Block, input.clone());

        let mut source = input;

        while !source.is_empty() {

            if let Ok((rest, (name, params, body))) = get_macro_def(source) {
                let mut macro_tokes = Tokens::new(self.ctx, &self.opts);
                macro_tokes.add_tokens(body).unwrap();
                let macro_tokes = macro_tokes.to_tokens();

                let pos = crate::locate::span_to_pos(body);
                let name = name.to_string();
                let params = params.iter().map(|x| x.to_string()).collect();

                let macro_def = Node::from_item_pos(Item::MacroDef(name, params), pos)
                    .with_children(macro_tokes);
                self.add_node(macro_def);
                source = rest;
                continue;
            }



            let res: Result<(), ParseError> = try {

                if let Ok((rest, _)) = get_struct(source) {
                    let (_, matched) = parse_struct_definition(source)?;
                    self.add_node(matched);
                    source = rest;
                    continue;
                }

                let (rest, line) = get_line(source)?;
                source = rest;

                let _ = self.tokenize_line(line)?;
                ()
            };

            match res {
                Ok(..) => (),
                Err(pe) => {
                    self.ctx.add_parse_error(pe)?;
                }
            };
        }

        Ok(())
    }
}

fn tokenize_file<P: AsRef<Path>, PP: AsRef<Path>>(
    depth: usize,
    ctx: &mut crate::ctx::Context,
    opts: &Opts,
    file: P,
    parent: PP,
) -> GResult<Node> {
    use anyhow::Context;

    use Item::*;
    let x = messages();

    let (file_name, source, id) = ctx.read_source(&file)?;

    let action = if depth == 0 {
        "Tokenizing"
    } else {
        "Including"
    };

    let comp_msg = format!("{} {}", action, file_name.to_string_lossy());
    x.status(&comp_msg);

    let input = Span::new_extra(&source, AsmSource::FileId(id));

    let mut tokes = Tokens::new(ctx, opts);
    tokes.add_tokens(input)?;
    let mut tokes = tokes.to_tokens();

    // Tokenize includes
    for n in tokes.iter_mut() {
        let parent = file.as_ref().to_path_buf().clone();
        if let Include(inc_file) = &n.item {
            x.indent();
            *n = tokenize_file(depth + 1, ctx, opts, inc_file, &parent)?;
            x.deindent();
        };
    }

    let item = TokenizedFile(file.as_ref().into(), parent.as_ref().into());
    let node = Node::from_item_span(item, input).with_children(tokes);
    Ok(node)
}

use crate::macros::Macros;

pub fn tokenize<P: AsRef<Path>>(
    ctx: &mut crate::ctx::Context,
    opts: &Opts,
    file: P,
) -> GResult<Node> {
    let parent = PathBuf::new();

    let msg = format!("Reading {}", file.as_ref().to_string_lossy());
    messages().status(msg);

    let block = tokenize_file(0, ctx, opts, &file, &parent)?;

    ctx.errors.raise_errors()?;

    Ok(block)
}

////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {
    use super::*;
    #[allow(unused_imports)]
    use pretty_assertions::{assert_eq, assert_ne};
}
