use crate::frontend::{
    err_fatal, err_kind_nomatch, err_nomatch, from_item_tspan, get_text, is_parsing_macro_def,
    parse_expr, AstNodeKind, FrontEndError, FrontEndErrorKind, Node, PResult, TSpan, TokenKind,
};

use crate::cpukind::CpuKind::Cpu6800 as Cpu;

use crate::cpu6800::{
    frontend::{
        error::AssemblyErrorKind6800::OnlySupports,
        get_this_reg, AddrModeParseType,
        NodeKind6800::{self, OpCode, Operand},
    },
    Asm6800,
};

use emu6800::cpu_core::{AddrModeEnum, Instruction, InstructionInfo, OpcodeData, RegEnum, DBASE};

use serde_json::value::Index;
use unraveler::{alt, match_span as ms, opt, preceded, sep_pair, tag, Collection};

fn get_opcode(input: TSpan<'_>) -> PResult<'_, (TSpan<'_>, &Instruction)> {
    let (rest, (sp, matched)) = ms(alt((TokenKind::CpuOpcode(Cpu), TokenKind::Identifier)))(input)?;
    let text = get_text(matched).to_lowercase();
    let info = DBASE
        .get_opcode(text.as_str())
        .ok_or(err_kind_nomatch(sp))?;
    Ok((rest, (sp, info)))
}

fn parse_indexed(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    use TokenKind::Comma;
    let (rest, (sp, (matched, _))) =
        ms(sep_pair(opt(parse_expr), Comma, get_this_reg(RegEnum::X)))(input)?;

    let matched = matched.unwrap_or_else(|| {
        let item = AstNodeKind::from_number(0, crate::frontend::ParsedFrom::Expression);
        from_item_tspan(item, sp)
    });

    let node = from_item_tspan(Indexed, sp).with_child(matched);
    Ok((rest, node))
}

fn parse_immediate(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    use TokenKind::Hash;
    let (rest, (sp, matched)) = ms(preceded(Hash, parse_expr))(input)?;
    let node = from_item_tspan(Immediate, sp).with_child(matched);
    Ok((rest, node))
}

fn parse_force_direct(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    use TokenKind::LessThan;
    let (rest, (sp, matched)) = ms(preceded(LessThan, parse_expr))(input)?;
    let node = from_item_tspan(Direct, sp).with_child(matched);
    Ok((rest, node))
}

fn parse_force_extended(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    use TokenKind::GreaterThan;
    let (rest, (sp, matched)) = ms(preceded(GreaterThan, parse_expr))(input)?;
    let node = from_item_tspan(Extended, sp).with_child(matched);
    Ok((rest, node))
}

fn parse_extended(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    let (rest, (sp, matched)) = ms(parse_expr)(input)?;
    let node = from_item_tspan(Extended, sp).with_child(matched);
    Ok((rest, node))
}

fn parse_opcode_arg(input: TSpan) -> PResult<Node> {
    // Indexed addressing is the only 6800 form that is unambiguously
    // identified by its first token.  Dispatching here avoids trying to parse
    // an expression speculatively before discovering the comma and X register.
    if input.iter().any(|token| token.kind == TokenKind::Comma) {
        return parse_indexed(input);
    }

    match input.first().map(|token| token.kind) {
        Some(TokenKind::Hash) => parse_immediate(input),
        Some(TokenKind::LessThan) => parse_force_direct(input),
        Some(TokenKind::GreaterThan) => parse_force_extended(input),
        _ => parse_extended(input),
    }
}

fn get_instruction(amode: AddrModeParseType, info: &Instruction) -> Option<&OpcodeData> {
    use AddrModeEnum::*;
    use AddrModeParseType as PT;
    let get = |amode| info.get_opcode_data(amode);

    match amode {
        PT::Indexed => get(Indexed),
        PT::Direct => get(Direct),
        PT::Extended => get(Extended),
        PT::Relative => get(Relative),
        PT::Inherent => get(Inherent),
        PT::Immediate => get(Immediate8).or_else(|| get(Immediate16)),
    }
}

pub fn parse_opcode(input: TSpan) -> PResult<Node> {
    // Parse the opcode once.  The old `alt` retried `get_opcode` for every
    // inherent instruction, which is particularly costly in large source
    // files and also made the no-argument path depend on parser backtracking.
    let (rest, (sp, info)) = get_opcode(input)?;
    if rest.is_empty() {
        if let Some(ins) = info.get_opcode_data(AddrModeEnum::Inherent) {
            let oc = OpCode(ins.id());
            return Ok((rest, from_item_tspan(oc, sp)));
        }
        return err_fatal(sp, OnlySupports(AddrModeParseType::Inherent));
    }

    use crate::frontend::CpuSpecific;
    use CpuSpecific::Cpu6800 as Cpu;
    use NodeKind6800::{OpCode, Operand};

    let (rest, arg) = parse_opcode_arg(rest)?;
    if let AstNodeKind::TargetSpecific(Cpu(Operand(parsed_addressing_mode))) = arg.item {
        if info.supports(AddrModeEnum::Relative)
            && parsed_addressing_mode == AddrModeParseType::Extended
        {
            let instruction = get_instruction(AddrModeParseType::Relative, info).unwrap();
            let item = NodeKind6800::opcode(instruction.id());
            let node = from_item_tspan(item, sp).take_others_children(arg);
            Ok((rest, node))
        } else if let Some(instruction) = get_instruction(parsed_addressing_mode, info) {
            let item = NodeKind6800::opcode(instruction.id());
            let node = from_item_tspan(item, sp).take_others_children(arg);
            Ok((rest, node))
        } else {
            err_fatal(sp, FrontEndErrorKind::Unexpected)
        }
    } else {
        panic!()
    }
}

pub fn parse_multi_opcode_vec(input: TSpan) -> PResult<Vec<Node>> {
    let (rest, opcode) = parse_opcode(input)?;
    Ok((rest, vec![opcode]))
}
