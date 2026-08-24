use super::{
    err_kind_nomatch, from_item_children_tspan, get_identifier, get_label_string, keyword,
    parse_block, parse_expr, AstNodeKind, CommandKind, GazmParser, Node, PResult, TSpan, TokenKind,
};
use unraveler::{many0, match_span as ms, opt, preceded, tuple, Collection, Parser};

use crate::{cpukind::CpuKind, frontend::FrontEndErrorKind};

impl GazmParser {
    fn consume_label_colon(input: TSpan) -> PResult<TSpan> {
        TokenKind::Colon.parse(input)
    }

    // Parse a single assembly statement
    pub fn parse_assembly(cpu_kind: CpuKind, input: TSpan) -> PResult<Node> {
        let (rest, nodes) = match cpu_kind {
            CpuKind::Cpu6809 => crate::cpu6809::frontend::parse_multi_opcode_vec(input),
            CpuKind::Cpu6800 => crate::cpu6800::frontend::parse_multi_opcode_vec(input),
            _ => {
                return Err(crate::frontend::error::FrontEndError::error(
                    input,
                    FrontEndErrorKind::Unexpected,
                ))
            }
        }?;
        if nodes.len() == 1 {
            Ok((rest, nodes.into_iter().next().unwrap()))
        } else {
            Ok((
                rest,
                crate::frontend::from_item_children_tspan(
                    crate::frontend::AstNodeKind::Block,
                    &nodes,
                    input,
                ),
            ))
        }
    }

    pub fn parse_command_args(k: CommandKind, args: TSpan) -> PResult<Node> {
        use CommandKind::*;
        match k {
            Scope => Self::parse_scope(args),
            Put => Self::parse_put(args),
            WriteBin => Self::parse_writebin(args),
            IncBin => Self::parse_incbin(args),
            IncBinRef => Self::parse_incbin_ref(args),
            Bsz | Zmb | Rzb => Self::parse_various_fills(args),
            Fill => Self::parse_fill(args),
            Fcb => Self::parse_fcb(args),
            Fdb => Self::parse_fdb(args),
            Fcc => Self::parse_fcc(args),
            Zmd => Self::parse_zmd(args),
            Rmb => Self::parse_rmb(args),
            Rmd => Self::parse_rmd(args),
            Org => Self::parse_org(args),
            Include => Self::parse_include(args),
            Exec => Self::parse_exec(args),
            Require => Self::parse_require(args),
            Import => Self::parse_import(args),
            GrabMem => Self::parse_grabmem(args),
            Section => Self::parse_section(args),
            SetDp => crate::cpu6809::frontend::parse_set_dp(args),
            Target | Macro | Struct | Equ => Err(crate::frontend::error::FrontEndError::error(
                args,
                crate::frontend::FrontEndErrorKind::Unexpected,
            )),
        }
    }

    /// `repeat <count> [, <index>] { body }`
    ///
    /// Count is an expression evaluated at assembly time; body children are
    /// assembled that many times. The optional `<index>` binds a loop
    /// variable (0-based) usable inside the body.
    ///
    /// New control-flow keywords (if/while/for/...) should follow this
    /// pattern: match by text via [`keyword`] from the statement-level
    /// alternatives in `parse_next_source_chunk`. Keywords are never
    /// reserved tokens, so they cannot collide with existing symbol names.
    pub fn parse_repeat(input: TSpan) -> PResult<Node> {
        // A line that starts with `repeat` but is not a valid repeat
        // construct (e.g. `REPEAT equ $C0`, `repeat:` as a label) must fall
        // through to the normal statement parser. The alternative machinery
        // aborts on fatal errors, so any failure here becomes a clean
        // NoMatch. This parser is only reached from the statement-level
        // alternatives, so that is always the right thing to do.
        let result = ms(preceded(
            keyword("repeat"),
            tuple((
                parse_expr,
                opt(preceded(TokenKind::Comma, get_label_string)),
                parse_block(many0(Self::parse_next_source_chunk)),
            )),
        ))(input);

        let (rest, (sp, (count, index, body))) = result.map_err(|_| err_kind_nomatch(input))?;

        let body: Vec<Node> = body.into_iter().flatten().collect();

        // First child is the count expression; the rest is the loop body.
        let mut children = Vec::with_capacity(1 + body.len());
        children.push(count);
        children.extend(body);

        let node = from_item_children_tspan(AstNodeKind::Repeat { index }, &children, sp);
        Ok((rest, node))
    }

    pub fn parse_statement(input: TSpan) -> PResult<Node> {
        use TokenKind::*;

        if matches!(
            input.first().map(|token| token.kind),
            Some(LocalIdentifier | Pling | At)
        ) {
            let (rest, _) = Self::parse_local_label(input)?;
            let (rest, _) = Self::consume_label_colon(rest)?;
            if keyword("equ")(rest.clone()).is_ok() {
                return Self::parse_equate(input);
            }
            return Self::parse_local_label(input).and_then(|(rest, node)| {
                let (rest, _) = Self::consume_label_colon(rest)?;
                Ok((rest, Self::mk_pc_equate(&node)))
            });
        }

        let (rest, x) = get_identifier(input)?;

        match x {
            // A command word is only a command when it is not immediately
            // followed by a label colon: `FDB: equ 1` defines a label named
            // FDB. Same for plain labels.
            Command(_) if matches!(rest.first().map(|token| token.kind), Some(Colon)) => {
                Self::parse_label_statement(input, rest)
            }
            Command(cmd_kind) => Self::parse_command_args(cmd_kind, input),
            CpuOpcode(cpu_kind) => Self::parse_assembly(cpu_kind, input),
            Label => Self::parse_label_statement(input, rest),
            _ => Err(crate::frontend::error::FrontEndError::error(
                input,
                crate::frontend::FrontEndErrorKind::Unexpected,
            )),
        }
    }

    /// Finish a statement that starts with a label definition: route to
    /// `parse_equate` when the label is followed by `equ`, to
    /// `parse_macro_call` when followed by an argument list, or produce a
    /// plain PC equate (label = current PC) otherwise.
    fn parse_label_statement<'a>(input: TSpan<'a>, rest: TSpan<'a>) -> PResult<'a, Node> {
        use TokenKind::*;

        let rest_after_label = if matches!(rest.first().map(|token| token.kind), Some(Colon)) {
            rest.drop(1).unwrap_or(rest)
        } else {
            rest
        };

        match rest_after_label.first().map(|token| token.kind) {
            Some(OpenBracket) => Self::parse_macro_call(input),
            _ if keyword("equ")(rest_after_label.clone()).is_ok() => Self::parse_equate(input),
            _ => Self::parse_label(input).and_then(|(rest, node)| {
                let (rest, _) = Self::consume_label_colon(rest)?;
                Ok((rest, Self::mk_pc_equate(&node)))
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        frontend::{create_source_file, make_tspan, to_tokens_no_comment, AstNodeKind},
        opts::Opts,
    };

    #[test]
    fn colon_labels_consume_colon() {
        let opts = Opts::default();
        let source = create_source_file("start: nop");
        let tokens = to_tokens_no_comment(&source);
        let span = make_tspan(&tokens, &source, &opts);

        let (rest, label) = GazmParser::parse_statement(span).unwrap();
        assert!(matches!(label.item, AstNodeKind::AssignmentFromPc(_)));
        let (rest, opcode) = GazmParser::parse_statement(rest).unwrap();
        assert!(matches!(opcode.item, AstNodeKind::TargetSpecific(_)));
        assert!(rest.is_empty());
    }

    #[test]
    fn colon_labels_support_equates() {
        let opts = Opts::default();
        let source = create_source_file("value: equ 1");
        let tokens = to_tokens_no_comment(&source);
        let span = make_tspan(&tokens, &source, &opts);

        let (rest, equate) = GazmParser::parse_statement(span).unwrap();
        assert!(matches!(equate.item, AstNodeKind::Assignment(_)));
        assert!(rest.is_empty());
    }

    #[test]
    fn opcodes_are_single_statement_lines() {
        let opts = Opts::default();
        let source = create_source_file("lda #1 : sta 0");
        let tokens = to_tokens_no_comment(&source);
        let span = make_tspan(&tokens, &source, &opts);

        let (rest, opcode) = GazmParser::parse_statement(span).unwrap();
        assert!(matches!(opcode.item, AstNodeKind::TargetSpecific(_)));
        assert!(matches!(
            rest.first().map(|token| token.kind),
            Some(TokenKind::Colon)
        ));
    }

    #[test]
    fn labels_require_colons() {
        let opts = Opts::default();
        let source = create_source_file("start nop");
        let tokens = to_tokens_no_comment(&source);
        let span = make_tspan(&tokens, &source, &opts);

        assert!(GazmParser::parse_statement(span).is_err());
    }
}
