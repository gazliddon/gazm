use crate::expr;
use crate::expr::parse_expr;
use crate::locate::matched_span;
use crate::register;
use crate::register::get_reg;
use crate::register::parse_index_reg;

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
use nom::combinator::{ recognize, opt };

use crate::error::{IResult, ParseError};
use crate::locate::{ Span, mk_span };


////////////////////////////////////////////////////////////////////////////////
// opcode parsing
lazy_static::lazy_static! {
    static ref OPCODES_REC: Dbase = Dbase::new();
}


pub fn opcode_just_token(input: Span) -> IResult<Span > {
    nom::combinator::map(opcode_token, |(_,e)| e)(input)
}

pub fn opcode_token(input: Span) -> IResult<(&InstructionInfo, Span )> {
    // Some opcodes have a number

    let (rest,matched) = recognize(pair(alpha1,digit0))(input)?;

    if let Some(op_code) = OPCODES_REC.get_opcode(&matched) {
        Ok((rest, (op_code, matched )))
    } else {
        
        Err(nom::Err::Error(ParseError::new(
            "This is not an opcode".to_owned(),
            &input,
        )))
    }
}

fn parse_immediate(input: Span) -> IResult<Node> {
    use Item::*;
    use AddrModeEnum::*;
    let (rest, matched) = preceded(tag("#"), expr::parse_expr)(input)?;
    let ret = Node::from_item(Operand(Immediate8), input).with_child(matched);
    Ok((rest, ret))
}

fn parse_dp(input: Span) -> IResult<Node> {
    use Item::*;
    use AddrModeEnum::*;
    let (rest, matched) = preceded(tag("<"), expr::parse_expr)(input)?;
    let ret = Node::from_item(Operand(Direct), input).with_child(matched);
    Ok((rest, ret))
}

// Post inc / dec
fn parse_post_inc(input: Span) -> IResult<Node> {
    let (rest, matched) = terminated( get_reg , tag("+"))(input)?;
    let ret = Node::from_item(Item::PostIncrement(matched), input);
    Ok((rest,ret))
}

fn parse_post_inc_inc(input: Span) -> IResult<Node> {
    let (rest, matched) = terminated( get_reg , tag("++"))(input)?;
    let ret =  Node::from_item(Item::DoublePostIncrement(matched), input);

    Ok((rest,ret))
}
fn parse_post_dec(input: Span) -> IResult<Node> {
    let (rest, matched) = terminated( get_reg , tag("-"))(input)?;
    let ret = Node::from_item( Item::PostDecrement(matched), input);
    Ok((rest,ret))
}
fn parse_post_dec_dec(input: Span) -> IResult<Node> {
    let (rest, matched) = terminated( get_reg , tag("--"))(input)?;
    let ret = Node::from_item(
        Item::DoublePostDecrement(matched), input);
    Ok((rest,ret))
}

// Pre inc / dec
fn parse_pre_dec(input: Span) -> IResult<Node> {
    let (rest, matched) = preceded(tag("-"), get_reg )(input)?;
    let ret = Node::from_item(
        Item::PreDecrement(matched), input);
    Ok((rest, ret))
}

fn parse_pre_inc(input: Span) -> IResult<Node> {
    let (rest, matched) = preceded(tag("+"), get_reg )(input)?;
    let ret = Node::from_item(Item::PreIncrement(matched), input);
    Ok((rest, ret))
}

fn parse_pre_inc_inc(input: Span) -> IResult<Node> {
    let (rest, matched) = preceded(tag("++"), get_reg )(input)?;
    let ret = Node::from_item(
        Item::DoublePreIncrement(matched), input) ;
    Ok((rest, ret))
}


fn parse_pre_dec_dec(input: Span) -> IResult<Node> {
    let (rest, matched) = preceded(tag("--"), get_reg )(input)?;
    let ret = Node::from_item(Item::DoublePreDecrement(matched), input);
    Ok((rest, ret))
}

// Simple index


fn parse_index_type(input : Span) -> IResult< Node> {
    let (rest, reg) = 
        alt((
                parse_pre_dec_dec,
                parse_pre_inc_inc,
                parse_pre_dec,
                parse_pre_inc,
                parse_post_dec_dec,
                parse_post_inc_inc,
                parse_post_dec,
                parse_post_inc,
                parse_index_reg  )
        )(input)?;

    Ok((rest, reg))
}

fn parse_indexed(input : Span) -> IResult< Node> {
    use AddrModeEnum::*;
    use Item::*;
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));

    let (rest, (expr,reg)) = separated_pair(
        opt(parse_expr),
        sep,
        parse_index_type
        )(input)?;

    let zero = Node::from_number(0, input);

    let expr = expr.unwrap_or(zero);

    let ret = Node::from_item(Operand(Indexed), input);

    let ret = ret.with_children(vec![expr, reg]);

    Ok((rest, ret))
}

fn parse_indirect(input: Span) -> IResult< Node> {
    use AddrModeEnum::*;
    use Item::*;
    use util::wrapped_chars;

    let (rest, matched) = wrapped_chars('[',
        alt((parse_indexed,parse_expr))
    , ']')(input)?;

    let ret = Node::from_item(Operand(Indexed), input).with_child(matched);

    Ok((rest, ret))
}

fn parse_reg_set(input : Span) -> IResult<Node> {
    use Item::*;
    use nom::combinator::map;
    let reg_map = |other| Node::from_item(Operand(AddrModeEnum::RegisterSet), input).take_children(other);
    map(register::parse_reg_set_1, reg_map)(input)
}

fn parse_extended(input : Span) -> IResult<Node> {
    use Item::*;
    use AddrModeEnum::*;
    use nom::combinator::map;
    let (rest,matched) = expr::parse_expr(input)?;
    let  res = Node::from_item(Operand(Extended), input).with_child(matched);
    Ok((rest, res))
}

fn parse_opcode_arg(input: Span) -> IResult< Node> {

    use Item::*;
    use nom::combinator::map;

    let (rest, matched) = 
        alt((parse_reg_set,
              parse_immediate,
              parse_indirect,
              parse_dp,
              parse_indexed,
              parse_extended
             ))(input)?;

    Ok((rest, matched))
}

pub fn parse_opcode_with_arg(input: Span) -> IResult< Node> {
    use AddrModeEnum::*;
    use Item::*;

    let (rest,(info, text)) = opcode_token(input)?;

    let (rest,  arg) = preceded(multispace1, parse_opcode_arg)(rest)?;

    let amode = if let Operand(amode) = arg.item() {
        amode
    } else {
        todo!("Need an error here {:?}", arg.item())
    };

    let mut amode = match amode {
        Immediate8 | Immediate16 => info.get_immediate_mode_supported().unwrap_or(*amode),
        _ => *amode,
    };

    if amode == Extended && info.supports_addr_mode(Relative) {
        amode = Relative
    }

    if let Some(instruction) = info.get_instruction(&amode) {
        let matched = matched_span(input, rest );
        let item = Item::OpCode(text.to_string(), instruction.clone());
        let node = Node::from_item(item, matched)
            .take_children(arg);
        Ok((rest, node))

    } else {
        let msg = format!("{} does not support {} addresing mode", text,amode);
        Err(nom::Err::Failure(ParseError::new( msg, &input)))

    }
}

pub fn parse_opcode_no_arg(input: Span) -> IResult< Node> {
    use Item::*;
    use AddrModeEnum::*;

    let (rest, (info, text )) = opcode_token(input)?;

    if let Some(instruction) = info.get_instruction(&Inherent) {
        let matched = matched_span(input, rest);
        let ret = Node::from_item(OpCode(text.to_string(), instruction.clone()), matched);
        Ok((rest,ret))

    } else {
        let msg = format!("Missing operand for {}", text);
        Err(nom::Err::Failure(ParseError::new( msg, &input)))
    }
}

pub fn parse_opcode(input: Span) -> IResult< Node> {
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

    // #[test]
    // fn test_opcode_reg_list() {
    //     use Item::*;
    //     use emu::cpu::RegEnum::*;

    //     let op_text = "pshu a,b,d,x,y";
    //     let op_pos = 0;
    //     let set_pos = op_pos+5;
    //     let end_pos = op_text.len();

    //     let (_rest, matched) = parse_opcode_with_arg(op_text.as_span()).unwrap();

    //     let set  = vec![A,B,D,X,Y].into_iter().collect();
    //     println!("{:?}", set);
    //     let rset = RegisterSet(set);
    //     println!("{:?}", rset);

    //     let des_child =  Node::from_item(rset).with_upos(set_pos, end_pos);

    //     let des_node = Node::from_item(OpCode("pshu".to_owned()))
    //         .with_child(des_child)
    //         .with_upos(op_pos, end_pos);

    //     assert_eq!(matched, des_node);

    //     let op_text = "pshu a,b,d,x,y,y";
    //     let res = parse_opcode_with_arg(op_text.as_span());

    //     if let Ok(( _,matched )) = &res {
    //         println!("{:#?}",matched);
    //         println!("{:#?}",matched.children);
    //     } else {
    //         println!("{:?}", res);
    //     }

    //     assert!(res.is_err())
    // }

    // #[test]
    // fn test_opcode_immediate() {
    //     let op_text = "lda #100";
    //     let (_rest, matched) = parse_opcode_with_arg(op_text.as_span()).unwrap();
    //     let op_end = op_text.len();
    //     let arg_pos = 4;
    //     let num_pos = arg_pos + 1;

    //     let oc = "lda";
    //     let num = 100;

    //     let des_node = Node::from_item(Item::OpCode(oc.to_string())).with_upos(0, op_end);
    //     let des_arg = Node::from_item(Item::Immediate)
    //         .with_child(
    //             Node::from_number(num).with_upos(num_pos, op_end))
    //         .with_upos(arg_pos, op_end);
    //     let des_node = des_node.with_child(des_arg);

    //     assert_eq!(matched, des_node);
    // }

    #[test]
    fn test_parse_immediate() {
        // use Item::*;
        // use AddrModeEnum::*;
        // let op_text = "#$100+10";
        // let op_text = mk_span("test", &op_text);

        // let num_p = 1;
        // let plus_p = num_p+4;
        // let ten_p = plus_p+1;
        // let last_p = op_text.len();

        // let res = parse_immediate(op_text);
        // assert!(res.is_ok());

        // let des_arg = vec![
        //     Node::from_number(256).with_upos(num_p,plus_p),
        //     Node::from_item(Item::Add).with_child(Node::from_number(10).with_upos(ten_p,last_p)).with_upos(5, last_p)
        // ];

        // let des_expr = Node::from_item(Expr).with_children(des_arg).with_upos(num_p,last_p);
        // let desired = Node::from_item(Operand(Immediate16)).with_child(des_expr).with_upos(0,last_p);
        // let (_, matched) = res.unwrap();

        // assert_eq!(matched,desired);
    }
    // fn simple_indexed(op : &str, middle : &str, index: &str, reg : RegEnum) {
    //     use emu::cpu::RegEnum::*;
    //     use Item::*;
    //     use AddrModeEnum::*;
    //     let op_text = format!("{}{}{}",op,middle, index);
    //     let op_text = mk_span("test", &op_text);
    //     let res = parse_indexed(op_text);
    //     assert!(res.is_ok());
    //     println!("line: {:?}", op_text.to_string());

    //     let op_start = 0;
    //     let middle_start = op.len();
    //     let index_start = middle_start + middle.len();
    //     let end = index_start + index.len();

    //     let des_args = vec![
    //         Node::from_number(0, op_text).with_ctx(op_text),
    //         Node::from_item(Item::Register(reg), op_text).with_ctx(op_text),
    //     ];

    //     let desired = Node::from_item(Operand(Indexed), op_text)
    //         .with_children(des_args)
    //         .with_ctx(op_text) ;

    //     let (_, matched) = res.unwrap();
    //     assert_eq!(matched, desired);
    // }

    // #[test]
    // fn test_simple_indexed() {
    //     use emu::cpu::RegEnum::*;
    //     use Item::*;

    //     let op = "0";
    //     let middle = ",";
    //     let index = "X";
    //     simple_indexed(op, middle, index, X);

    //     let op = "";
    //     let middle = ",";
    //     let index = "X";
    //     simple_indexed(op, middle, index,X);

    //     let op = "";
    //     let middle = ",";
    //     let index = "Y";
    //     simple_indexed(op, middle, index,Y);
    // }

    // fn test_item<'a, F>(mut parse : F, input : &'a str, des : &Item) -> Item
    //     where
    //         F : nom::Parser<Span<'a>,Node<'a>,ParseError<'a>>
    //         {
    //             let input = mk_span("test", input);
    //             let res = parse.parse(input);
    //             assert!(res.is_ok());

    //             let (_, res) = res.unwrap();
    //             assert_eq!(*res.item(), *des);
    //             res.item().clone()
    //         }

    // #[test]
    // fn test_pre_post_dec() {

    //     use emu::cpu::RegEnum::*;
    //     use Item::*;

    //     let input = "--X";
    //     let des = DoublePreDecrement(X);
    //     let p = parse_pre_dec_dec;
    //     test_item(p, input, &des);

    //     let input = "-X";
    //     let des = PreDecrement(X);
    //     let p = parse_pre_dec;
    //     test_item(p, input, &des);

    //     let input = "+X";
    //     let des = PreIncrement(X);
    //     let p = parse_pre_inc;
    //     test_item(p, input, &des);

    //     let input = "++X";
    //     let des = DoublePreIncrement(X);
    //     let p = parse_pre_inc_inc;
    //     test_item(p, input, &des);

    //     let input = "X--";
    //     let des = DoublePostDecrement(X);
    //     let p = parse_post_dec_dec;
    //     test_item(p, input, &des);

    //     let input = "X-";
    //     let des = PostDecrement(X);
    //     let p = parse_post_dec;
    //     test_item(p, input, &des);

    //     let input = "X+";
    //     let des = PostIncrement(X);
    //     let p = parse_post_inc;
    //     test_item(p, input, &des);

    //     let input = "X++";
    //     let des = DoublePostIncrement(X);
    //     let p = parse_post_inc_inc;
    //     test_item(p, input, &des);
    // }
}
