/// Parse the assembler commands
use std::{
    path::{Path, PathBuf},
    process::CommandArgs,
};

use crate::{
    async_tokenize::IncludeErrorKind,
    cli::parse,
    error::IResult,
    item::{Item, LabelDefinition, Node, ParsedFrom},
    item6809::{IndexParseType, MC6809::SetDp},
    parse::locate::span_to_pos,
};

use thin_vec::{thin_vec, ThinVec};

use super::{
    parse_expr, parse_expr_list, to_pos, CommandKind, IdentifierKind, PResult, ParseText, TSpan,
    TokenKind,get_text
};
use IdentifierKind::Command;

use grl_sources::Position;
use unraveler::{
    all, alt, cut, is_a, many0, many1, many_until, not, opt, pair, preceded, sep_pair, succeeded,
    tuple, until, wrapped_cut, Collection, ParseError, ParseErrorKind, Parser, Severity,
};

use super::match_span as ms;

// pub fn command(command_kind: CommandKind) -> impl for<'a> FnMut(TSpan<'a>) -> PResult<TSpan> {
//     use IdentifierKind::*;
//     use TokenKind::*;
//     move |i| tag(Identifier(Command(command_kind.clone())))(i)
// }

fn get_quoted_string(input: TSpan) -> PResult<String> {
    let (rest, matched) = TokenKind::QuotedString.parse(input)?;
    Ok((rest, get_text(matched)))
}

fn get_file_name(input: TSpan) -> PResult<PathBuf> {
    let (rest, text) = get_quoted_string(input)?;
    Ok((rest, PathBuf::from(text)))
}

fn get_identifier(input: TSpan) -> PResult<String> {
    let (rest, matched) = TokenKind::Identifier(IdentifierKind::Label).parse(input)?;
    Ok((rest, get_text(matched)))
}

fn simple_command<I>(
    command_kind: CommandKind,
    item: I,
) -> impl for<'a> FnMut(TSpan<'a>) -> PResult<Node>
where
    I: Into<Item> + Clone,
{
    move |i| parse_simple_command(i, command_kind, item.clone().into())
}

fn parse_simple_command<I: Into<Item>>(
    input: TSpan,
    command_kind: CommandKind,
    item: I,
) -> PResult<Node> {
    let (rest, matched) = preceded(command_kind, parse_expr)(input)?;
    let node = Node::new_with_children(item.into(), &[matched], to_pos(input));
    Ok((rest, node))
}

fn parse_scope(input: TSpan) -> PResult<Node> {
    let (rest, name) = preceded(CommandKind::Scope, get_identifier)(input)?;
    let node = Node::new(Item::Scope(name), to_pos(input));
    Ok((rest, node))
}

fn to_ast(_tokes: &[TokenKind], _txt: &str) {}

fn command_with_file(input: TSpan, ck: CommandKind) -> PResult<PathBuf> {
    preceded(ck, get_file_name)(input)
}

fn parse_require(input: TSpan) -> PResult<Node> {
    command_with_file(input, CommandKind::Require)
        .map(|(rest, matched)| (rest, Node::new(Item::Require(matched), to_pos(input))))
}

fn parse_include(input: TSpan) -> PResult<Node> {
    command_with_file(input, CommandKind::Require)
        .map(|(rest, matched)| (rest, Node::new(Item::Include(matched), to_pos(input))))
}

fn parse_bsz(input: TSpan) -> PResult<Node> {
    let (rest, (a1, a2)) = preceded(
        CommandKind::Bsz,
        pair(parse_expr, opt(preceded(TokenKind::Comma, parse_expr))),
    )(input)?;

    let zero = Node::from_number_pos(0, to_pos(input));
    let cv = (a1, a2.unwrap_or(zero));
    Ok((rest, mk_fill(input, cv)))
}

fn mk_fill(input: TSpan, cv: (Node, Node)) -> Node {
    Node::new_with_children(Item::Fill, &vec![cv.0, cv.1], to_pos(input))
}

fn parse_grabmem(input: TSpan) -> PResult<Node> {
    let (rest, (src, size)) = preceded(
        CommandKind::GrabMem,
        sep_pair(parse_expr, TokenKind::Comma, parse_expr),
    )(input)?;
    let node = Node::new_with_children(Item::GrabMem, &vec![src, size], to_pos(input));
    Ok((rest, node))
}

fn parse_fill(input: TSpan) -> PResult<Node> {
    let (rest, matched) = sep_pair(parse_expr, TokenKind::Comma, parse_expr)(input)?;
    Ok((rest, mk_fill(input, matched)))
}

fn parse_writebin(input: TSpan) -> PResult<Node> {
    use TokenKind::*;
    let (rest, (sp, (file_name, _, source_addr, _, size))) =
        ms(tuple((get_file_name, Comma, parse_expr, Comma, parse_expr)))(input)?;

    let node = Node::new_with_children(
        Item::WriteBin(file_name),
        &vec![source_addr, size],
        to_pos(sp),
    );

    Ok((rest, node))
}

/// Parses for file with optional list of com sep expr
fn incbin_args(_input: TSpan) -> PResult<(PathBuf, Vec<Node>)> {
    use TokenKind::Comma;
    let (rest, (file, extra_args)) =
        tuple((get_file_name, many0(preceded(Comma, parse_expr))))(_input)?;

    Ok((rest, (file, extra_args)))
}

fn parse_incbin(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (file, extra_args))) = ms(preceded(CommandKind::IncBin, incbin_args))(input)?;

    let node = Node::new_with_children(Item::IncBin(file), &extra_args, to_pos(sp));
    Ok((rest, node))
}
fn parse_incbin_ref(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (file, extra_args))) =
        ms(preceded(CommandKind::IncBinRef, incbin_args))(input)?;
    let node = Node::new_with_children(Item::IncBinRef(file), &extra_args, to_pos(sp));
    Ok((rest, node))
}

fn parse_fcb(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(preceded(CommandKind::Fcb, parse_expr_list))(input)?;
    let node = Node::new_with_children(Item::Fcb(matched.len()), &matched, to_pos(sp));
    Ok((rest, node))
}

fn parse_fdb(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(preceded(CommandKind::Fdb, parse_expr_list))(input)?;
    let node = Node::new_with_children(Item::Fdb(matched.len()), &matched, to_pos(sp));
    Ok((rest, node))
}

fn parse_fcc(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(preceded(CommandKind::Fcc, get_quoted_string))(input)?;
    let node = Node::new(Item::Fcc(matched), to_pos(sp));
    Ok((rest, node))
}

fn parse_import(_input: TSpan) -> PResult<Node> {
    todo!()
}

pub fn parse_commands(input: TSpan) -> PResult<Node> {
    use crate::item6809::MC6809;
    use CommandKind::*;

    let parse_zmd = simple_command(Zmd, Item::Zmd);
    let parse_zmb = simple_command(Zmb, Item::Zmb);
    let parse_put = simple_command(Put, Item::Put);
    let parse_org = simple_command(Org, Item::Org);
    let parse_exec = simple_command(Exec, Item::Exec);
    let parse_setdp = simple_command(SetDp, MC6809::SetDp);
    let parse_rmb = simple_command(Rmb, Item::Rmb);

    let (rest, matched) = alt((
        parse_scope,
        parse_put,
        parse_writebin,
        parse_incbin,
        parse_incbin_ref,
        parse_setdp,
        parse_bsz,
        parse_fill,
        parse_fcb,
        parse_fdb,
        parse_fcc,
        parse_zmb,
        parse_zmd,
        parse_rmb,
        parse_org,
        parse_include,
        parse_exec,
        parse_require,
        parse_import,
    ))(input)?;

    Ok((rest, matched))
}
