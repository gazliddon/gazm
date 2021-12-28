use crate::expr;
use crate::expr::parse_expr;
use crate::register;
use crate::register::get_reg;
use crate::register::parse_index_reg;

use super::item::{ Item,Node };
use super::util;
use emu::cpu::RegEnum;
use nom::character::complete::digit0;
use emu::isa::{Dbase, Instruction};

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
use crate::locate::{ Span, AsSpan };

////////////////////////////////////////////////////////////////////////////////
// opcode parsing
pub struct OpCodes {
    name_to_ins: HashMap<String, Vec<Instruction>>,
    dbase: Dbase,
}

// Some opcodes have an aliase delimited by underscores
fn split_opcodes(_input: &str) -> Option<(&str, &str)> {
    let split: Vec<&str> = _input.split('_').collect();

    if split.len() != 2 {
        None
    } else {
        Some((split[0], split[1]))
    }
}

impl OpCodes {
    pub fn new() -> Self {
        let dbase = Dbase::new();

        let mut name_to_ins: HashMap<String, Vec<Instruction>> = HashMap::new();

        let mut add = |name: &str, i: &Instruction| {
            let i = i.clone();
            let name = String::from(name).to_ascii_lowercase();
            if let Some(rec) = name_to_ins.get_mut(&name) {
                rec.push(i);
            } else {
                name_to_ins.insert(name.to_string(), vec![i]);
            }
        };

        for i in dbase.all_instructions() {
            if let Some((a, b)) = split_opcodes(&i.action) {
                add(a, i);
                add(b, i);
            } else {
                add(&i.action, i);
            }
        }
        Self { name_to_ins, dbase }
    }

    pub fn is_opcode(&self, input: Span) -> bool {
        self.get_opcode(input).is_some()
    }

    pub fn get_opcode(&self, input: Span) -> Option<&Vec<Instruction>> {
        let op = input.to_string().to_lowercase();
        self.name_to_ins.get(&op)
    }

    pub fn get_db(&self) -> &Dbase {
        &self.dbase
    }
}

////////////////////////////////////////////////////////////////////////////////
// opcode parsing
lazy_static::lazy_static! {
    static ref OPCODES_REC: OpCodes = OpCodes::new();
}

pub fn opcode_token(input: Span) -> IResult<Span> {
    // Some opcodes have a number
    let (rest,matched) = recognize(pair(
            alpha1,digit0))(input)?;

    if OPCODES_REC.is_opcode(matched) {
        Ok((rest, matched))
    } else {
        
        Err(nom::Err::Error(ParseError::new(
            "This is not an opcode".to_owned(),
            input,
        )))
    }
}

fn parse_immediate(input: Span) -> IResult<Node> {
    let (rest, matched) = preceded(tag("#"), expr::parse_expr)(input)?;
    let ret = Node::from_item(Item::Immediate).with_child(matched).with_pos(input,rest);
    Ok((rest, ret))
}

fn parse_dp(input: Span) -> IResult<Node> {
    let (rest, matched) = preceded(tag("<"), expr::parse_expr)(input)?;
    let ret = Node::from_item(Item::DirectPage).with_child(matched).with_pos(input,rest);
    Ok((rest, ret))
}

// Post inc / dec
fn parse_post_inc(input: Span) -> IResult<Node> {
    let (rest, matched) = terminated( get_reg , tag("+"))(input)?;
    let ret = Node::from_item(Item::PostIncrement(matched)).with_pos(input, rest);
    Ok((rest,ret))
}

fn parse_post_inc_inc(input: Span) -> IResult<Node> {
    let (rest, matched) = terminated( get_reg , tag("++"))(input)?;
    let ret =  Node::from_item(Item::DoublePostIncrement(matched)).with_pos(input, rest);

    Ok((rest,ret))
}
fn parse_post_dec(input: Span) -> IResult<Node> {
    let (rest, matched) = terminated( get_reg , tag("-"))(input)?;
    let ret = Node::from_item( Item::PostDecrement(matched)).with_pos(input, rest);
    Ok((rest,ret))
}
fn parse_post_dec_dec(input: Span) -> IResult<Node> {
    let (rest, matched) = terminated( get_reg , tag("--"))(input)?;
    let ret = Node::from_item(
        Item::DoublePostDecrement(matched)).with_pos(input, rest);
    Ok((rest,ret))
}

// Pre inc / dec
fn parse_pre_dec(input: Span) -> IResult<Node> {
    let (rest, matched) = preceded(tag("-"), get_reg )(input)?;
    let ret = Node::from_item(
        Item::PreDecrement(matched)).with_pos(input, rest);
    Ok((rest, ret))
}

fn parse_pre_inc(input: Span) -> IResult<Node> {
    let (rest, matched) = preceded(tag("+"), get_reg )(input)?;
    let ret = Node::from_item(
        Item::PreIncrement(matched)).
        with_pos(input, rest);
    Ok((rest, ret))
}

fn parse_pre_inc_inc(input: Span) -> IResult<Node> {
    let (rest, matched) = preceded(tag("++"), get_reg )(input)?;
    let ret = Node::from_item(
        Item::DoublePreIncrement(matched))
        .with_pos(input,rest);
    Ok((rest, ret))
}


fn parse_pre_dec_dec(input: Span) -> IResult<Node> {
    let (rest, matched) = preceded(tag("--"), get_reg )(input)?;
    let ret = Node::from_item(Item::DoublePreDecrement(matched)).with_pos(input,rest);
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

    Ok((rest, reg.with_pos(input, rest)))
}

fn parse_indexed(input : Span) -> IResult< Node> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));

    let (rest, (expr,reg)) = separated_pair(
        opt(parse_expr),
        sep,
        parse_index_type
        )(input)?;

    let zero = Node::from_number(0).with_pos(input,input);

    let expr = expr.unwrap_or(zero);

    let ret = Node::from_item(Item::Indexed);

    let ret = ret.with_children(vec![expr, reg]).with_pos(input, rest);

    Ok((rest, ret))
}

fn parse_indirect(input: Span) -> IResult< Node> {
    use util::wrapped_chars;

    let (rest, matched) = wrapped_chars('[',
        alt((parse_indexed,parse_expr))
    , ']')(input)?;

    let ret = Node::from_item(Item::Indirect).with_child(matched).with_pos(input, rest);

    Ok((rest, ret))
}

fn parse_opcode_arg(input: Span) -> IResult< Node> {
    let (rest, matched) = 
        alt( (
                register::parse_reg_set_2,
                parse_immediate,
                parse_indirect,
                parse_dp,
                parse_indexed,
                expr::parse_expr,
               ))(input)?;

    Ok((rest, matched))
}

fn parse_opcode_with_arg(input: Span) -> IResult< Node> {
    let (rest, (op, arg)) = separated_pair(opcode_token,
                                 multispace1, parse_opcode_arg)(input)?;

    let item = Item::OpCode(op.to_string());
    let node = Node::from_item(item)
        .with_child(arg)
        .with_pos(input, rest);

    Ok((rest, node))
}

fn parse_opcode_no_arg(input: Span) -> IResult< Node> {
    use Item::*;
    let (rest, text) = opcode_token(input)?;
    let ret = Node::from_item(OpCode(text.to_string())).with_pos(input, rest);
    Ok((rest,ret))
}

pub fn parse_opcode(input: Span) -> IResult< Node> {
    let (rest, item) = alt((parse_opcode_with_arg, parse_opcode_no_arg))(input)?;
    Ok((rest, item.with_pos(input, rest)))
}


#[allow(unused_imports)]
mod test {

    use std::os::unix::prelude::JoinHandleExt;

    use emu::cpu::RegEnum;
    use pretty_assertions::{assert_eq, assert_ne};
    use crate::locate::Position;

    use super::*;

    #[test]
    fn test_opcode_reg_list() {
        use Item::*;
        use emu::cpu::RegEnum::*;

        let op_text = "pshu a,b,d,x,y";

        let (_rest, matched) = parse_opcode_with_arg(op_text.as_span()).unwrap();

        let set  = vec![A,B,D,X,Y].into_iter().collect();
        let des_node = Node::from_item_item(OpCode("pshu".to_owned()),RegisterSet(set));

        assert_eq!(matched, des_node);

        let op_text = "pshu a,b,d,x,y,y";
        let res = parse_opcode_with_arg(op_text.as_span());

        if let Ok(( _,matched )) = &res {
            println!("{:#?}",matched);
            println!("{:#?}",matched.children);
        } else {
            println!("{:?}", res);
        }

        assert!(res.is_err())
    }

    #[test]
    fn test_opcode_immediate() {
        let op_text = "lda #100";
        let (_rest, matched) = parse_opcode_with_arg(op_text.as_span()).unwrap();

        let oc = "lda";
        let num = 100;

        let des_node = Node::from_item(Item::OpCode(oc.to_string()));
        let des_arg = Node::from_item(Item::Immediate).with_child(Node::from_number(num));
        let des_node = des_node.with_child(des_arg);

        assert_eq!(matched, des_node);
    }

    #[test]
    fn test_parse_immediate() {
        let op_text = "#$100+10";
        let op_text = op_text.as_span();

        let num_p = 1;
        let plus_p = num_p+4;
        let ten_p = plus_p+1;
        let last_p = op_text.len();

        let res = parse_immediate(op_text);
        assert!(res.is_ok());

        let des_arg = vec![
            Node::from_number(256).with_upos(num_p,plus_p),
            Node::from_item(Item::Add).with_child(Node::from_number(10).with_upos(ten_p,last_p)).with_upos(5, last_p)
        ];

        let des_expr = Node::from_item(Item::Expr).with_children(des_arg).with_upos(num_p,last_p);
        let desired = Node::from_item(Item::Immediate).with_child(des_expr).with_upos(0,last_p);
        let (_, matched) = res.unwrap();

        assert_eq!(matched,desired);
    }
    fn simple_indexed(op : &str, middle : &str, index: &str, reg : RegEnum) {
        use emu::cpu::RegEnum::*;
        use Item::*;
        let op_text = format!("{}{}{}",op,middle, index);
        let op_text = op_text.as_span();
        let res = parse_indexed(op_text);
        assert!(res.is_ok());
        println!("line: {:?}", op_text.to_string());

        let op_start = 0;
        let middle_start = op.len();
        let index_start = middle_start + middle.len();
        let end = index_start + index.len();

        let des_args = vec![
            Node::from_number(0).with_ctx(Position::from_usize((op_start,middle_start))),
            Node::from_item(Item::Register(reg)).with_ctx(Position::from_usize((index_start,end))),
        ];

        let desired = Node::from_item(Item::Indexed)
            .with_children(des_args)
            .with_ctx(Position::from_usize(( 0,end))) ;

        let (_, matched) = res.unwrap();
        assert_eq!(matched, desired);
    }

    #[test]
    fn test_simple_indexed() {
        use emu::cpu::RegEnum::*;
        use Item::*;

        let op = "0";
        let middle = ",";
        let index = "X";
        simple_indexed(op, middle, index, X);

        let op = "";
        let middle = ",";
        let index = "X";
        simple_indexed(op, middle, index,X);

        let op = "";
        let middle = ",";
        let index = "Y";
        simple_indexed(op, middle, index,Y);
    }

    fn test_item<'a, F>(mut parse : F, input : &'a str, des : &Item) -> Item
        where
            F : nom::Parser<Span<'a>,Node,ParseError<'a>>
    {
        let input = input.as_span();
        let res = parse.parse(input);
        assert!(res.is_ok());

        let (_, res) = res.unwrap();
        assert_eq!(*res.item(), *des);
        res.item().clone()
    }

    #[test]
    fn test_pre_post_dec() {

        use emu::cpu::RegEnum::*;
        use Item::*;

        let input = "--X";
        let des = DoublePreDecrement(X);
        let p = parse_pre_dec_dec;
        test_item(p, input, &des);

        let input = "-X";
        let des = PreDecrement(X);
        let p = parse_pre_dec;
        test_item(p, input, &des);

        let input = "+X";
        let des = PreIncrement(X);
        let p = parse_pre_inc;
        test_item(p, input, &des);

        let input = "++X";
        let des = DoublePreIncrement(X);
        let p = parse_pre_inc_inc;
        test_item(p, input, &des);

        let input = "X--";
        let des = DoublePostDecrement(X);
        let p = parse_post_dec_dec;
        test_item(p, input, &des);

        let input = "X-";
        let des = PostDecrement(X);
        let p = parse_post_dec;
        test_item(p, input, &des);

        let input = "X+";
        let des = PostIncrement(X);
        let p = parse_post_inc;
        test_item(p, input, &des);

        let input = "X++";
        let des = DoublePostIncrement(X);
        let p = parse_post_inc_inc;
        test_item(p, input, &des);
    }
}
