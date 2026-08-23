use super::{get_identifier, CommandKind, GazmParser, Node, PResult, TSpan, TokenKind};
use unraveler::{Collection, Parser};

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

    pub fn parse_statement(input: TSpan) -> PResult<Node> {
        use TokenKind::*;

        if matches!(
            input.first().map(|token| token.kind),
            Some(LocalIdentifier | Pling | At)
        ) {
            let (rest, _) = Self::parse_local_label(input)?;
            let (rest, _) = Self::consume_label_colon(rest)?;
            if matches!(
                rest.first().map(|token| token.kind),
                Some(Command(CommandKind::Equ))
            ) {
                return Self::parse_equate(input);
            }
            return Self::parse_local_label(input).and_then(|(rest, node)| {
                let (rest, _) = Self::consume_label_colon(rest)?;
                Ok((rest, Self::mk_pc_equate(&node)))
            });
        }

        let (rest, x) = get_identifier(input)?;

        let rest_after_label = if matches!(rest.first().map(|token| token.kind), Some(Colon)) {
            rest.drop(1).unwrap_or(rest)
        } else {
            rest
        };

        match x {
            Command(cmd_kind) => Self::parse_command_args(cmd_kind, input),
            CpuOpcode(cpu_kind) => Self::parse_assembly(cpu_kind, input),
            Label => match rest_after_label.first().map(|token| token.kind) {
                Some(Command(CommandKind::Equ)) => Self::parse_equate(input),
                Some(OpenBracket) => Self::parse_macro_call(input),
                _ => Self::parse_label(input).and_then(|(rest, node)| {
                    let (rest, _) = Self::consume_label_colon(rest)?;
                    Ok((rest, Self::mk_pc_equate(&node)))
                }),
            },
            _ => Err(crate::frontend::error::FrontEndError::error(
                input,
                crate::frontend::FrontEndErrorKind::Unexpected,
            )),
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
