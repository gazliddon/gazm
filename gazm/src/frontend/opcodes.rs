use emu6809::{cpu::Inherent, isa::InstructionInfo};
/// parse opodes
use grl_sources::Position;

use unraveler::{
    all, alt, any, cut, is_a, many0, many1, many_until, match_item, not, opt, pair, preceded,
    sep_pair, succeeded, tag, tuple, until, wrapped_cut, Collection, ParseError, ParseErrorKind,
    Parser, Severity,
};

use super::{
    get_str, get_text, match_span as ms, to_pos, IdentifierKind, NumberKind, PResult, TSpan, Token,
    TokenKind::{self, *},
};

use crate::{
    async_tokenize::{GetTokensResult, IncludeErrorKind},
    item::{Item, LabelDefinition, Node, ParsedFrom},
    item6809::MC6809,
    parse6809::opcodes::{get_opcode_info, OPCODES_REC}, parse::util::sep_list1,
};
use emu6809::isa::{AddrModeEnum, Instruction};


fn parse_opcode_arg(_input: TSpan) -> PResult<Node> {
    todo!()
}

fn parse_reg_set(_input: TSpan) -> PResult<Node> { 
    todo!()
}

fn parse_opcode_reg_pair(_input: TSpan) -> PResult<Node> {
    // let (rest,(sp,matched)) = sep_pair(first, Comma, second)
    todo!()
}

fn parse_opcode_with_arg(input: TSpan) -> PResult<Node> {
    let (rest, (_sp, _text,info)) = get_opcode(input)?;

    let (_rest, _arg) = if info.supports_addr_mode(AddrModeEnum::RegisterSet) {
        parse_reg_set(rest)
    } else if info.supports_addr_mode(AddrModeEnum::RegisterPair) {
        parse_opcode_reg_pair(rest)
    } else {
        parse_opcode_arg(rest)
    }?;

    todo!()
}

fn get_opcode(input: TSpan) -> PResult<(TSpan, String,&InstructionInfo )> {
    use {IdentifierKind::Opcode, TokenKind::Identifier};
    let (rest, (sp, matched)) = ms(Identifier(Opcode))(input)?;
    let text = get_text(matched);
    let info = OPCODES_REC.get_opcode(text.as_str()).unwrap();
    Ok((rest,(sp,text,info)))
}

fn parse_opcode_no_arg(input: TSpan) -> PResult<Node> {
    use crate::item6809::AddrModeParseType::Inherent as ParseInherent;
    use {IdentifierKind::Opcode, TokenKind::Identifier};

    let (rest, (sp, text,ins)) = get_opcode(input )?;
    let ins = ins.get_boxed_instruction(&AddrModeEnum::Inherent).unwrap();
    let oc = MC6809::OpCode(text, ins, ParseInherent);
    let node = Node::new(oc.into(), to_pos(sp));
    Ok((rest, node))
}
pub fn parse_opcode(input: TSpan) -> PResult<Node> {
    let (rest, item) = alt((parse_opcode_with_arg, parse_opcode_no_arg))(input)?;
    Ok((rest, item))
}
