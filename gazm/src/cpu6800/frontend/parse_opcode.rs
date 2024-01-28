use crate::cpu6800::frontend::{ get_this_reg};
use crate::cpu6809::frontend::Cpu6809AssemblyErrorKind;
use crate::frontend::{err_fatal, err_nomatch, get_text, PResult, TSpan, TokenKind};

use crate::cpu6800::{
    from_item_tspan,
    frontend::{
        error::AssemblyErrorKind6800::OnlySupports,
        Asm6800, GParser, Item, Node,
        MC6800::{self, OpCode, Operand},
    },
    parse_expr, AddrModeParseType,
};

use emu6800::cpu_core::{AddrModeEnum, Instruction, InstructionInfo, OpcodeData, DBASE, RegEnum};

use serde_json::value::Index;
use unraveler::{alt, match_span as ms, preceded, sep_list, tag, sep_pair};

fn get_opcode(input: TSpan) -> PResult<(TSpan, String, &Instruction)> {
    let (rest, (sp, matched)) = ms(TokenKind::OpCode)(input)?;
    let text = get_text(matched);
    let info = DBASE.get_opcode(text.as_str()).unwrap();
    Ok((rest, (sp, text, info)))
}

fn parse_opcode_no_arg(input: TSpan) -> PResult<Node> {
    let (rest, (sp, text, ins)) = get_opcode(input)?;

    if let Some(ins) = ins.get_opcode_data(AddrModeEnum::Inherent) {
        let oc = OpCode(text, ins.clone(), AddrModeParseType::Inherent);
        let node = from_item_tspan(Item::CpuSpecific(oc), sp);
        Ok((rest, node))
    } else {
        err_fatal(sp, OnlySupports(AddrModeParseType::Inherent))
    }
}

fn parse_indexed(input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    use TokenKind::Comma;
    let (rest, (sp, (matched,_))) = ms(sep_pair(parse_expr, Comma,get_this_reg(RegEnum::X)))(input)?;
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

fn parse_acc_a(input: TSpan) -> PResult<Node> { 
    use AddrModeParseType::*;
    let (rest, (sp, _)) = ms(get_this_reg(RegEnum::A))(input)?;
    let node = from_item_tspan(AccA, sp);
    Ok((rest, node))
}

fn parse_acc_b(input: TSpan) -> PResult<Node> { 
    use AddrModeParseType::*;
    let (rest, (sp, _)) = ms(get_this_reg(RegEnum::B))(input)?;
    let node = from_item_tspan(AccB, sp);
    Ok((rest, node))
}

fn parse_opcode_arg(input: TSpan) -> PResult<Node> {
    let (rest, matched) = alt((
        parse_indexed,
        parse_immediate,
        parse_force_direct,
        parse_force_extended,
        parse_acc_a,
        parse_acc_b,
        parse_extended,
    ))(input)?;

    Ok((rest, matched))
}

fn get_instruction(amode: AddrModeParseType, info: &Instruction) -> Option<&OpcodeData> {
    use AddrModeEnum::*;
    use AddrModeParseType as PT;
    let get = |amode| info.get_opcode_data(amode);

    match amode {
        PT::AccA => get(AccA),
        PT::AccB => get(AccB),
        PT::Indexed => get(Indexed),
        PT::Direct => get(Direct),
        PT::Extended => get(Extended),
        PT::Relative => get(Relative),
        PT::Inherent => get(Inherent),
        PT::Immediate => get(Immediate8).or_else(|| get(Immediate16)),
    }
}

fn parse_opcode_with_arg(input: TSpan) -> PResult<Node> {
    use MC6800::{OpCode, Operand};

    let (rest, (sp, text, info)) = get_opcode(input)?;

    let (rest, arg) = parse_opcode_arg(rest)?;

    if let Item::CpuSpecific(Operand(amode)) = arg.item {
        if let Some(instruction) = get_instruction(amode, info) {
            let item = OpCode(text.to_string(), instruction.clone(), amode);
            let node = from_item_tspan(item, sp).take_others_children(arg);
            Ok((rest, node))
        } else {
            panic!()
            // err_fatal(sp, Cpu6809AssemblyErrorKind::ThisAddrModeUnsupported(amode))
        }
    } else {
        panic!()
    }
}

fn parse_opcode(input: TSpan) -> PResult<Node> {
    let (rest, item) = alt((parse_opcode_with_arg, parse_opcode_no_arg))(input)?;
    Ok((rest, item))
}

pub fn parse_multi_opcode_vec(input: TSpan) -> PResult<Vec<Node>> {
    let (rest, matched) = sep_list(parse_opcode, tag(TokenKind::Colon))(input)?;
    Ok((rest, matched))
}
