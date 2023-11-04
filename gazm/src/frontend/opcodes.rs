#![deny(unused_imports)]

use super::{
    get_text, parse_expr, parse_indexed, parse_opcode_reg_pair, IdentifierKind,
    PResult, TSpan, TokenKind,
};
use unraveler::match_span as ms;
use unraveler::{alt, preceded, sep_list};

use crate::{
    frontend::parse_reg_set_operand,
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

fn parse_immediate(_input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    use TokenKind::Hash;
    let (rest, (sp, matched)) = ms(preceded(Hash, parse_expr))(_input)?;
    let node = Node::from_item_tspan(Immediate.into(), sp).with_child(matched);
    Ok((rest, node))
}

fn parse_force_dp(_input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    use TokenKind::LessThan;
    let (rest, (sp, matched)) = ms(preceded(LessThan, parse_expr))(_input)?;
    let node = Node::from_item_tspan(Direct.into(), sp).with_child(matched);
    Ok((rest, node))
}

fn parse_force_extended(_input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    use TokenKind::GreaterThan;
    let (rest, (sp, matched)) = ms(preceded(GreaterThan, parse_expr))(_input)?;
    let node = Node::from_item_tspan(Extended(true).into(), sp).with_child(matched);
    Ok((rest, node))
}

fn parse_extended(_input: TSpan) -> PResult<Node> {
    use AddrModeParseType::*;
    let (rest, (sp, matched)) = ms(parse_expr)(_input)?;
    let node = Node::from_item_tspan(Extended(false).into(), sp).with_child(matched);
    Ok((rest, node))
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

    let (rest, arg) = if info.supports_addr_mode(AddrModeEnum::RegisterSet) {
        parse_reg_set_operand(rest)
    } else if info.supports_addr_mode(AddrModeEnum::RegisterPair) {
        parse_opcode_reg_pair(rest)
    } else {
        parse_opcode_arg(rest)
    }?;

    let amode = match arg.item {
        Cpu(Operand(amode)) => amode,
        Cpu(OperandIndexed(amode, indirect)) => AddrModeParseType::Indexed(amode, indirect),
        _ => {
            println!("{:?}", info);
            todo!("Need an error here {:?}", arg.item);
        }
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

pub fn parse_multi_opcode(input: TSpan) -> PResult<Node> {
    use unraveler::tag;
    use TokenKind::Colon;
    let (rest, (sp, matched)) = ms(sep_list(parse_opcode, tag(Colon)))(input)?;
    Ok((rest, Node::block(matched.into(), sp)))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::frontend::{create_source_file, get_items, make_tspan, to_tokens_no_comment};
    use crate::item::{Item, ParsedFrom};
    use crate::item6809::IndexParseType;
    use emu6809::cpu::RegEnum;
    use unraveler::Collection;

    fn check_opcode(
        text: &str,
        opcode: &str,
        expected_amode: AddrModeParseType,
        expected_kids: &[Item],
    ) {
        let sf = create_source_file(text);
        let tokens = to_tokens_no_comment(&sf);
        let span = make_tspan(&tokens, &sf);
        let tk: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        println!("{:?}", tk);
        let (_, p) = parse_opcode(span).expect("Can't parse opcode");
        let items = get_items(&p);
        println!("{:?}", items);
        let (item, kids) = get_items(&p);

        if let Item::Cpu(OpCode(_, i, addr_mode)) = item {
            assert_eq!(i.action, opcode);
            assert_eq!(addr_mode, expected_amode);
            assert_eq!(kids, expected_kids);
        } else {
            panic!("Failed")
        }
    }

    #[test]
    fn parse_multi() {
        // TODO: check for success
        let text = "lda #10 : sta $20";
        let sf = create_source_file(text);
        let tokens = to_tokens_no_comment(&sf);
        let span = make_tspan(&tokens, &sf);
        let tk: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        println!("{:?}", tk);
        let (rest, p) = parse_multi_opcode(span).expect("Can't parse opcode");
        println!("Rest len is {}", rest.length());
        let tk: Vec<_> = rest.kinds_iter().collect();
        println!("REST: {:?}", tk);
        let _items = get_items(&p);
    }

    #[test]
    fn test_op() {
        use AddrModeParseType::*;
        use IndexParseType::*;
        use Item::*;
        use ParsedFrom::*;
        use RegEnum::*;

        let test_data = vec![
            ("lda", "lda #10", Immediate, vec![Num(10, Dec)]),
            ("ldb", "ldb #$10", Immediate, vec![Num(16, Hex)]),
            (
                "lda",
                "lda [10],y",
                Indexed(ExtendedIndirect, false),
                vec![Num(10, Dec)],
            ),
            ("tfr", "tfr x,y", RegisterPair(X, Y), vec![]),
            ("nop", "nop", Inherent, vec![]),
            ("lda", "lda <(10+8/2)", Direct, vec![Expr]),
            ("lda", "lda (10+8/2)", Extended(false), vec![Expr]),
            ("lda", "lda >(10+8/2)", Extended(true), vec![Expr]),
        ];

        for (opcode, text, expected_amode, expected_kids) in test_data {
            check_opcode(text, opcode, expected_amode, &expected_kids);
        }
    }
}
