#![deny(unused_imports)]
use crate::frontend::{
    err_kind_nomatch, fatal, parse_inherent, parse_opcode_operand, parse_prefixed_operand,
    AstNodeKind, CpuSpecific, FrontEndError, Node, PResult, TSpan, TokenKind,
};

use crate::cpu6809::assembler::ISA_DBASE;
use crate::cpukind::CpuKind;

use super::{
    parse_indexed, parse_opcode_reg_pair, parse_reg_set_operand, AddrModeParseType,
    AddrModeParseType::Inherent as ParseInherent,
    Cpu6809AssemblyErrorKind,
    NodeKind6809::{OpCode, Operand, OperandIndexed},
};

use emu6809::isa::{AddrModeEnum, Instruction, InstructionId, InstructionInfo};
use unraveler::{alt, match_span as ms, Collection};

pub fn get_opcode_info(id: InstructionId) -> Option<&'static InstructionInfo> {
    ISA_DBASE.get_info_by_id(id)
}

fn parse_immediate(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    parse_prefixed_operand(input, Some(TokenKind::Hash), Immediate)
}

fn parse_force_dp(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    parse_prefixed_operand(input, Some(TokenKind::LessThan), Direct)
}

fn parse_force_extended(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    parse_prefixed_operand(input, Some(TokenKind::GreaterThan), Extended(true))
}

fn parse_extended(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    parse_prefixed_operand(input, None, Extended(false))
}

fn parse_opcode_arg(input: TSpan) -> PResult<Node> {
    use TokenKind::{GreaterThan, Hash, LessThan, OpenSquareBracket};

    // A comma cannot occur in a 6809 expression, so its presence identifies
    // indexed syntax, including width-qualified offsets such as `<<5,u`.
    let indexed = matches!(
        input.first().map(|token| token.kind),
        Some(OpenSquareBracket)
    ) || input.iter().any(|token| token.kind == TokenKind::Comma);

    match input.first().map(|token| token.kind) {
        Some(Hash) => parse_immediate(input),
        _ if indexed => parse_indexed(input),
        Some(LessThan) => parse_force_dp(input),
        Some(GreaterThan) => parse_force_extended(input),
        _ => parse_extended(input),
    }
}

fn parse_opcode_with_arg_parts<'a>(
    rest: TSpan<'a>,
    sp: TSpan<'a>,
    info: &'a InstructionInfo,
) -> PResult<'a, Node> {
    let parse_arg = |rest| {
        if info.supports_addr_mode(AddrModeEnum::RegisterSet) {
            parse_reg_set_operand(rest)
        } else if info.supports_addr_mode(AddrModeEnum::RegisterPair) {
            parse_opcode_reg_pair(rest)
        } else {
            parse_opcode_arg(rest)
        }
    };

    let resolve = |arg: &Node| -> Result<(AddrModeParseType, InstructionId), FrontEndError> {
        let amode = match &arg.item {
            AstNodeKind::TargetSpecific(CpuSpecific::Cpu6809(Operand(amode))) => *amode,
            AstNodeKind::TargetSpecific(CpuSpecific::Cpu6809(OperandIndexed(amode, indirect))) => {
                AddrModeParseType::Indexed(*amode, *indirect)
            }
            _ => return Err(fatal(sp, Cpu6809AssemblyErrorKind::AddrModeUnsupported)),
        };

        match get_instruction(amode, info) {
            Some(instruction) => Ok((amode, instruction.id())),
            None => Err(fatal(
                sp,
                Cpu6809AssemblyErrorKind::ThisAddrModeUnsupported(amode),
            )),
        }
    };

    parse_opcode_operand(rest, sp, parse_arg, resolve, |amode, ins| {
        OpCode(ins, amode).into()
    })
}

fn parse_opcode_no_arg_parts<'a>(
    rest: TSpan<'a>,
    sp: TSpan<'a>,
    ins: &'a InstructionInfo,
) -> PResult<'a, Node> {
    parse_inherent(
        rest,
        sp,
        || ins.get_instruction_id(AddrModeEnum::Inherent),
        |ins| OpCode(ins, ParseInherent).into(),
        Cpu6809AssemblyErrorKind::OnlySupports(AddrModeParseType::Inherent),
    )
}

pub fn parse_opcode(input: TSpan) -> PResult<Node> {
    let (rest, (sp, info)) = get_opcode(input)?;
    if rest.is_empty() {
        parse_opcode_no_arg_parts(rest, sp, info)
    } else {
        parse_opcode_with_arg_parts(rest, sp, info)
    }
}

pub fn parse_multi_opcode_vec(input: TSpan) -> PResult<Vec<Node>> {
    let (rest, opcode) = parse_opcode(input)?;
    Ok((rest, vec![opcode]))
}

fn get_opcode(input: TSpan<'_>) -> PResult<'_, (TSpan<'_>, &InstructionInfo)> {
    use TokenKind::{CpuOpcode, Identifier};
    let (rest, (sp, matched)) = ms(alt((CpuOpcode(CpuKind::Cpu6809), Identifier)))(input)?;
    let text = crate::frontend::get_text(matched);
    let info = ISA_DBASE
        .get_opcode(text.as_str())
        .ok_or(err_kind_nomatch(sp))?;
    Ok((rest, (sp, info)))
}

fn get_instruction(amode: AddrModeParseType, info: &InstructionInfo) -> Option<&Instruction> {
    use AddrModeEnum::*;
    let get = |amode| info.get_instruction(&amode);

    match amode {
        AddrModeParseType::Indexed(..) => get(Indexed),

        AddrModeParseType::Direct => get(Direct),

        AddrModeParseType::Extended(_) => get(Extended)
            .or_else(|| get(Relative))
            .or_else(|| get(Relative16)),

        AddrModeParseType::Relative => get(Relative).or_else(|| get(Relative16)),

        AddrModeParseType::Inherent => get(Inherent),

        AddrModeParseType::Immediate => get(Immediate8).or_else(|| get(Immediate16)),
        AddrModeParseType::RegisterPair(..) => get(RegisterPair),

        AddrModeParseType::RegisterSet => get(RegisterSet),
    }
}
