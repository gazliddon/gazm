use crate::frontend::{
    err_fatal, err_kind_nomatch, fatal, from_item_tspan, get_text, parse_expr, parse_inherent,
    parse_opcode_operand, parse_prefixed_operand, AstNodeKind, CpuAssemblyErrorKind, CpuSpecific,
    FrontEndError, FrontEndErrorKind, Node, PResult, ParsedFrom, TSpan, TokenKind,
};

use crate::cpukind::CpuKind::Cpu6800 as Cpu;

use crate::cpu6800::frontend::{
    get_this_reg, AddrModeParseType,
    NodeKind6800::{self, OpCode, Operand},
};

use emu6800::cpu_core::{AddrModeEnum, Instruction, OpcodeData, OpcodeId, RegEnum, DBASE};

use unraveler::{alt, match_span as ms, opt, sep_pair, Collection};

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
    parse_prefixed_operand(input, Some(TokenKind::Hash), Immediate)
}

fn parse_force_direct(input: TSpan) -> PResult<Node> {
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
        PT::Extended(_) => get(Extended),
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
        return parse_inherent(
            rest,
            sp,
            || info.get_opcode_data(AddrModeEnum::Inherent).map(|i| i.id()),
            |id| OpCode(id, AddrModeParseType::Inherent).into(),
            CpuAssemblyErrorKind::OnlySupports,
        );
    }

    parse_opcode_operand(
        rest,
        sp,
        parse_opcode_arg,
        |arg| -> Result<(AddrModeParseType, OpcodeId), FrontEndError> {
            if let AstNodeKind::TargetSpecific(CpuSpecific::Cpu6800(Operand(
                parsed_addressing_mode,
            ))) = &arg.item
            {
                if info.supports(AddrModeEnum::Relative)
                    && matches!(parsed_addressing_mode, AddrModeParseType::Extended(_))
                {
                    let instruction = get_instruction(AddrModeParseType::Relative, info).unwrap();
                    Ok((AddrModeParseType::Relative, instruction.id()))
                } else if let Some(instruction) = get_instruction(*parsed_addressing_mode, info) {
                    Ok((*parsed_addressing_mode, instruction.id()))
                } else {
                    Err(fatal(sp, FrontEndErrorKind::Unexpected))
                }
            } else {
                panic!()
            }
        },
        |amode, id| NodeKind6800::opcode(id, amode).into(),
    )
}

pub fn parse_multi_opcode_vec(input: TSpan) -> PResult<Vec<Node>> {
    let (rest, opcode) = parse_opcode(input)?;
    Ok((rest, vec![opcode]))
}
