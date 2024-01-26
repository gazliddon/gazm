use crate::frontend::{get_text, GazmParser, PResult, TSpan, TokenKind};
use super::{ Node, GParser, Item };

use emu6800::cpu_core::{Instruction, InstructionInfo, DBASE, AddrModeEnum};

use crate::cpu6800::{
    frontend::MC6800::{self,OpCode, Operand, OperandIndexed},
    Assembler6800,
};

use unraveler::{alt, match_span as ms, sep_list, tag};

fn get_opcode(input: TSpan) -> PResult<(TSpan, String, &Instruction)> {
    use TokenKind::OpCode;
    let (rest, (sp, matched)) = ms(OpCode)(input)?;
    let text = get_text(matched);
    let info = DBASE.get_opcode(text.as_str()).unwrap();
    Ok((rest, (sp, text, info)))
}

fn parse_opcode_no_arg(input: TSpan) -> PResult<Node> {
    // use Cpu6809AssemblyErrorKind::OnlySupports;

    let (rest, (sp, text, ins)) = get_opcode(input)?;

    if let Some(ins) = ins.get_boxed_instruction(AddrModeEnum::Inherent) {
        let oc = OpCode(text, *ins, AddrModeEnum::Inherent);
        let node = GParser::from_item_tspan(Item::CpuSpecific(oc), sp);
        Ok((rest, node))
    } else {
        err_fatal(sp, OnlySupports(AddrModeParseType::Inherent))
    }
}

fn parse_opcode_with_arg(input: TSpan) -> PResult<Node> {
    panic!()
}

fn parse_opcode(input: TSpan) -> PResult<Node> {
    let (rest, item) = alt((parse_opcode_with_arg, parse_opcode_no_arg))(input)?;
    Ok((rest, item))
}

pub fn parse_multi_opcode_vec(input: TSpan) -> PResult<Vec<Node>> {
    use unraveler::tag;
    use TokenKind::Colon;
    let (rest, matched) = sep_list(parse_opcode, tag(Colon))(input)?;
    Ok((rest, matched))
}
