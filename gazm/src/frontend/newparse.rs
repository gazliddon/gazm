use super::{get_identifier, CommandKind, GazmParser, Node, PResult, TSpan, TokenKind};

use crate::{cpu6800::frontend::parse_commands, cpukind::CpuKind};

impl GazmParser {
    // Parse a single assembly statement
    pub fn parse_assembly(cpu_kind: CpuKind, _input: TSpan) -> PResult<Node> {
        match cpu_kind {
            CpuKind::Cpu6809 => todo!(),
            CpuKind::Cpu6800 => todo!(),
            CpuKind::Cpu6502 => todo!(),
            CpuKind::Cpu65c02 => todo!(),
            CpuKind::CpuZ80 => todo!(),
        }
    }

    pub fn parse_command_args(_k: CommandKind, _args: TSpan) -> PResult<Node> {
        todo!()
    }

    pub fn parse_statement(input: TSpan) -> PResult<Node> {
        use TokenKind::*;
        let (rest, x) = get_identifier(input)?;

        match x {
            Command(cmd_kind) => Self::parse_command_args(cmd_kind, rest),
            OpCode(cpu_kind) => Self::parse_assembly(cpu_kind, input),
            Label => Self::parse_label(rest),
            _ => panic!("Whut?"),
        }
    }
}
