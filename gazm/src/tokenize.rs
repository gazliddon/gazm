use futures::AsyncWriteExt;

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::is_not,
    character::complete::{line_ending, multispace0},
    combinator::{all_consuming, cut, not, opt, recognize},
    multi::many0,
    sequence::{preceded, terminated},
};

use std::{
    ops::DerefMut,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::{parse::parse_comment, async_tokenize::TokenizeResult};

use crate::{
    ast::AstNodeId,
    async_tokenize::tokenize_file as real_tokenize_file,
    commands,
    ctx::{Context, Opts},
    error::{parse_error, ErrorCollector, GResult, GazmErrorKind, IResult, ParseError, UserError},
    item::{Item, LabelDefinition, Node},
    labels::parse_label,
    locate::{matched_span, span_to_pos, Span},
    macros::{get_macro_def, get_scope_block, parse_macro_call},
    messages::messages,
    opcodes::parse_opcode,
    parse::util::{parse_assignment, ws},
    structs::{get_struct, parse_struct_definition},
};

use emu::utils::sources;

use emu::utils::PathSearcher;
use sources::fileloader::SourceFileLoader;
use sources::{AsmSource, Position};

fn get_line(input: Span) -> IResult<Span> {
    let (rest, line) = cut(preceded(
        multispace0,
        terminated(recognize(many0(is_not("\r\n"))), opt(line_ending)),
    ))(input)?;

    Ok((rest, line))
}

fn parse_comments(stars: bool, input: Span) -> IResult<Node> {
    use crate::parse::{parse_comment, parse_star_comment};
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
    parse_label(input)
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
}

impl Tokens {
    pub fn from_text(ctx: &Context, text: Span) -> GResult<Self> {
        let mut x = Self {
            opts: ctx.opts.clone(),
            ..Default::default()
        };

        x.parse_to_tokens(ctx, text)?;

        if ctx.opts.do_includes {
            let includes: GResult<Vec<_>> = x
                .includes
                .into_iter()
                .unique()
                .map(|(pos, path)| Ok((pos, ctx.get_full_path(path)?)))
                .collect();
            x.includes = includes?;
        }

        Ok(x)
    }

    fn add_node(&mut self, node: Node) {
        self.tokens.push(node);
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

    fn tokenize_line<'a>(&'a mut self, line: Span<'a>) -> IResult<()> {
        use commands::parse_command;

        let mut input = line;

        if line.is_empty() && self.opts.encode_blank_lines {
            let node = Node::from_item_span(Item::BlankLine, line);
            self.add_node(node);
            return Ok((input, ()));
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
            let node = mk_pc_equate(&label);
            self.add_node(node);
            input = rest;
        }

        // if this is an opcode parse and return
        if let Ok((rest, node)) =
            alt((ws(parse_command), ws(parse_opcode), ws(parse_macro_call)))(input)
        {
            if let Item::Include(name) = &node.item {
                self.add_include_with_pos(node.ctx.clone(), name.clone())
            }

            self.add_node(node);
            self.trailing_text(rest)
        } else {
            self.trailing_text(input)
        }
    }

    fn take_tokens(self) -> Vec<Node> {
        self.tokens
    }

    fn add_include_with_pos<P: AsRef<Path>>(&mut self, pos: Position, file: P) {
        self.includes.push((pos, file.as_ref().into()));
    }

    fn parse_to_tokens(&mut self, ctx: &Context, input: Span) -> GResult<()> {
        use crate::macros::MacroCall;

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
                use std::string::ToString;
                let macro_tokes = Tokens::from_text(ctx, body)?.take_tokens();

                let pos = crate::locate::span_to_pos(body);
                let name = name.to_string();
                let params = params.iter().map(ToString::to_string).collect();

                let macro_def =
                    Node::new_with_children(Item::MacroDef(name, params), &macro_tokes, pos);
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

                self.tokenize_line(line)?;
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

fn from_file<P: AsRef<Path>>(
    _depth: usize,
    ctx: &mut crate::ctx::Context,
    file: P,
    parent: Option<PathBuf>,
) -> GResult<PathBuf> {
    use anyhow::Context;
    use Item::*;

    let file = ctx.get_full_path(file)?;

    if !ctx.get_tokens(&file).is_some() {
        let tokenized = real_tokenize_file(ctx, &file, parent)?;

        for e in &tokenized.errors {
            ctx.add_parse_error(e.clone())?;
        }


        if ctx.opts.do_includes {
            let includes = ctx.get_untokenized_files(&tokenized.includes);
            for (_pos, inc_file) in includes {
                from_file(_depth + 1, ctx, &inc_file, Some(file.clone()))?;
            }
        }

        let ts = ctx.get_token_store_mut();
        ts.add_tokens(&file, tokenized);
    }
    Ok(file)
}

////////////////////////////////////////////////////////////////////////////////

pub fn tokenize<P: AsRef<Path>>(ctx: &mut Context, file: P) -> GResult<TokenizeResult> {
    let msg = format!("Reading {}", file.as_ref().to_string_lossy());
    messages().status(msg);

    // let mut ctx_arc = arc_ctx.lock().unwrap();
    // let ctx = &mut *ctx_arc;
    let file_tokenized = from_file(0, ctx, &file, None)?;
    ctx.asm_out.errors.raise_errors()?;
    let toks = ctx.get_tokens(&file_tokenized).unwrap().clone();
    Ok(toks)
}

////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {
    use super::*;
    #[allow(unused_imports)]
    use pretty_assertions::{assert_eq, assert_ne};
}
