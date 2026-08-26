#![deny(unused_imports)]

use crate::cpukind::CpuKind;

use super::{
    err_nomatch, from_item_children_tspan, from_item_tspan, get_label_string, get_str, get_text,
    keyword, parse_expr, AstNodeKind, CommandKind, FeResult, FrontEndError, LabelDefinition, Node,
    PResult, TSpan, TokenKind, TokenKind::Comma,
};

use core::panic;
use std::{path::PathBuf, str::FromStr};

use unraveler::{
    alt, cut, many0, match_span as ms, opt, pair, preceded, sep_pair, tuple, Collection, Parser,
};

pub(crate) fn get_quoted_string(input: TSpan) -> PResult<String> {
    let (rest, matched) = TokenKind::QuotedString.parse(input)?;
    let txt = get_text(matched);
    let text = &txt[1..txt.len() - 1];
    Ok((rest, text.into()))
}

fn get_file_name(input: TSpan) -> PResult<PathBuf> {
    let (rest, matched) = TokenKind::QuotedString.parse(input)?;
    let txt = get_text(matched);
    let text = &txt[1..txt.len() - 1];
    let p = expand_path(matched, text.into())?;
    Ok((rest, p))
}

pub struct GazmParser {}

impl GazmParser {
    pub fn simple_command<I>(
        command_kind: CommandKind,
        item: I,
    ) -> impl for<'a> FnMut(TSpan<'a>) -> PResult<Node>
    where
        I: Into<AstNodeKind> + Clone,
    {
        move |i| Self::parse_simple_command(i, command_kind, item.clone().into())
    }

    fn parse_simple_command<I>(input: TSpan, command_kind: CommandKind, item: I) -> PResult<Node>
    where
        I: Into<AstNodeKind>,
    {
        let (rest, (sp, matched)) = ms(preceded(command_kind, parse_expr))(input)?;
        let node = from_item_children_tspan(item, &[matched], sp);

        Ok((rest, node))
    }
    pub(crate) fn parse_scope(input: TSpan) -> PResult<Node> {
        let (rest, (sp, name)) = ms(preceded(CommandKind::Scope, get_label_string))(input)?;
        Ok((rest, from_item_tspan(AstNodeKind::Scope(name), sp)))
    }
    pub(crate) fn parse_section(input: TSpan) -> PResult<Node> {
        let (rest, (sp, name)) = ms(preceded(
            CommandKind::Section,
            alt((get_label_string, get_quoted_string)),
        ))(input)?;

        let mut curr = rest;
        let mut children = Vec::new();

        while let Ok((next, _)) = TokenKind::Comma.parse(curr) {
            let (after_key, _) = opt(tuple((
                alt((get_label_string, get_quoted_string)),
                TokenKind::Equals,
            )))(next)?;

            let (next_expr, expr_node) = parse_expr(after_key)?;
            children.push(expr_node);
            curr = next_expr;
        }

        let node = from_item_children_tspan(AstNodeKind::Section(name), &children, sp);
        Ok((curr, node))
    }
    pub(crate) fn parse_require(input: TSpan) -> PResult<Node> {
        command_with_file(input, CommandKind::Require)
            .map(|(rest, (sp, file))| (rest, from_item_tspan(AstNodeKind::Require(file), sp)))
    }
    pub(crate) fn parse_include(input: TSpan) -> PResult<Node> {
        command_with_file(input, CommandKind::Include).and_then(|(rest, (sp, file))| {
            let path = expand_path(sp, file)?;
            Ok((rest, from_item_tspan(AstNodeKind::Include(path), sp)))
        })
    }

    pub(crate) fn parse_target(input: TSpan) -> PResult<Node> {
        let (rest, (sp, cpu)) = ms(preceded(CommandKind::Target, get_label_string))(input)?;
        let kind = CpuKind::from_str(&cpu).unwrap();
        Ok((rest, from_item_tspan(AstNodeKind::Cpu(kind), sp)))
    }

    /// FILL value,count
    pub(crate) fn parse_fill(input: TSpan) -> PResult<Node> {
        use CommandKind::*;
        let (rest, (sp, (value, count))) =
            ms(preceded(Fill, sep_pair(parse_expr, Comma, parse_expr)))(input)?;
        Ok((rest, Self::mk_fill(sp, (value, count))))
    }

    /// ZMB | BSZ | RZB count <value> — the zero-fill spellings.
    pub(crate) fn parse_zero_fills(input: TSpan) -> PResult<Node> {
        use CommandKind::*;
        let (rest, (sp, (a1, a2))) = ms(preceded(
            ZeroFill,
            pair(parse_expr, opt(preceded(Comma, parse_expr))),
        ))(input)?;

        let cv = (a1, a2.unwrap_or(Self::from_num_tspan(0, sp)));
        Ok((rest, Self::mk_fill(sp, cv)))
    }

    fn mk_fill(input: TSpan, cv: (Node, Node)) -> Node {
        from_item_children_tspan(AstNodeKind::Fill, &[cv.0, cv.1], input)
    }

    pub(crate) fn parse_grabmem(input: TSpan) -> PResult<Node> {
        let (rest, (sp, (src, size))) = ms(preceded(
            CommandKind::GrabMem,
            sep_pair(parse_expr, Comma, parse_expr),
        ))(input)?;
        let node = from_item_children_tspan(AstNodeKind::GrabMem, &[src, size], sp);
        Ok((rest, node))
    }

    // WRITEBIN "file",source_addr,size
    pub(crate) fn parse_writebin(input: TSpan) -> PResult<Node> {
        use TokenKind::*;
        let (rest, (sp, (file_name, _, source_addr, _, size))) = ms(preceded(
            CommandKind::WriteBin,
            tuple((get_file_name, Comma, parse_expr, Comma, parse_expr)),
        ))(input)?;

        let node =
            from_item_children_tspan(AstNodeKind::WriteBin(file_name), &[source_addr, size], sp);
        Ok((rest, node))
    }

    /// Parses for file with optional list of com sep expr
    fn incbin_args(_input: TSpan) -> PResult<(PathBuf, Vec<Node>)> {
        let (rest, (file, extra_args)) =
            tuple((get_file_name, many0(preceded(Comma, parse_expr))))(_input)?;
        Ok((rest, (file, extra_args)))
    }

    pub(crate) fn parse_incbin(input: TSpan) -> PResult<Node> {
        let (rest, (sp, (file, extra_args))) =
            ms(preceded(CommandKind::IncBin, Self::incbin_args))(input)?;
        let node = from_item_children_tspan(AstNodeKind::IncBin(file), &extra_args, sp);
        Ok((rest, node))
    }

    pub(crate) fn parse_incbin_ref(input: TSpan) -> PResult<Node> {
        let (rest, (sp, (file, extra_args))) =
            ms(preceded(CommandKind::IncBinRef, Self::incbin_args))(input)?;

        let num_of_args = extra_args.len();

        if num_of_args < 1 {
            panic!("Too few args for incbinref")
        } else if num_of_args > 2 {
            panic!("Too many args for incbinref")
        } else {
            let node = from_item_children_tspan(AstNodeKind::IncBinRef(file), &extra_args, sp);
            Ok((rest, node))
        }
    }

    pub(crate) fn parse_emit_bytes(input: TSpan) -> PResult<Node> {
        let (rest, (sp, matched)) =
            ms(preceded(CommandKind::EmitBytes, cut(Self::parse_expr_list)))(input)?;
        let node = from_item_children_tspan(AstNodeKind::EmitBytes(matched.len()), &matched, sp);
        Ok((rest, node))
    }

    pub(crate) fn parse_emit_words(input: TSpan) -> PResult<Node> {
        let (rest, (sp, matched)) =
            ms(preceded(CommandKind::EmitWords, Self::parse_expr_list))(input)?;
        let node = from_item_children_tspan(AstNodeKind::EmitWords(matched.len()), &matched, sp);
        Ok((rest, node))
    }

    pub(crate) fn parse_emit_string(input: TSpan) -> PResult<Node> {
        let (rest, (sp, matched)) =
            ms(preceded(CommandKind::EmitString, get_quoted_string))(input)?;
        let node = from_item_tspan(AstNodeKind::EmitString(matched), sp);
        Ok((rest, node))
    }

    fn parse_import_item(input: TSpan) -> PResult<Vec<Node>> {
        use TokenKind::{CloseBrace, Comma, DoubleColon, FqnIdentifier, OpenBrace};

        if input.is_empty() {
            return err_nomatch(input);
        }

        // Case 1: Brace-enclosed list without prefix: `{ A, B, C }`
        if let Ok((after_open, _)) = OpenBrace.parse(input) {
            let mut curr = after_open;
            let mut nodes = Vec::new();
            while !curr.is_empty() {
                if let Ok((after_close, _)) = CloseBrace.parse(curr) {
                    return Ok((after_close, nodes));
                }
                let (next_curr, node) = Self::parse_label(curr)?;
                nodes.push(node);
                curr = next_curr;

                if let Ok((after_comma, _)) = Comma.parse(curr) {
                    curr = after_comma;
                } else if let Ok((after_close, _)) = CloseBrace.parse(curr) {
                    return Ok((after_close, nodes));
                } else {
                    break;
                }
            }
            let (after_close, _) = CloseBrace.parse(curr)?;
            return Ok((after_close, nodes));
        }

        // Case 2: Grouped with prefix or path: `::core::{A, B}` or `core::{A, B}` or `::core::sub::{A, B}`
        // Or single `::core::GETOB` / `core::GETOB`
        let mut curr = input;
        let mut prefix = String::new();
        let mut had_prefix = false;

        if let Ok((after_dc, sp)) = DoubleColon.parse(curr) {
            prefix.push_str(get_str(&sp));
            curr = after_dc;
            had_prefix = true;
        }

        // Consume path segments separated by DoubleColon
        while let Some(first) = curr.first() {
            if !matches!(
                first.kind,
                TokenKind::Identifier | TokenKind::Label | TokenKind::CpuOpcode(_)
            ) {
                break;
            }
            let seg_span = curr.take(1).unwrap();
            let seg_text = get_str(&seg_span);
            let after_seg = curr.drop(1).unwrap();

            if let Ok((after_dc, dc_span)) = DoubleColon.parse(after_seg) {
                prefix.push_str(seg_text);
                prefix.push_str(get_str(&dc_span));
                curr = after_dc;
                had_prefix = true;
            } else {
                break;
            }
        }

        // If we have a prefix (e.g. `::core::` or `core::`) and the next token is `{`
        if had_prefix && !curr.is_empty() && curr.first().map(|t| t.kind) == Some(OpenBrace) {
            let (mut inner_curr, _) = OpenBrace.parse(curr)?;
            let mut nodes = Vec::new();

            while !inner_curr.is_empty() {
                if let Ok((after_close, _)) = CloseBrace.parse(inner_curr) {
                    return Ok((after_close, nodes));
                }
                if let Some(tok) = inner_curr.first() {
                    if tok.kind == CloseBrace {
                        let (after_close, _) = CloseBrace.parse(inner_curr)?;
                        return Ok((after_close, nodes));
                    }
                    let sym_span = inner_curr.take(1).unwrap();
                    let sym_name = get_str(&sym_span);
                    let full_name = format!("{prefix}{sym_name}");
                    let node = from_item_tspan(
                        AstNodeKind::Label(LabelDefinition::TextScoped(full_name)),
                        sym_span,
                    );
                    nodes.push(node);
                    inner_curr = inner_curr.drop(1).unwrap();

                    if let Ok((after_comma, _)) = Comma.parse(inner_curr) {
                        inner_curr = after_comma;
                    } else if let Ok((after_close, _)) = CloseBrace.parse(inner_curr) {
                        return Ok((after_close, nodes));
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            let (after_close, _) = CloseBrace.parse(inner_curr)?;
            return Ok((after_close, nodes));
        }

        // Case 3: Standard single scoped label / FqnIdentifier / label
        if let Ok((rest, sp)) = FqnIdentifier.parse(input) {
            let node = from_item_tspan(
                AstNodeKind::Label(LabelDefinition::TextScoped(get_text(sp))),
                sp,
            );
            return Ok((rest, vec![node]));
        }

        // If prefix was consumed and we have a final segment without `{`
        if had_prefix && !curr.is_empty() {
            if let Some(tok) = curr.first() {
                if matches!(
                    tok.kind,
                    TokenKind::Identifier | TokenKind::Label | TokenKind::CpuOpcode(_)
                ) {
                    let sym_span = curr.take(1).unwrap();
                    let sym_name = get_str(&sym_span);
                    let full_name = format!("{prefix}{sym_name}");
                    let span = input
                        .take(input.length() - curr.drop(1).unwrap().length())
                        .unwrap_or(sym_span);
                    let node = from_item_tspan(
                        AstNodeKind::Label(LabelDefinition::TextScoped(full_name)),
                        span,
                    );
                    return Ok((curr.drop(1).unwrap(), vec![node]));
                }
            }
        }

        let (rest, node) = Self::parse_label(input)?;
        Ok((rest, vec![node]))
    }

    pub(crate) fn parse_import(input: TSpan) -> PResult<Node> {
        let (rest, span_import) = keyword("import")(input)?;
        let mut rest = rest;
        let mut all_children = Vec::new();

        loop {
            let (next_rest, mut nodes) = Self::parse_import_item(rest)?;
            all_children.append(&mut nodes);
            rest = next_rest;

            if let Ok((after_comma, _)) = TokenKind::Comma.parse(rest) {
                if !after_comma.is_empty() {
                    rest = after_comma;
                    continue;
                }
            }
            break;
        }

        let full_span = input
            .take(input.length() - rest.length())
            .unwrap_or(span_import);
        let node = from_item_children_tspan(AstNodeKind::Import, &all_children, full_span);
        Ok((rest, node))
    }

    pub(crate) fn parse_org(_input: TSpan) -> PResult<Node> {
        Self::simple_command(CommandKind::Org, AstNodeKind::Org)(_input)
    }

    pub(crate) fn parse_put(_input: TSpan) -> PResult<Node> {
        Self::simple_command(CommandKind::Put, AstNodeKind::Put)(_input)
    }

    pub(crate) fn parse_reserve_bytes(_input: TSpan) -> PResult<Node> {
        Self::simple_command(CommandKind::ReserveBytes, AstNodeKind::ReserveBytes)(_input)
    }

    pub(crate) fn parse_zero_words(_input: TSpan) -> PResult<Node> {
        Self::simple_command(CommandKind::ZeroWords, AstNodeKind::ZeroWords)(_input)
    }

    pub(crate) fn parse_exec(_input: TSpan) -> PResult<Node> {
        Self::simple_command(CommandKind::Exec, AstNodeKind::Exec)(_input)
    }

    pub fn parse_command(_input: TSpan) -> PResult<Node> {
        let (rest, matched) = alt((
            Self::parse_scope,
            Self::parse_put,
            Self::parse_writebin,
            Self::parse_incbin,
            Self::parse_incbin_ref,
            Self::parse_zero_fills,
            Self::parse_fill,
            Self::parse_emit_bytes,
            Self::parse_emit_words,
            Self::parse_emit_string,
            Self::parse_zero_words,
            Self::parse_reserve_bytes,
            Self::parse_org,
            Self::parse_include,
            Self::parse_exec,
            Self::parse_require,
            Self::parse_import,
            Self::parse_grabmem,
        ))(_input)?;

        Ok((rest, matched))
    }
}

fn command_with_file(input: TSpan, ck: CommandKind) -> PResult<(TSpan, PathBuf)> {
    ms(preceded(ck, get_file_name))(input)
}

pub(crate) fn expand_path(sp: TSpan, file: PathBuf) -> FeResult<PathBuf> {
    let path = sp
        .extra()
        .opts
        .expand_path(file)
        .map_err(|e| FrontEndError::error(sp, e))?;
    Ok(path)
}
