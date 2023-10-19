#![deny(unused_imports)]

use unraveler::alt;

use super::{
    get_text, match_span as ms, parse_opcode_reg_pair, parse_reg_set, IdentifierKind,
    PResult, TSpan, TokenKind,
};

use crate::{
    item::{Item, Node},
    item6809::MC6809,
    parse6809::opcodes::OPCODES_REC,
};

use crate::item6809::{
    AddrModeParseType,
    AddrModeParseType::Inherent as ParseInherent,
    MC6809::{OpCode, Operand, OperandIndexed},
};

use emu6809::isa::{AddrModeEnum, Instruction, InstructionInfo};

fn parse_indexed(_input: TSpan) -> PResult<Node> {
    panic!()
}

fn parse_immediate(_input: TSpan) -> PResult<Node> {
    panic!()
}

fn parse_force_dp(_input: TSpan) -> PResult<Node> {
    panic!()
}

fn parse_force_extended(_input: TSpan) -> PResult<Node> {
    panic!()
}

fn parse_extended(_input: TSpan) -> PResult<Node> {
    panic!()
}

fn parse_opcode_arg(input: TSpan) -> PResult<Node> {

    let (rest, matched) = alt((
        parse_indexed,
        parse_immediate,
        parse_force_dp,
        parse_force_extended,
        parse_extended,
    ))(input)?;

    Ok((rest, matched))
}


fn get_instruction(
    amode: crate::item6809::AddrModeParseType,
    info: &InstructionInfo,
) -> Option<&Instruction> {
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

fn parse_opcode_with_arg(input: TSpan) -> PResult<Node> {
    use Item::*;
    let (rest, (sp, text, info)) = get_opcode(input)?;

    let (_rest, arg) = if info.supports_addr_mode(AddrModeEnum::RegisterSet) {
        parse_reg_set(rest)
    } else if info.supports_addr_mode(AddrModeEnum::RegisterPair) {
        parse_opcode_reg_pair(rest)
    } else {
        parse_opcode_arg(rest)
    }?;

    let amode = match arg.item {
        Cpu(Operand(amode)) => amode,
        Cpu(OperandIndexed(amode, indirect)) => AddrModeParseType::Indexed(amode, indirect),
        _ => todo!("Need an error here {:?}", arg.item),
    };

    if let Some(instruction) = get_instruction(amode, info) {
        let item = OpCode(text.to_string(), Box::new(instruction.clone()), amode);
        let node = Node::from_item_tspan(item.into(), sp).take_others_children(arg);
        Ok((rest, node))
    } else {
        let _msg = format!("{text} does not support {amode:?} addresing mode");
        panic!()
        // Err(crate::error::parse_error(&msg, input))
    }
}

fn get_opcode(input: TSpan) -> PResult<(TSpan, String, &InstructionInfo)> {
    use {IdentifierKind::Opcode, TokenKind::Identifier};
    let (rest, (sp, matched)) = ms(Identifier(Opcode))(input)?;
    let text = get_text(matched);
    let info = OPCODES_REC.get_opcode(text.as_str()).unwrap();
    Ok((rest, (sp, text, info)))
}

fn parse_opcode_no_arg(input: TSpan) -> PResult<Node> {
    let (rest, (sp, text, ins)) = get_opcode(input)?;
    let ins = ins.get_boxed_instruction(&AddrModeEnum::Inherent).unwrap();
    let oc = MC6809::OpCode(text, ins, ParseInherent);
    let node = Node::from_item_tspan(oc.into(), sp);
    Ok((rest, node))
}
pub fn parse_opcode(input: TSpan) -> PResult<Node> {
    let (rest, item) = alt((parse_opcode_with_arg, parse_opcode_no_arg))(input)?;
    Ok((rest, item))
}
