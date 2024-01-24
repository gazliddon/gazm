#![deny(unused_imports)]
use crate::{
    cpu6809::assembler::Assembler6809,
    frontend::{
        err_fatal, get_text, GazmParser, IdentifierKind, Item, Node, PResult, TSpan, TokenKind,
    },
};

use super::{
    AddrModeParseType,
    AddrModeParseType::Inherent as ParseInherent,
    Cpu6809AssemblyErrorKind, MC6809,
    MC6809::{OpCode, Operand, OperandIndexed},
};

use emu6809::isa::{AddrModeEnum, Dbase, Instruction, InstructionInfo};
use unraveler::{alt, match_span as ms, preceded, sep_list};

lazy_static::lazy_static! {
    pub static ref OPCODES_REC: Dbase = Dbase::new();
}

pub fn get_opcode_info(i: &Instruction) -> Option<&InstructionInfo> {
    OPCODES_REC.get_opcode_info_from_opcode(i.opcode)
}

impl GazmParser<Assembler6809> {
    fn parse_immediate(_input: TSpan) -> PResult<Node<MC6809>> {
        use AddrModeParseType::*;
        use TokenKind::Hash;
        let (rest, (sp, matched)) = ms(preceded(Hash, Self::parse_expr))(_input)?;
        let node = Self::from_item_tspan(Immediate.into(), sp).with_child(matched);
        Ok((rest, node))
    }

    fn parse_force_dp(_input: TSpan) -> PResult<Node<MC6809>> {
        use AddrModeParseType::*;
        use TokenKind::LessThan;
        let (rest, (sp, matched)) = ms(preceded(LessThan, Self::parse_expr))(_input)?;
        let node = Self::from_item_tspan(Direct.into(), sp).with_child(matched);
        Ok((rest, node))
    }

    fn parse_force_extended(_input: TSpan) -> PResult<Node<MC6809>> {
        use AddrModeParseType::*;
        use TokenKind::GreaterThan;
        let (rest, (sp, matched)) = ms(preceded(GreaterThan, Self::parse_expr))(_input)?;
        let node = Self::from_item_tspan(Extended(true).into(), sp).with_child(matched);
        Ok((rest, node))
    }

    fn parse_extended(_input: TSpan) -> PResult<Node<MC6809>> {
        use AddrModeParseType::*;
        let (rest, (sp, matched)) = ms(Self::parse_expr)(_input)?;
        let node = Self::from_item_tspan(Extended(false).into(), sp).with_child(matched);
        Ok((rest, node))
    }

    fn parse_opcode_arg(input: TSpan) -> PResult<Node<MC6809>> {
        let (rest, matched) = alt((
            Self::parse_indexed,
            Self::parse_immediate,
            Self::parse_force_dp,
            Self::parse_force_extended,
            Self::parse_extended,
        ))(input)?;

        Ok((rest, matched))
    }

    fn parse_opcode_with_arg(input: TSpan) -> PResult<Node<MC6809>> {
        use Item::*;
        let (rest, (sp, text, info)) = get_opcode(input)?;

        let (rest, arg) = if info.supports_addr_mode(AddrModeEnum::RegisterSet) {
            Self::parse_reg_set_operand(rest)
        } else if info.supports_addr_mode(AddrModeEnum::RegisterPair) {
            Self::parse_opcode_reg_pair(rest)
        } else {
            Self::parse_opcode_arg(rest)
        }?;

        let amode = match arg.item {
            CpuSpecific(Operand(amode)) => amode,
            CpuSpecific(OperandIndexed(amode, indirect)) => {
                AddrModeParseType::Indexed(amode, indirect)
            }
            _ => return err_fatal(sp, Cpu6809AssemblyErrorKind::AddrModeUnsupported),
        };

        if let Some(instruction) = get_instruction(amode, info) {
            let item = OpCode(text.to_string(), Box::new(instruction.clone()), amode);
            let node = Self::from_item_tspan(item.into(), sp).take_others_children(arg);
            Ok((rest, node))
        } else {
            err_fatal(sp, Cpu6809AssemblyErrorKind::ThisAddrModeUnsupported(amode))
        }
    }
    fn parse_opcode_no_arg(input: TSpan) -> PResult<Node<MC6809>> {
        use Cpu6809AssemblyErrorKind::OnlySupports;
        let (rest, (sp, text, ins)) = get_opcode(input)?;

        if let Some(ins) = ins.get_boxed_instruction(AddrModeEnum::Inherent) {
            let oc = MC6809::OpCode(text, ins, ParseInherent);
            let node = Self::from_item_tspan(oc.into(), sp);
            Ok((rest, node))
        } else {
            err_fatal(sp, OnlySupports(AddrModeParseType::Inherent))
        }
    }
    pub fn parse_opcode(input: TSpan) -> PResult<Node<MC6809>> {
        let (rest, item) = alt((Self::parse_opcode_with_arg, Self::parse_opcode_no_arg))(input)?;
        Ok((rest, item))
    }

    pub fn parse_multi_opcode_vec(input: TSpan) -> PResult<Vec<Node<MC6809>>> {
        use unraveler::tag;
        use TokenKind::Colon;
        let (rest, matched) = sep_list(Self::parse_opcode, tag(Colon))(input)?;
        Ok((rest, matched))
    }
}

fn get_opcode(input: TSpan) -> PResult<(TSpan, String, &InstructionInfo)> {
    use {IdentifierKind::Opcode, TokenKind::Identifier};
    let (rest, (sp, matched)) = ms(Identifier(Opcode))(input)?;
    let text = get_text(matched);
    let info = OPCODES_REC.get_opcode(text.as_str()).unwrap();
    Ok((rest, (sp, text, info)))
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

#[allow(unused_imports)]
mod test {
    use crate::cpu6809::{
        frontend::MC6809::{self, OpCode},
        frontend::{AddrModeParseType, IndexParseType},
        Assembler6809,
    };

    use crate::frontend::{
        create_source_file, get_items, make_tspan, to_tokens_no_comment, GazmParser, Item,
        ParsedFrom,
    };
    use crate::opts::Opts;

    type GParser = GazmParser<Assembler6809>;

    use emu6809::cpu::RegEnum;

    fn check_opcode(
        text: &str,
        opcode: &str,
        expected_amode: AddrModeParseType,
        expected_kids: &[Item<MC6809>],
    ) {
        let opts = Opts::default();

        let sf = create_source_file(text);
        let tokens = to_tokens_no_comment(&sf);
        let span = make_tspan(&tokens, &sf, &opts);
        let tk: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        println!("{:?}", tk);
        let (_, p) = GParser::parse_opcode(span).expect("Can't parse opcode");
        let items = get_items(&p);
        println!("{:?}", items);
        let (item, kids) = get_items(&p);

        if let Item::CpuSpecific(OpCode(_, i, addr_mode)) = item {
            assert_eq!(i.action, opcode);
            assert_eq!(addr_mode, expected_amode);
            assert_eq!(kids, expected_kids);
        } else {
            panic!("Failed")
        }
    }

    #[test]
    fn parse_multi() {
        let opts = Opts::default();
        let text = "lda #10 : sta $20";
        let sf = create_source_file(text);
        let tokens = to_tokens_no_comment(&sf);
        let span = make_tspan(&tokens, &sf, &opts);
        let tk: Vec<_> = tokens.iter().map(|t| t.kind).collect();
        println!("{:?}", tk);
        let (rest, _p) = GParser::parse_multi_opcode_vec(span).expect("Can't parse opcode");
        println!("Rest len is {}", rest.length());
        let tk: Vec<_> = rest.kinds_iter().collect();
        println!("REST: {:?}", tk);
    }

    #[test]
    fn test_op() {
        use AddrModeParseType::*;
        use IndexParseType::*;
        use Item::*;
        use ParsedFrom::*;
        use RegEnum::*;

        let test_data = vec![
            ("lda", "lda #10", Immediate, vec![Num(10, Decimal)]),
            ("ldb", "ldb #$10", Immediate, vec![Num(16, Hexadecimal)]),
            (
                "lda",
                "lda [10],y",
                Indexed(ExtendedIndirect, false),
                vec![Num(10, Decimal)],
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
