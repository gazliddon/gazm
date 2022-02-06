use crate::{
    cli, commands, comments,
    expr::{self, parse_expr},
    item,
    labels::{get_just_label, parse_label},
    locate::matched_span,
    macros::{parse_macro_call, parse_macro_definition},
    messages, opcodes,
    structs::get_struct,
    util::{self, sep_list1, wrapped_chars, ws},
};
use colored::*;
use std::{
    borrow::BorrowMut,
    path::{Path, PathBuf},
    vec,
};

use nom::{
    branch::alt,
    bytes::complete::{is_not, take_until},
    character::complete::{line_ending, multispace0, multispace1},
    combinator::{all_consuming, eof, not, opt, recognize},
    multi::{many0, many1},
    sequence::{pair, preceded, separated_pair, terminated},
    AsBytes,
};
use romloader::ResultExt;

use crate::error::{IResult, ParseError, UserError};
use crate::item::{Item, Node};
use crate::locate::Span;
use romloader::sources::{AsmSource, SourceFileLoader, Sources};

fn get_line(input: Span) -> IResult<Span> {
    let (rest, line) = preceded(
        multispace0,
        terminated(recognize(many0(is_not("\n"))), opt(line_ending)),
    )(input)?;

    Ok((rest, line))
}

struct Token {
    text: String,
    tokens: Vec<item::Node>,
}

pub fn tokenize_file_from_str<'a>(
    file: &PathBuf,
    input: &'a str,
) -> Result<Node, ParseError<'a>> {
    let span = Span::new_extra(input, AsmSource::FromStr);
    let source = input.to_string();
    let mut macros = Macros::new();
    let mut sources = Sources::new();
    let mut matched = Tokens::new().to_tokens(span, &mut sources, &mut macros)?;
    matched.item = Item::TokenizedFile(file.into(), file.into(), source);
    Ok(matched)
}

fn mk_pc_equate(node: Node) -> Node {
    use item::{Item::*, Node};
    let pos = node.ctx().clone();

    match &node.item {
        Label(name) => Node::from_item(AssignmentFromPc(name.clone()), pos),
        LocalLabel(name) => Node::from_item(LocalAssignmentFromPc(name.clone()), pos),
        _ => panic!("shouldn't happen"),
    }
}

struct Tokens {
    tokens: Vec<Node>,
    macro_stack: Vec<String>,
}

impl Default for Tokens {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokens {
    fn new() -> Self {
        Self {
            tokens: vec![],
            macro_stack: vec![],
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

    pub fn to_tokens<'a>(
        &mut self,
        input: Span<'a>,
        sources: &mut Sources,
        macros: &mut Macros,
    ) -> Result<Node, ParseError<'a>> {
        use commands::parse_command;
        use item::{Item,Node};
        use opcodes::parse_opcode;
        use util::parse_assignment;
        use util::ws;
        use crate::macros::MacroCall;

        self.tokens = vec![];

        let ret = Node::from_item_span(Item::Block, input.clone());
        let mut source = input.clone();

        while !source.is_empty() {
            let res = get_struct(source);

            if res.is_ok() {
                let (rest, _matched) = res.unwrap();
                source = rest;
                continue;
            }

            let res = parse_macro_definition(source);

            if res.is_ok() {
                let (rest, def) = res.unwrap();
                macros.add_def(def);
                source = rest;
                continue;
            }

            let (rest, line) = get_line(source)?;

            source = rest;

            if !line.is_empty() {
                let (input, comment) = comments::strip_comments(line)?;
                self.add_some_node(comment);

                if input.is_empty() {
                    continue;
                }

                // An equate
                if let Ok((_, equate)) = all_consuming(ws(parse_assignment))(input) {
                    self.add_node(equate);
                    continue;
                }

                if let Ok((_, label)) = all_consuming(ws(parse_label))(input) {
                    let node = mk_pc_equate(label);
                    self.add_node(node);
                    continue;
                }

                if let Ok((rest, mcall)) = ws(parse_macro_call)(input) {
                    let span = matched_span(input, rest);
                    let node = Node::from_item_span(Item::MacroCall(mcall), span);
                    self.add_node(node);
                    continue;
                }

                let body = alt((ws(parse_opcode), ws(parse_command)));

                let (_, (label, body)) = all_consuming(pair(opt(parse_label), body))(input)?;

                let label = label.map(mk_pc_equate);
                self.add_some_node(label);
                self.add_node(body);
            }
        }

        {
            // Expand all macros for this block of stuff
            let mut tokes = self.tokens.clone();
            self.tokens = vec![];

            let mcalls : Vec<(&mut Node, MacroCall)>= tokes.iter_mut().filter_map(|x| match x.item.clone() {
                Item::MacroCall(mcall) => Some((x, mcall.clone())),
                _ => None,
            }).collect();

            for (node, macro_call) in mcalls {
                let (pos, text) = macros.expand_macro(sources, macro_call.clone());
                let input = Span::new_extra(&text, pos.src);
                let mut new_node = self.to_tokens(input, sources, macros).unwrap();
                new_node.item = Item::ExpandedMacro(macro_call);
                *node = new_node;
            }

            Ok(ret.with_children(tokes))
        }
    }
}

fn tokenize_file(
    depth: usize,
    _ctx: &cli::Context,
    fl: &mut SourceFileLoader,
    file: &std::path::Path,
    parent: &std::path::Path,
    macros: &mut Macros,
) -> anyhow::Result<Node> {
    use anyhow::Context;

    use super::messages::*;
    use item::Item::*;
    let x = messages::messages();

    let (file_name, source, id) = fl
        .read_to_string(file)
        .with_context(|| format!("Failed to load file: {}", file.to_string_lossy()))?;

    let action = if depth == 0 {
        "Tokenizing"
    } else {
        "Including"
    };

    let mapper = |e| UserError::from_parse_error(e, &file_name);

    let comp_msg = format!("{} {}", action, file_name.to_string_lossy());

    x.info(&comp_msg);

    let input = Span::new_extra(&source, AsmSource::FileId(id));

    let mut matched = Tokens::new()
        .to_tokens(input, &mut fl.sources, macros)
        .map_err(mapper)?;

    matched.item = TokenizedFile(file.to_path_buf(), parent.to_path_buf(), source.clone());

    // Tokenize includes
    for n in matched.children.iter_mut() {
        if let Some(inc_file) = n.get_include_file() {
            x.indent();
            *n = tokenize_file(depth + 1, _ctx, fl, inc_file, file, macros)?.into();
            x.deindent();
        }
    }

    Ok(matched)
}
use crate::macros::Macros;

pub fn tokenize(ctx: &cli::Context) -> anyhow::Result<(Node, Sources)> {
    let mut macros = Macros::new();
    let file = ctx.file.clone();

    let mut paths = vec![];

    if let Some(dir) = file.parent() {
        paths.push(dir);
    }

    let mut fl = SourceFileLoader::from_search_paths(&paths);
    let parent = PathBuf::new();

    let res = tokenize_file(0, ctx, &mut fl, &ctx.file, &parent, &mut macros)?;
    Ok((res, fl.sources))
}

////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {

    use super::*;
    #[allow(unused_imports)]
    use pretty_assertions::{assert_eq, assert_ne};
}
