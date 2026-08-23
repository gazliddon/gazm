#![deny(unused_imports)]

use crate::cpukind::CpuKind;

use super::{
    err_nomatch, from_item_children_tspan, from_item_tspan, get_label_string, get_str, get_text,
    parse_expr, AstNodeKind, CommandKind, FeResult, FrontEndError, LabelDefinition, Node, PResult,
    TSpan, TokenKind, TokenKind::Comma,
};

use core::panic;
use std::{path::PathBuf, str::FromStr};

use unraveler::{
    alt, cut, many0, match_span as ms, opt, pair, preceded, sep_pair, tuple, Collection, Parser,
};

fn get_quoted_string(input: TSpan) -> PResult<String> {
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

    /// BSZ | ZMB | RZB count <value>
    pub(crate) fn parse_various_fills(input: TSpan) -> PResult<Node> {
        use CommandKind::*;
        let (rest, (sp, (a1, a2))) = ms(preceded(
            alt((Bsz, Zmb, Rzb)),
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

    pub(crate) fn parse_fcb(input: TSpan) -> PResult<Node> {
        let (rest, (sp, matched)) =
            ms(preceded(CommandKind::Fcb, cut(Self::parse_expr_list)))(input)?;
        let node = from_item_children_tspan(AstNodeKind::Fcb(matched.len()), &matched, sp);
        Ok((rest, node))
    }

    pub(crate) fn parse_fdb(input: TSpan) -> PResult<Node> {
        let (rest, (sp, matched)) = ms(preceded(CommandKind::Fdb, Self::parse_expr_list))(input)?;
        let node = from_item_children_tspan(AstNodeKind::Fdb(matched.len()), &matched, sp);
        Ok((rest, node))
    }

    pub(crate) fn parse_fcc(input: TSpan) -> PResult<Node> {
        let (rest, (sp, matched)) = ms(preceded(CommandKind::Fcc, get_quoted_string))(input)?;
        let node = from_item_tspan(AstNodeKind::Fcc(matched), sp);
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
                TokenKind::Identifier
                    | TokenKind::Label
                    | TokenKind::CpuOpcode(_)
                    | TokenKind::Command(_)
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
                    TokenKind::Identifier
                        | TokenKind::Label
                        | TokenKind::CpuOpcode(_)
                        | TokenKind::Command(_)
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
        let (rest, span_import) = TokenKind::Command(CommandKind::Import).parse(input)?;
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

    pub(crate) fn parse_rmb(_input: TSpan) -> PResult<Node> {
        Self::simple_command(CommandKind::Rmb, AstNodeKind::Rmb)(_input)
    }

    pub(crate) fn parse_rmd(_input: TSpan) -> PResult<Node> {
        Self::simple_command(CommandKind::Rmd, AstNodeKind::Rmd)(_input)
    }
    pub(crate) fn parse_zmd(_input: TSpan) -> PResult<Node> {
        Self::simple_command(CommandKind::Zmd, AstNodeKind::Zmd)(_input)
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
            Self::parse_various_fills,
            Self::parse_fill,
            Self::parse_fcb,
            Self::parse_fdb,
            Self::parse_fcc,
            Self::parse_zmd,
            Self::parse_rmb,
            Self::parse_rmd,
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

#[allow(unused_imports)]
#[cfg(test)]
mod test {
    use crate::{
        assembler::AssemblerCpuTrait,
        cli::parse_command_line,
        cpu6809::{frontend::NodeKind6809, Asm6809},
        frontend::{
            AstNodeKind::{self, *},
            ParsedFrom::*,
            *,
        },
        opts::Opts,
    };

    // pub type GParser = GazmParser<Asm6809>;

    use grl_sources::SourceFile;
    use pretty_assertions::{assert_eq, assert_ne};
    use thin_vec::ThinVec;
    use unraveler::{Collection, Parser};

    // fn test_command<P, C>(mut parser: P, text: &str, x: AstNodeKind<C>, xs: &[AstNodeKind<C>])
    // where
    //     P: for<'a> Parser<TSpan<'a>, Node<C::NodeKind>, FrontEndError>,
    //     C: AssemblerCpuTrait,
    // {
    //     println!("Parsing command - {text}");
    //     let opts = Opts::default();
    //     let sf = create_source_file(text);
    //     let tokens = to_tokens_no_comment(&sf);
    //     let span = make_tspan(&tokens, &sf, &opts);

    //     let tk: Vec<_> = tokens.iter().map(|t| t.kind).collect();
    //     println!("{:?}", tk);

    //     let check = |rest: TSpan, matched: Node| {
    //         let (rx, rxs) = get_items(&matched);
    //         println!("\t{:?} - {:?}", rx, rxs);
    //         assert_eq!(rx, x);
    //         assert_eq!(rxs, xs);
    //         assert!(rest.is_empty());
    //     };

    //     // Test the passed parser
    //     let (rest, matched) = parser.parse(span).unwrap();
    //     check(rest, matched);

    //     // test the command parser
    //     let (rest, matched) = GParser::parse_command(span).unwrap();
    //     check(rest, matched);
    // }

    // #[test]
    // fn test_parse_scope() {
    //     let text = "scope hello";
    //     let desired = AstNodeKind::Scope("hello".to_owned());
    //     let desired_args = [];
    //     test_command(GParser::parse_scope, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_put() {
    //     let text = "put 3 + 4";
    //     let desired = AstNodeKind::Put;
    //     let desired_args = [Expr];
    //     test_command(GParser::parse_put, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_writebin() {
    //     let text = "writebin \"out.bin\",0,10";
    //     let desired = AstNodeKind::WriteBin("out.bin".into());
    //     let desired_args = [Num(0, Decimal), Num(10, Decimal)];
    //     test_command(GParser::parse_writebin, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_incbin() {
    //     let text = "incbin \"a\", 10,10";
    //     let desired = AstNodeKind::IncBin("a".into());
    //     let desired_args = [Num(10, Decimal), Num(10, Decimal)];
    //     test_command(GParser::parse_incbin, text, desired, &desired_args);

    //     let text = "incbin \"a\"";
    //     let desired = AstNodeKind::IncBin("a".into());
    //     let desired_args = [];
    //     test_command(GParser::parse_incbin, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_incbin_ref() {
    //     let text = "incbinref \"a\", 10,20";
    //     let desired = AstNodeKind::IncBinRef("a".into());
    //     let desired_args = [Num(10, Decimal), Num(20, Decimal)];
    //     test_command(GParser::parse_incbin_ref, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_setdp() {
    //     let text = "setdp $ff00";
    //     let desired = AstNodeKind::CpuSpecific(NodeKind6809::SetDp);
    //     let desired_args = [Num(0xff00, Hexadecimal)];
    //     test_command(GParser::parse_setdp, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_various_fills() {
    //     let text = "rzb $ff00";
    //     let desired = AstNodeKind::Fill;
    //     let desired_args = [Num(0xff00, Hexadecimal), Num(0, Expression)];
    //     test_command(GParser::parse_various_fills, text, desired, &desired_args);

    //     let text = "rzb $ff00";
    //     let desired = AstNodeKind::Fill;
    //     let desired_args = [Num(0xff00, Hexadecimal), Num(0, Expression)];
    //     test_command(GParser::parse_various_fills, text, desired, &desired_args);

    //     let text = "bsz $ff00,0";
    //     let desired = AstNodeKind::Fill;
    //     let desired_args = [Num(0xff00, Hexadecimal), Num(0, Decimal)];
    //     test_command(GParser::parse_various_fills, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_fill() {
    //     let text = "fill 10,$ff00";
    //     let desired = AstNodeKind::Fill;
    //     let desired_args = [Num(10, Decimal), Num(0xff00, Hexadecimal)];
    //     test_command(GParser::parse_fill, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_fcb() {
    //     let text = "fcb $ff00,10";
    //     let desired = AstNodeKind::Fcb(2);
    //     let desired_args = [Num(0xff00, Hexadecimal), Num(10, Decimal)];
    //     test_command(GParser::parse_fcb, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_fdb() {
    //     let text = "fdb $ff00,10";
    //     let desired = AstNodeKind::Fdb(2);
    //     let desired_args = [Num(0xff00, Hexadecimal), Num(10, Decimal)];
    //     test_command(GParser::parse_fdb, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_fcc() {
    //     let text = "fcc \"Hello!\"";
    //     let desired = AstNodeKind::Fcc("Hello!".into());
    //     test_command(GParser::parse_fcc, text, desired, &[]);
    // }

    // #[test]
    // fn test_parse_zmd() {
    //     let text = "zmd $ff00";
    //     let desired = AstNodeKind::Zmd;
    //     let desired_args = [Num(0xff00, Hexadecimal)];
    //     test_command(GParser::parse_zmd, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_rmb() {
    //     let text = "rmb $ff00";
    //     let desired = AstNodeKind::Rmb;
    //     let desired_args = [Num(0xff00, Hexadecimal)];
    //     test_command(GParser::parse_rmb, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_org() {
    //     let text = "org $ff00";
    //     let desired = AstNodeKind::Org;
    //     let desired_args = [Num(0xff00, Hexadecimal)];
    //     test_command(GParser::parse_org, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_include() {
    //     let text = "include \"a\"";
    //     let desired = AstNodeKind::Include("a".into());
    //     let desired_args = [];
    //     test_command(GParser::parse_include, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_exec() {
    //     let text = "exec $ff00";
    //     let desired = AstNodeKind::Exec;
    //     let desired_args = [Num(0xff00, Hexadecimal)];
    //     test_command(GParser::parse_exec, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_require() {
    //     let text = "require \"a\"";
    //     let desired = AstNodeKind::Require("a".into());
    //     let desired_args = [];
    //     test_command(GParser::parse_require, text, desired, &desired_args);
    // }

    // #[test]
    // fn test_parse_import() {
    //     let text = "import ::xx::y";
    //     let desired = AstNodeKind::Import;
    //     let desired_args = [AstNodeKind::Label(LabelDefinition::TextScoped("::xx::y".into()))];
    //     test_command(GParser::parse_import, text, desired, &desired_args);
    // }
}
