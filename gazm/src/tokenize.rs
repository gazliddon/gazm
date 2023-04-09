use futures::AsyncWriteExt;

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

use crate::{
    async_tokenize::tokenize_file as real_tokenize_file,
    commands, comments,
    ctx::{Context, Opts},
    error::{parse_error, ErrorCollector, GResult, GazmErrorType, IResult, ParseError, UserError},
    item::{Item, Node, LabelDefinition},
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

fn mk_pc_equate(node: Node) -> Node {
    use Item::*;
    let pos = node.ctx().clone();
    
    match &node.item {
        Label(label_def) => Node::from_item(AssignmentFromPc(label_def.clone()), pos),
        LocalLabel(label_def) => Node::from_item(LocalAssignmentFromPc(label_def.clone()), pos),
        _ => panic!("shouldn't happen"),
    }
}

struct Tokens {
    tokens: Vec<Node>,
    opts: Opts,
    parse_errors: Vec<ParseError>,
    includes: Vec<PathBuf>,
}

impl Tokens {
    fn new(text: Span, opts: Opts) -> GResult<Self> {
        let mut x = Self {
            tokens: vec![],
            opts,
            // tok_ctx,
            parse_errors: vec![],
            includes: vec![],
        };

        x.parse_to_tokens(text)?;

        Ok(x)
    }

    fn add_node(&mut self, node: Node) {
        self.tokens.push(node)
    }

    fn trailing_text<'a>(&'a mut self, input: Span<'a>) -> IResult<()> {
        if !input.is_empty() {
            let (rest, node) = parse_trailing_line_text(&self.opts, input)?;
            self.add_node(node);
            Ok((rest, ()))
        } else {
            Ok((input, ()))
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
            let node = mk_pc_equate(label);
            self.add_node(node);
            input = rest;
        }

        // if this is an opcode parse and return
        if let Ok((rest, node)) =
            alt((ws(parse_command), ws(parse_opcode), ws(parse_macro_call)))(input)
        {
            match node.item() {
                Item::Include(name) => self.add_include(name.clone()),
                _ => (),
            }

            self.add_node(node);
            self.trailing_text(rest)
        } else {
            self.trailing_text(input)
        }
    }

    fn to_tokens(self) -> Vec<Node> {
        self.tokens
    }

    fn add_include<P: AsRef<Path>>(&mut self, file: P) {
        self.includes.push(file.as_ref().into())
    }

    fn add_includes(&mut self, paths: &[PathBuf]) {
        for p in paths {
            self.includes.push(p.clone())
        }
    }

    fn parse_to_tokens(&mut self, input: Span) -> GResult<()> {
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
                let macro_tokes = Tokens::new(body, self.opts.clone())?.to_tokens();

                let pos = crate::locate::span_to_pos(body);
                let name = name.to_string();
                let params = params.iter().map(|x| x.to_string()).collect();

                let macro_def =
                    Node::new_with_children(Item::MacroDef(name, params), macro_tokes, pos);
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

pub fn tokenize_file<P: AsRef<Path>>(
    depth: usize,
    ctx: &mut crate::ctx::Context,
    file: P,
    parent: Option<PathBuf>,
    do_includes: bool,
) -> GResult<PathBuf> {
    use anyhow::Context;
    use Item::*;

    let file = ctx.get_full_path(file)?;

    if !ctx.has_tokens(&file) {
        let tokenized = real_tokenize_file(ctx, &file, parent)?;

        for e in &tokenized.errors {
            ctx.add_parse_error(e.clone())?;
        }

        let ts = ctx.get_token_store_mut();
        ts.add_tokens(&file, tokenized.node);

        if do_includes {
            let includes = ctx.get_untokenized_files(&tokenized.includes);
            for inc_file in includes {
                tokenize_file(depth + 1, ctx, &inc_file, Some(file.clone()), true)?;
            }
        }
    }

    Ok(file)
}

impl From<Tokens> for TokenizedText {
    fn from(toks: Tokens) -> Self {
        Self {
            tokens: toks.tokens,
            parse_errors: toks.parse_errors,
            includes: toks.includes,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn tokenize<P: AsRef<Path>>(
    arc_ctx: &Arc<Mutex<Context>>,
    file: P,
    do_includes: bool,
) -> GResult<Node> {
    let msg = format!("Reading {}", file.as_ref().to_string_lossy());
    messages().status(msg);

    let mut ctx_arc = arc_ctx.lock().unwrap();
    let ctx = &mut ctx_arc.deref_mut();

    let file_tokenized = tokenize_file(0, ctx, &file, None, do_includes)?;

    ctx.asm_out.errors.raise_errors()?;
    let toks = ctx.get_tokens(&file_tokenized).unwrap().clone();
    Ok(toks)
}

pub struct TokenizedText {
    pub tokens: Vec<Node>,
    pub includes: Vec<PathBuf>,
    pub parse_errors: Vec<ParseError>,
}

pub fn tokenize_text(ctx: &Context, text: Span) -> GResult<TokenizedText> {
    let mut toks = Tokens::new(text, ctx.opts.clone())?;
    let includes: GResult<Vec<_>> = toks
        .includes
        .iter()
        .cloned()
        .map(|path| ctx.get_full_path(path))
        .collect();

    toks.includes = includes?;

    Ok(toks.into())
}

////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {
    use super::*;
    #[allow(unused_imports)]
    use pretty_assertions::{assert_eq, assert_ne};
}
