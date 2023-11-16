#![deny(unused_imports)]

use super::{
    get_text, item6809::MC6809, parse_expr, parse_expr_list, parse_scoped_label, CommandKind,
    IdentifierKind, Item, Node, PResult, TSpan, TokenKind, TokenKind::Comma,
};
use crate::debug_mess;
use std::path::PathBuf;
use unraveler::{alt, many0, match_span as ms, opt, pair, preceded, sep_pair, tuple, Parser};

fn get_quoted_string(input: TSpan) -> PResult<String> {
    let (rest, matched) = TokenKind::QuotedString.parse(input)?;
    let txt = get_text(matched);
    let text = &txt[1..txt.len() - 1];
    Ok((rest, text.into()))
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
    let (rest, (sp, matched)) = ms(preceded(command_kind, parse_expr))(input)?;
    let node = Node::from_item_kids_tspan(item.into(), &[matched], sp);
    Ok((rest, node))
}

pub(crate) fn parse_scope(input: TSpan) -> PResult<Node> {
    let (rest, (sp, name)) = ms(preceded(CommandKind::Scope, get_identifier))(input)?;
    Ok((rest, Node::from_item_tspan(Item::Scope(name), sp)))
}

fn to_ast(_tokes: &[TokenKind], _txt: &str) {}

fn command_with_file(input: TSpan, ck: CommandKind) -> PResult<(TSpan, PathBuf)> {
    ms(preceded(ck, get_file_name))(input)
}

pub(crate) fn parse_require(input: TSpan) -> PResult<Node> {
    command_with_file(input, CommandKind::Require)
        .map(|(rest, (sp, file))| (rest, Node::from_item_tspan(Item::Require(file), sp)))
}

pub(crate) fn parse_include(input: TSpan) -> PResult<Node> {
    command_with_file(input, CommandKind::Include)
        .map(|(rest, (sp, file))| (rest, Node::from_item_tspan(Item::Include(file), sp)))
}

/// FILL value,count
pub(crate) fn parse_fill(input: TSpan) -> PResult<Node> {
    use CommandKind::*;
    let (rest, (sp, (value, count))) =
        ms(preceded(Fill, sep_pair(parse_expr, Comma, parse_expr)))(input)?;
    Ok((rest, mk_fill(sp, (value, count))))
}

/// BSZ | ZMB | RZB count <value>
pub(crate) fn parse_various_fills(input: TSpan) -> PResult<Node> {
    use CommandKind::*;
    let (rest, (sp, (a1, a2))) = ms(preceded(
        alt((Bsz, Zmb, Rzb)),
        pair(parse_expr, opt(preceded(Comma, parse_expr))),
    ))(input)?;

    let cv = (a1, a2.unwrap_or(Node::from_num_tspan(0, sp)));
    Ok((rest, mk_fill(sp, cv)))
}

fn mk_fill(input: TSpan, cv: (Node, Node)) -> Node {
    Node::from_item_kids_tspan(Item::Fill, &vec![cv.0, cv.1], input)
}

pub(crate) fn parse_grabmem(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (src, size))) = ms(preceded(
        CommandKind::GrabMem,
        sep_pair(parse_expr, Comma, parse_expr),
    ))(input)?;
    let node = Node::from_item_kids_tspan(Item::GrabMem, &[src, size], sp);
    Ok((rest, node))
}

// WRITEBIN "file",source_addr,size
pub(crate) fn parse_writebin(input: TSpan) -> PResult<Node> {
    use TokenKind::*;
    let (rest, (sp, (file_name, _, source_addr, _, size))) = ms(preceded(
        CommandKind::WriteBin,
        tuple((get_file_name, Comma, parse_expr, Comma, parse_expr)),
    ))(input)?;

    let node = Node::from_item_kids_tspan(Item::WriteBin(file_name), &[source_addr, size], sp);
    Ok((rest, node))
}

/// Parses for file with optional list of com sep expr
fn incbin_args(_input: TSpan) -> PResult<(PathBuf, Vec<Node>)> {
    let (rest, (file, extra_args)) =
        tuple((get_file_name, many0(preceded(Comma, parse_expr))))(_input)?;
    Ok((rest, (file, extra_args)))
}

pub(crate) fn parse_incbin(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (file, extra_args))) = ms(preceded(CommandKind::IncBin, incbin_args))(input)?;
    let node = Node::from_item_kids_tspan(Item::IncBin(file), &extra_args, sp);
    Ok((rest, node))
}
pub(crate) fn parse_incbin_ref(input: TSpan) -> PResult<Node> {
    let (rest, (sp, (file, extra_args))) =
        ms(preceded(CommandKind::IncBinRef, incbin_args))(input)?;
    let node = Node::from_item_kids_tspan(Item::IncBinRef(file), &extra_args, sp);
    Ok((rest, node))
}

pub(crate) fn parse_fcb(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(preceded(CommandKind::Fcb, parse_expr_list))(input)?;
    let node = Node::from_item_kids_tspan(Item::Fcb(matched.len()), &matched, sp);
    Ok((rest, node))
}

pub(crate) fn parse_fdb(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(preceded(CommandKind::Fdb, parse_expr_list))(input)?;
    let node = Node::from_item_kids_tspan(Item::Fdb(matched.len()), &matched, sp);
    Ok((rest, node))
}

pub(crate) fn parse_fcc(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(preceded(CommandKind::Fcc, get_quoted_string))(input)?;
    let node = Node::from_item_tspan(Item::Fcc(matched), sp);
    Ok((rest, node))
}

pub(crate) fn parse_import(input: TSpan) -> PResult<Node> {
    let (rest, (sp, matched)) = ms(preceded(CommandKind::Import, parse_scoped_label))(input)?;
    let node = Node::from_item_kids_tspan(Item::Import, &[matched], sp);
    Ok((rest, node))
}

pub(crate) fn parse_org(_input: TSpan) -> PResult<Node> {
    simple_command(CommandKind::Org, Item::Org)(_input)
}

pub(crate) fn parse_put(_input: TSpan) -> PResult<Node> {
    simple_command(CommandKind::Put, Item::Put)(_input)
}
pub(crate) fn parse_setdp(_input: TSpan) -> PResult<Node> {
    simple_command(CommandKind::SetDp, MC6809::SetDp)(_input)
}
pub(crate) fn parse_rmb(_input: TSpan) -> PResult<Node> {
    simple_command(CommandKind::Rmb, Item::Rmb)(_input)
}

pub(crate) fn parse_rmd(_input: TSpan) -> PResult<Node> {
    simple_command(CommandKind::Rmd, Item::Rmd)(_input)
}
pub(crate) fn parse_zmd(_input: TSpan) -> PResult<Node> {
    simple_command(CommandKind::Zmd, Item::Zmd)(_input)
}

pub(crate) fn parse_exec(_input: TSpan) -> PResult<Node> {
    simple_command(CommandKind::Exec, Item::Exec)(_input)
}

pub fn parse_command(input: TSpan) -> PResult<Node> {
    let (rest, matched) = alt((
        parse_scope,
        parse_put,
        parse_writebin,
        parse_incbin,
        parse_incbin_ref,
        parse_setdp,
        parse_various_fills,
        parse_fill,
        parse_fcb,
        parse_fdb,
        parse_fcc,
        parse_zmd,
        parse_rmb,
        parse_rmd,
        parse_org,
        parse_include,
        parse_exec,
        parse_require,
        parse_import,
    ))(input)?;

    debug_mess!("Parse command: {:?}", matched.item);

    Ok((rest, matched))
}

#[allow(unused_imports)]
#[cfg(test)]
mod test {
    use crate::{
        cli::parse_command_line,
        frontend::{*,
            item6809::MC6809,
            Item::{self, *},
            ParsedFrom::*,
        },
    };
    use grl_sources::SourceFile;
    use pretty_assertions::{assert_eq, assert_ne};
    use thin_vec::ThinVec;
    use unraveler::{Collection, Parser};

    fn test_command<P>(mut parser: P, text: &str, x: Item, xs: &[Item])
    where
        P: for<'a> Parser<TSpan<'a>, Node, FrontEndError>,
    {
        println!("Parsing command - {text}");
        let sf = create_source_file(text);
        let tokens = to_tokens_no_comment(&sf);
        let span = make_tspan(&tokens, &sf);

        let tk: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        println!("{:?}", tk);

        let check = |rest: TSpan, matched: Node| {
            let (rx, rxs) = get_items(&matched);
            println!("\t{:?} - {:?}", rx, rxs);
            assert_eq!(rx, x);
            assert_eq!(rxs, xs);
            assert!(rest.is_empty());
        };

        // Test the passed parser
        let (rest, matched) = parser.parse(span).unwrap();
        check(rest, matched);

        // test the command parser
        let (rest, matched) = parse_command(span).unwrap();
        check(rest, matched);
    }

    #[test]
    fn test_parse_scope() {
        let text = "scope hello";
        let desired = Item::Scope("hello".to_owned());
        let desired_args = [];
        test_command(parse_scope, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_put() {
        let text = "put 3 + 4";
        let desired = Item::Put;
        let desired_args = [Expr];
        test_command(parse_put, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_writebin() {
        let text = "writebin \"out.bin\",0,10";
        let desired = Item::WriteBin("out.bin".into());
        let desired_args = [Num(0, Dec), Num(10, Dec)];
        test_command(parse_writebin, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_incbin() {
        let text = "incbin \"a\", 10,10";
        let desired = Item::IncBin("a".into());
        let desired_args = [Num(10, Dec), Num(10, Dec)];
        test_command(parse_incbin, text, desired, &desired_args);

        let text = "incbin \"a\"";
        let desired = Item::IncBin("a".into());
        let desired_args = [];
        test_command(parse_incbin, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_incbin_ref() {
        let text = "incbinref \"a\", 10,20";
        let desired = Item::IncBinRef("a".into());
        let desired_args = [Num(10, Dec), Num(20, Dec)];
        test_command(super::parse_incbin_ref, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_setdp() {
        let text = "setdp $ff00";
        let desired = Item::Cpu(MC6809::SetDp);
        let desired_args = [Num(0xff00, Hex)];
        test_command(super::parse_setdp, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_various_fills() {
        let text = "rzb $ff00";
        let desired = Item::Fill;
        let desired_args = [Num(0xff00, Hex), Num(0, FromExpr)];
        test_command(super::parse_various_fills, text, desired, &desired_args);

        let text = "rzb $ff00";
        let desired = Item::Fill;
        let desired_args = [Num(0xff00, Hex), Num(0, FromExpr)];
        test_command(super::parse_various_fills, text, desired, &desired_args);

        let text = "bsz $ff00,0";
        let desired = Item::Fill;
        let desired_args = [Num(0xff00, Hex), Num(0, Dec)];
        test_command(super::parse_various_fills, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_fill() {
        let text = "fill 10,$ff00";
        let desired = Item::Fill;
        let desired_args = [Num(10, Dec),Num(0xff00, Hex), ];
        test_command(super::parse_fill, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_fcb() {
        let text = "fcb $ff00,10";
        let desired = Item::Fcb(2);
        let desired_args = [Num(0xff00, Hex), Num(10, Dec)];
        test_command(super::parse_fcb, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_fdb() {
        let text = "fdb $ff00,10";
        let desired = Item::Fdb(2);
        let desired_args = [Num(0xff00, Hex), Num(10, Dec)];
        test_command(super::parse_fdb, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_fcc() {
        let text = "fcc \"Hello!\"";
        let desired = Item::Fcc("Hello!".into());
        test_command(super::parse_fcc, text, desired, &[]);
    }

    #[test]
    fn test_parse_zmd() {
        let text = "zmd $ff00";
        let desired = Item::Zmd;
        let desired_args = [Num(0xff00, Hex)];
        test_command(super::parse_zmd, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_rmb() {
        let text = "rmb $ff00";
        let desired = Item::Rmb;
        let desired_args = [Num(0xff00, Hex)];
        test_command(super::parse_rmb, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_org() {
        let text = "org $ff00";
        let desired = Item::Org;
        let desired_args = [Num(0xff00, Hex)];
        test_command(super::parse_org, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_include() {
        let text = "include \"a\"";
        let desired = Item::Include("a".into());
        let desired_args = [];
        test_command(parse_include, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_exec() {
        let text = "exec $ff00";
        let desired = Item::Exec;
        let desired_args = [Num(0xff00, Hex)];
        test_command(super::parse_exec, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_require() {
        let text = "require \"a\"";
        let desired = Item::Require("a".into());
        let desired_args = [];
        test_command(parse_require, text, desired, &desired_args);
    }

    #[test]
    fn test_parse_import() {
        let text = "import ::xx::y";
        let desired = Item::Import;
        let desired_args = [Item::Label(LabelDefinition::TextScoped("::xx::y".into()))];
        test_command(parse_import, text, desired, &desired_args);
    }
}
