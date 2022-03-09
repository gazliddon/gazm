use crate::{
    commands, comments,
    item::{Item, Node},
    labels::parse_label,
    locate::{matched_span, span_to_pos},
    macros::{parse_macro_call, parse_macro_definition},
    messages::messages,
    opcodes,
    structs::{get_struct, parse_struct_definition},
    util::{self, ws},
};

use std::path::{Path, PathBuf};

use nom::{
    branch::alt,
    bytes::complete::is_not,
    character::complete::{line_ending, multispace0},
    combinator::{all_consuming, cut, opt, recognize},
    multi::many0,
    sequence::{preceded, terminated},
};
use utils::sources::Position;

use crate::error::{IResult, ParseError, UserError, UserErrors};
use crate::locate::Span;
use utils::sources::AsmSource;

fn get_line(input: Span) -> IResult<Span> {
    let (rest, line) = cut(preceded(
        multispace0,
        terminated(recognize(many0(is_not("\n"))), opt(line_ending)),
    ))(input)?;

    Ok((rest, line))
}

pub fn tokenize_file_from_str<'a>(
    file: &Path,
    input: &str,
    errors: &mut UserErrors,
    ctx: &'a mut crate::ctx::Context,
) -> Result<Node, UserError> {
    let span = Span::new_extra(input, AsmSource::FromStr);
    let mut macros = Macros::new();
    let matched = Tokens::new(ctx).convert_to_tokens(span, &mut macros, errors)?;
    let item = Item::TokenizedFile(file.into(), file.into());
    let file_node = Node::from_item_span(item, span).with_children(matched);
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

struct Tokens<'a> {
    tokens: Vec<Node>,
    ctx: &'a crate::ctx::Context,
}

impl<'a> Tokens<'a> {
    fn new(ctx: &'a crate::ctx::Context) -> Self {
        Self {
            tokens: vec![],
            ctx,
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

    fn handle_trailing_text(&mut self, rest: Span) -> Result<(), ParseError> {
        if !rest.is_empty() {
            if self.ctx.trailing_comments {
                self.add_comment(rest);
            } else {
                let message = "Unexpected characters";
                return Err(ParseError::new(message.to_string(), &rest, false));
            }
        }
        Ok(())
    }

    fn tokenize_line(&mut self, line: Span) -> Result<(), ParseError> {
        use commands::parse_command;
        use opcodes::parse_opcode;
        use util::parse_assignment;

        if self.ctx.star_comments {
            if let Ok((_rest, matched)) = comments::strip_star_comment(line) {
                self.add_node(matched);
                return Ok(());
            }
        }

        let (mut input, comment) = comments::strip_comments(line)?;
        self.add_some_node(comment);

        if input.is_empty() {
            return Ok(());
        }

        // An equate
        if let Ok((rest, equate)) = ws(parse_assignment)(input) {
            self.add_node(equate);
            self.handle_trailing_text(rest)?;
            return Ok(());
        }

        if let Ok((rest, mcall)) = ws(parse_macro_call)(input) {
            let span = matched_span(input, rest);
            let node = Node::from_item_span(Item::MacroCall(mcall), span);
            self.add_node(node);
            self.handle_trailing_text(rest)?;
            return Ok(());
        }

        if let Ok((_, label)) = all_consuming(ws(parse_label))(input) {
            let node = mk_pc_equate(label);
            self.add_node(node);
            return Ok(());
        }

        if let Ok((rest, label)) = ws(parse_label)(input) {
            let node = mk_pc_equate(label);
            self.add_node(node);
            input = rest;
        }

        let (rest, body) = alt((ws(parse_command), ws(parse_opcode)))(input)?;

        self.handle_trailing_text(rest)?;
        self.add_node(body);

        Ok(())
    }

    fn convert_to_tokens(
        &mut self,
        input: Span,
        macros: &mut Macros,
        errors: &mut UserErrors,
    ) -> Result<Vec<Node>, UserError> {
        use crate::macros::MacroCall;

        self.tokens = vec![];

        // let ret = Node::from_item_span(Item::Block, input.clone());

        let mut source = input;

        while !source.is_empty() {
            let res: Result<(), ParseError> = try {
                if let Ok(..) = get_struct(source) {
                    let (rest, matched) = parse_struct_definition(source)?;
                    self.add_node(matched);
                    source = rest;
                    continue;
                }

                if let Ok((rest, def)) = parse_macro_definition(source) {
                    macros.add_def(def);
                    source = rest;
                    continue;
                }

                let (rest, line) = get_line(source)?;
                source = rest;

                self.tokenize_line(line)?;
            };

            match &res {
                Ok(..) => (),
                Err(pe) => {
                    errors.add_parse_error(pe.clone(), self.ctx.sources())?;
                }
            };
        }
        errors.raise_errors()?;
        // Expand all macros for this block of stuff
        let mut tokes = self.tokens.clone();

        self.tokens = vec![];

        let mcalls: Vec<(&mut Node, MacroCall)> = tokes
            .iter_mut()
            .filter_map(|x| match x.item.clone() {
                Item::MacroCall(mcall) => Some((x, mcall)),
                _ => None,
            })
            .collect();

        // Expand all macro calls
        //
        for (node, macro_call) in mcalls {
            let (pos, text) = macros.expand_macro(self.ctx.sources(), macro_call.clone())?;

            let input = Span::new_extra(&text, pos.src);

            let new_tokens = self
                .convert_to_tokens(input, macros, errors)
                .map_err(|mut e| {
                    let args: Vec<_> = macro_call
                        .args
                        .iter()
                        .map(|a| self.ctx.sources().get_source_info(a))
                        .collect();

                    let err1 = format!("Macro expansion:\n {}", text);
                    let err2 = format!("Args:\n {:#?}", args);
                    e.message = format!("{}\n{}", err1, err2);
                    e
                })?;
            let pos: Position = span_to_pos(input);

            let new_node =
                Node::from_item_pos(Item::ExpandedMacro(macro_call), pos).with_children(new_tokens);

            *node = new_node;
        }

        Ok(tokes)
    }
}

fn tokenize_file(
    depth: usize,
    ctx: &mut crate::ctx::Context,
    file: &std::path::Path,
    parent: &std::path::Path,
    macros: &mut Macros,
    errors: &mut UserErrors,
) -> anyhow::Result<Node> {
    use anyhow::Context;

    use Item::*;
    let x = messages();

    let (file_name, source, id) = ctx
        .read_source(&file)
        .with_context(|| format!("Failed to load file: {}", file.to_string_lossy()))?;

    let action = if depth == 0 {
        "Tokenizing"
    } else {
        "Including"
    };

    let comp_msg = format!("{} {}", action, file_name.to_string_lossy());
    x.status(&comp_msg);

    let input = Span::new_extra(&source, AsmSource::FileId(id));

    let mut tokes = Tokens::new(ctx).convert_to_tokens(input, macros, errors)?;

    // Tokenize includes
    for n in tokes.iter_mut() {
        if let Include(inc_file) = &n.item {
            x.indent();
            *n = tokenize_file(depth + 1, ctx, inc_file, file, macros, errors)?;
            x.deindent();
        };
    }

    let item = TokenizedFile(file.to_path_buf(), parent.to_path_buf());
    let node = Node::from_item_span(item, input).with_children(tokes);
    Ok(node)
}

use crate::macros::Macros;

pub fn tokenize(ctx: &mut crate::ctx::Context) -> anyhow::Result<Node> {
    let mut macros = Macros::new();

    let parent = PathBuf::new();

    let mut all_tokens = vec![];
    let mut errors = UserErrors::new(ctx.max_errors);
    let files = ctx.files.clone();

    for file in files {
        let msg = format!("Reading {}", file.to_string_lossy());
        messages().status(msg);

        let res = tokenize_file(0, ctx, &file, &parent, &mut macros, &mut errors);

        match res {
            Err(e) => {
                if errors.has_errors() {
                    return Err(anyhow::Error::new(errors));
                } else {
                    return Err(e);
                }
            }

            Ok(node) => {
                all_tokens.push(node);
            }
        };
    }

    let block = Node::from_item(Item::Block, Position::default()).with_children(all_tokens);

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
