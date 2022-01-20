use crate::indexed::parse_indexed;
use crate::expr;
use crate::expr::parse_expr;
use crate::locate::matched_span;
use crate::register;

use super::item::{ Item,Node };
use super::util;
use emu::isa::AddrModeEnum;
use emu::cpu::RegEnum;
use nom::character::complete::digit0;
use emu::isa::{Dbase, Instruction, InstructionInfo};

use nom::branch::alt;

// use std::ascii::AsciiExt;
use std::collections::HashMap;
use nom::error::ErrorKind::NoneOf;
use nom::error::Error;

use nom::character::complete::{
    alpha1, multispace0, multispace1
};

use nom::bytes::complete::tag;
use nom::sequence::{ pair, preceded, separated_pair, terminated, tuple};
use nom::combinator::{ recognize, opt, cut };

use crate::error::{IResult, ParseError};
use crate::locate::Span;

////////////////////////////////////////////////////////////////////////////////
// opcode parsing
lazy_static::lazy_static! {
    static ref OPCODES_REC: Dbase = Dbase::new();
}

pub fn opcode_just_token(input: Span) -> IResult<Span > {
    nom::combinator::map(opcode_token, |(_,e)| e)(input)
}

pub fn opcode_token(input: Span) -> IResult<(&InstructionInfo, Span )> {
    use crate::error::error;
    let (rest,matched) = recognize(pair(alpha1,digit0))(input)?;

    if let Some(op_code) = OPCODES_REC.get_opcode(&matched) {
        Ok((rest, (op_code, matched )))
    } else {
        Err(error("Expected an opcode", input))
    }
}

fn parse_immediate(input: Span) -> IResult<Node> {
    use Item::*;
    use crate::item::AddrModeParseType::*;
    let (rest, matched) = preceded(tag("#"), expr::parse_expr)(input)?;
    let ret = Node::from_item(Operand(Immediate), input).with_child(matched);
    Ok((rest, ret))
}

fn parse_dp(input: Span) -> IResult<Node> {
    use Item::*;
    use crate::item::AddrModeParseType::*;
    let (rest, matched) = preceded(tag("<"), expr::parse_expr)(input)?;
    let ret = Node::from_item(Operand(Direct), input).with_child(matched);
    Ok((rest, ret))
}

fn parse_reg_set(input : Span) -> IResult<Node> {
    use Item::*;
    use crate::item::AddrModeParseType;
    use nom::combinator::map;

    let (rest, matched) = register::parse_reg_set_1(input)?;
    let matched = Node::from_item(Operand(AddrModeParseType::RegisterSet), input).with_child(matched);
    Ok((rest,matched))
}
fn parse_opcode_reg_pair(input : Span) -> IResult<Node> {
    use Item::*;
    use crate::item::AddrModeParseType;
    use nom::combinator::map;
    let reg_map = |(a,b)| Node::from_item(Operand(AddrModeParseType::RegisterPair(a,b)), input.clone());

    let (rest, matched) =map(register::get_reg_pair, reg_map)(input)?;

    Ok((rest,matched))
}

fn parse_extended(input : Span) -> IResult<Node> {
    use Item::*;
    use crate::item::AddrModeParseType::*;
    use nom::combinator::map;
    let (rest,matched) = expr::parse_expr(input)?;
    let  res = Node::from_item(Operand(Extended), input).with_child(matched);
    Ok((rest, res))
}

fn parse_opcode_arg(input: Span) -> IResult< Node> {
    use Item::*;
    use nom::combinator::map;
    use super::indexed::parse_indexed;

    let (rest, matched) = 
        alt((
              parse_indexed,
              parse_immediate,
              parse_dp,
              parse_extended
             ))(input)?;

    Ok((rest, matched))
}

fn get_instruction<'a>(amode : &crate::item::AddrModeParseType, info: &'a InstructionInfo) -> Option<&'a Instruction>{ 
    use crate::item::AddrModeParseType;
    use AddrModeEnum::*;
    let get = |amode| info.get_instruction(&amode);

    match amode {
        AddrModeParseType::Indexed(_)=>{get(Indexed)},

        AddrModeParseType::Direct=>{get(Direct)},

        AddrModeParseType::Extended=>{
            get(Extended)
                .or_else(||get(Relative))
                .or_else(||get(Relative16))
        },

        AddrModeParseType::Relative=>{
            get(Relative)
                .or_else(|| get(Relative16))
        },

        AddrModeParseType::Inherent=>{get(Inherent)},

        AddrModeParseType::Immediate => {
            get(Immediate8)
                .or_else(|| get(Immediate16))
        },
        AddrModeParseType::RegisterPair(..)=>{get(RegisterPair)},

        AddrModeParseType::RegisterSet=>{get(RegisterSet)},
    }
}


fn parse_opcode_with_arg(input: Span) -> IResult< Node> {
    use AddrModeEnum::*;
    use Item::*;

    let (rest,(info, text)) = opcode_token(input)?;

    let (rest, arg) = if info.supports_addr_mode(AddrModeEnum::RegisterSet) {
        let (rest, arg) = preceded(multispace1, parse_reg_set)(rest)?;
        Ok((rest, arg))
    } else if info.supports_addr_mode(AddrModeEnum::RegisterPair){
        preceded(multispace1, parse_opcode_reg_pair)(rest)
    } else {
        preceded(multispace1, parse_opcode_arg)(rest)
    }?;

    let amode = if let Operand(amode) = arg.item() {
        amode
    } else {
        todo!("Need an error here {:?}", arg.item())
    };

    if let Some(instruction) = get_instruction(amode, info) {
        let matched = matched_span(input, rest );
        let item = Item::OpCode(instruction.clone(), *amode);
        let node = Node::from_item(item, matched)
            .take_children(arg);
        Ok((rest, node))

    } else {
        let msg = format!("{} does not support {:?} addresing mode", text,amode);
        Err(nom::Err::Failure(ParseError::new( msg, &input)))
    }
}

fn parse_opcode_no_arg(input: Span) -> IResult< Node> {
    use Item::*;
    use AddrModeEnum::*;

    let (rest, (info, text )) = opcode_token(input)?;
    let matched_span = matched_span(input, rest);

    if let Some(instruction) = info.get_instruction(&Inherent) {
        let ret = Node::from_item(OpCode(instruction.clone(), super::item::AddrModeParseType::Inherent), matched_span);
        Ok((rest,ret))

    } else {
        let msg = format!("Missing operand for {}", text);
        Err(nom::Err::Failure(ParseError::new( msg, &input)))
    }
}

pub fn parse_opcode(input: Span) -> IResult<Node> {
    let (rest, item) = alt((parse_opcode_with_arg, parse_opcode_no_arg))(input)?;
    Ok((rest, item))
}

#[allow(unused_imports)]
mod test {

    use std::os::unix::prelude::JoinHandleExt;

    use emu::cpu::RegEnum;
    use pretty_assertions::{assert_eq, assert_ne};
    use crate::locate::Position;

    use super::*;
    use crate::util::compile_text;


    #[test]
    fn test_parse_misc() {
        let test = vec!{
            ("sync", "sync"),
            ("lda #$100", "lda #256"),
            ("lda <$100", "lda <256"),
            ("lda 1000,x", "lda 1000,X"),
            ("lda ,x", "lda ,X"),
            ("lda ,y", "lda ,Y"),
            ("lda ,--y", "lda ,--Y"),
            ("lda ,-y", "lda ,-Y"),
            ("lda ,y+", "lda ,Y+"),
            ("lda ,U++", "lda ,U++"),
            ("lda a,U", "lda A,U"),
            ("lda b,x", "lda B,X"),
            ("lda d,y", "lda D,Y"),
            ("lda %1111,pc", "lda 15,PC"),
        };

        for (code, desired) in test {
            println!("{:?} -> {:?}", code, desired);

            let ast_text = compile_text(code);
            println!("{:?}", ast_text);

            let ast_text = ast_text.unwrap();
            assert_eq!(desired, &ast_text);
        }
    }
}
