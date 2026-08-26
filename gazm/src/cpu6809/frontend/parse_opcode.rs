#![deny(unused_imports)]
use crate::cpu6809::frontend::NodeKind6809;
use crate::frontend::{
    err_fatal, from_item_tspan, parse_expr, AstNodeKind, CpuSpecific, Node, PResult, TSpan,
    TokenKind,
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
use unraveler::{alt, match_span as ms, preceded, Collection};

pub fn get_opcode_info(id: InstructionId) -> Option<&'static InstructionInfo> {
    ISA_DBASE.get_info_by_id(id)
}

fn parse_immediate(_input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    use TokenKind::Hash;
    let (rest, (sp, matched)) = ms(preceded(Hash, parse_expr))(_input)?;
    let node = from_item_tspan(Immediate, sp).with_child(matched);
    Ok((rest, node))
}

fn parse_force_dp(_input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    use TokenKind::LessThan;
    let (rest, (sp, matched)) = ms(preceded(LessThan, parse_expr))(_input)?;
    let node = from_item_tspan(Direct, sp).with_child(matched);
    Ok((rest, node))
}

fn parse_force_extended(_input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    use TokenKind::GreaterThan;
    let (rest, (sp, matched)) = ms(preceded(GreaterThan, parse_expr))(_input)?;
    let node = from_item_tspan(Extended(true), sp).with_child(matched);
    Ok((rest, node))
}

fn parse_extended(_input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    let (rest, (sp, matched)) = ms(parse_expr)(_input)?;
    let node = from_item_tspan(Extended(false), sp).with_child(matched);
    Ok((rest, node))
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
    let (rest, arg) = if info.supports_addr_mode(AddrModeEnum::RegisterSet) {
        parse_reg_set_operand(rest)
    } else if info.supports_addr_mode(AddrModeEnum::RegisterPair) {
        parse_opcode_reg_pair(rest)
    } else {
        parse_opcode_arg(rest)
    }?;

    let amode = match arg.item {
        AstNodeKind::TargetSpecific(CpuSpecific::Cpu6809(Operand(amode))) => amode,
        AstNodeKind::TargetSpecific(CpuSpecific::Cpu6809(OperandIndexed(amode, indirect))) => {
            AddrModeParseType::Indexed(amode, indirect)
        }
        _ => return err_fatal(sp, Cpu6809AssemblyErrorKind::AddrModeUnsupported),
    };

    if let Some(instruction) = get_instruction(amode, info) {
        let item = OpCode(instruction.id(), amode);
        let node = from_item_tspan(item, sp).take_others_children(arg);
        Ok((rest, node))
    } else {
        err_fatal(sp, Cpu6809AssemblyErrorKind::ThisAddrModeUnsupported(amode))
    }
}

fn parse_opcode_no_arg_parts<'a>(
    rest: TSpan<'a>,
    sp: TSpan<'a>,
    ins: &'a InstructionInfo,
) -> PResult<'a, Node> {
    use Cpu6809AssemblyErrorKind::OnlySupports;

    if let Some(ins) = ins.get_instruction_id(AddrModeEnum::Inherent) {
        let oc = NodeKind6809::OpCode(ins, ParseInherent);
        let node = from_item_tspan(oc, sp);
        Ok((rest, node))
    } else {
        err_fatal(sp, OnlySupports(AddrModeParseType::Inherent))
    }
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
    let info = ISA_DBASE.get_opcode(text.as_str()).unwrap();
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
