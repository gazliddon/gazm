use crate::frontend::{err_fatal, get_text, PResult, TSpan, TokenKind, err_nomatch};

use crate::cpu6800::{
    from_item_tspan,
    parse_expr,
    frontend::{
        error::AssemblyErrorKind6800::OnlySupports,
        Asm6800, GParser, Item, Node,
        MC6800::{self, OpCode, Operand, OperandIndexed},
    },
    AddrModeParseType,
};

use emu6800::cpu_core::{AddrModeEnum, Instruction, InstructionInfo, DBASE};

use unraveler::{alt, match_span as ms, sep_list, tag, preceded};

fn get_opcode(input: TSpan) -> PResult<(TSpan, String, &Instruction)> {
    let (rest, (sp, matched)) = ms(TokenKind::OpCode)(input)?;
    let text = get_text(matched);
    let info = DBASE.get_opcode(text.as_str()).unwrap();
    Ok((rest, (sp, text, info)))
}

fn parse_opcode_no_arg(input: TSpan) -> PResult<Node> {
    let (rest, (sp, text, ins)) = get_opcode(input)?;

    if let Some(ins) = ins.get_boxed_instruction(AddrModeEnum::Inherent) {
        let oc = OpCode(text, *ins, AddrModeEnum::Inherent);
        let node = from_item_tspan(Item::CpuSpecific(oc), sp);
        Ok((rest, node))
    } else {
        err_fatal(sp, OnlySupports(AddrModeParseType::Inherent))
    }
}

fn parse_indexed(input: TSpan) -> PResult<Node> { 
    err_nomatch(input)
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

fn parse_extended(input:TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    let (rest, (sp, matched)) = ms(parse_expr)(input)?;
    let node = from_item_tspan(Extended, sp).with_child(matched);
    Ok((rest, node))
}

fn parse_opcode_arg(input: TSpan) -> PResult<Node> {
    let (rest, matched) = alt((
        parse_indexed,
        parse_immediate,
        parse_force_direct,
        parse_force_extended,
        parse_extended,
    ))(input)?;

    Ok((rest, matched))
}

fn parse_opcode_with_arg(_input: TSpan) -> PResult<Node> {
    err_nomatch(_input)
}

fn parse_opcode(input: TSpan) -> PResult<Node> {
    let (rest, item) = alt((parse_opcode_with_arg, parse_opcode_no_arg))(input)?;
    Ok((rest, item))
}

pub fn parse_multi_opcode_vec(input: TSpan) -> PResult<Vec<Node>> {
    let (rest, matched) = sep_list(parse_opcode, tag(TokenKind::Colon))(input)?;
    Ok((rest, matched))
}
