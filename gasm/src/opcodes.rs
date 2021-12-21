use crate::expr;
use crate::expr::parse_expr;
use crate::register;
use crate::register::get_reg;
use crate::register::parse_reg;
use crate::util::parse_not_sure;

use super::item::Item;
use super::util;
use super::numbers;
use emu::cpu::RegEnum;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{ digit0, digit1 };
use romloader::{Dbase, Instruction};

use nom::branch::alt;
use nom::IResult;
// use std::ascii::AsciiExt;
use std::collections::{HashMap, HashSet};
use std::num::IntErrorKind;
use nom::error::ErrorKind::NoneOf;
use nom::error::{Error, ParseError};

use nom::character::complete::{
    alpha1, alphanumeric1, anychar, char as nom_char, line_ending, multispace0, multispace1,
    not_line_ending, one_of, satisfy, space1,
};

use nom::bytes::complete::tag;
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::multi::{ many1, separated_list1,separated_list0,  };
use nom::combinator::{ recognize, opt };


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

    pub fn is_opcode(&self, input: &str) -> bool {
        self.get_opcode(input).is_some()
    }

    pub fn get_opcode(&self, input: &str) -> Option<&Vec<Instruction>> {
        let op = String::from(input).to_lowercase();
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

pub fn opcode_token(input: &str) -> IResult<&str, &str> {

    // Some opcodes have a number
    let (rest,matched) = recognize(pair(
            alpha1,digit0))(input)?;

    if OPCODES_REC.is_opcode(matched) {
        Ok((rest, matched))
    } else {
        Err(nom::Err::Error(Error::new(input, NoneOf)))
    }
}

fn parse_immediate(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = preceded(tag("#"), expr::parse_expr)(input)?;
    Ok((rest, Item::Immediate(Box::new(matched))))
}

fn parse_dp(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = preceded(tag("<"), expr::parse_expr)(input)?;
    Ok((rest, Item::DirectPage(Box::new(matched))))
}


// Post inc / dec
fn parse_post_inc(input: &str) -> IResult<&str,Item> {
    let (rest, matched) = terminated( get_reg , tag("+"))(input)?;
    Ok((rest, Item::PostIncrement(matched)))
}
fn parse_post_inc_inc(input: &str) -> IResult<&str,Item> {
    let (rest, matched) = terminated( get_reg , tag("++"))(input)?;
    Ok((rest, Item::DoublePostIncrement(matched)))
}
fn parse_post_dec(input: &str) -> IResult<&str,Item> {
    let (rest, matched) = terminated( get_reg , tag("-"))(input)?;
    Ok((rest, Item::PostDecrement(matched)))
}
fn parse_post_dec_dec(input: &str) -> IResult<&str,Item> {
    let (rest, matched) = terminated( get_reg , tag("--"))(input)?;
    Ok((rest, Item::DoublePostDecrement(matched)))
}


// Pre inc / dec
fn parse_pre_dec(input: &str) -> IResult<&str,Item> {
    let (rest, matched) = preceded(tag("-"), get_reg )(input)?;
    Ok((rest, Item::PreDecrement(matched)))
}

fn parse_pre_inc(input: &str) -> IResult<&str,Item> {
    let (rest, matched) = preceded(tag("+"), get_reg )(input)?;
    Ok((rest, Item::PreIncrement(matched)))
}

fn parse_pre_inc_inc(input: &str) -> IResult<&str,Item> {
    let (rest, matched) = preceded(tag("++"), get_reg )(input)?;
    Ok((rest, Item::DoublePreIncrement(matched)))
}

fn parse_pre_dec_dec(input: &str) -> IResult<&str,Item> {
    let (rest, matched) = preceded(tag("--"), get_reg )(input)?;
    Ok((rest, Item::DoublePreDecrement(matched)))
}

// Simple index
fn parse_simple_indexed(input : &str) -> IResult<&str, Item> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));

    let (rest, (expr,reg)) = separated_pair(
        opt(parse_expr),
        sep,
        alt((
                parse_pre_dec_dec,
                parse_pre_inc_inc,
                parse_pre_dec,
                parse_pre_inc,
                parse_post_dec_dec,
                parse_post_inc_inc,
                parse_post_dec,
                parse_post_inc,

                parse_reg  ))
        )(input)?;

    let expr = expr.unwrap_or(Item::Expr(vec![Item::Number(0)]));

    Ok((rest, Item::IndexedSimple(
                Box::new(expr),
                Box::new(reg))))
}

fn parse_indirect(input: &str) -> IResult<&str, Item> {
    use util::wrapped_chars;

    let (rest, matched) = wrapped_chars('[',
        alt((parse_simple_indexed,parse_expr))
    , ']')(input)?;

    Ok((rest, Item::Indirect(Box::new(matched))))
}

fn parse_opcode_arg(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = 
        alt( (
                register::parse_reg_list_2_or_more,
                parse_immediate,
                parse_indirect,
                parse_dp,
                parse_simple_indexed,
                expr::parse_expr,
                util::parse_not_sure,
               ))(input)?;

    Ok((rest, matched))
}

fn opcode_with_arg(input: &str) -> IResult<&str, Item> {

    let (rest, (op, arg)) = separated_pair(opcode_token,
                                 multispace1, parse_opcode_arg)(input)?;
    let arg = Box::new(arg);

    Ok((rest, Item::OpCodeWithArg(op, arg)))
}

fn opcode_no_arg(input: &str) -> IResult<&str, Item> {
    let (rest, text) = opcode_token(input)?;
    Ok((rest, Item::OpCode(text)))
}

pub fn parse_opcode(input: &str) -> IResult<&str, Item> {
    let (rest, item) = alt((opcode_with_arg, opcode_no_arg))(input)?;
    Ok((rest, item))
}

mod test {

    use pretty_assertions::{assert_eq, assert_ne};
    use super::*;

    #[test]
    fn test_opcode_immediate() {
        let res = opcode_with_arg("lda #100");

        let des_arg = Item::Expr(vec![
            Item::Number(100),
        ]);

        let desired = Item::OpCodeWithArg("lda", Box::new(Item::Immediate(Box::new( des_arg ))));
        assert_eq!(res, Ok(("", desired)));
    }

    #[test]
    fn test_parse_immediate() {

        let res = parse_immediate("#$100+10");

        let des_arg = Item::Expr(vec![
            Item::Number(256),
            Item::Op("+"),
            Item::Number(10),
        ]);

        let desired = Item::Immediate(Box::new(des_arg));
        assert_eq!(res, Ok(("", desired)));

    }
    #[test]
    fn test_simple_indexed() {
        use emu::cpu::RegEnum::*;
        use Item::*;

        let res = parse_simple_indexed("0,X");

        let des = IndexedSimple(Box::new(
                           Expr(
                               vec![Number(0)])),
                               Box::new(Register(X)));
        assert_eq!(res, Ok(("", des)));

        let res = parse_simple_indexed(",X");
        let des = IndexedSimple(Box::new(
                           Expr(
                               vec![Number(0)])),
                               Box::new(Register(X)));
        assert_eq!(res, Ok(("", des)));

    }

    #[test]
    fn test_pre_post_dec() {

        use emu::cpu::RegEnum::*;
        use Item::*;

        let res = parse_pre_dec_dec("--X");
        let des = DoublePreDecrement(X);
        assert_eq!(res, Ok(("", des)));

        let res = parse_pre_dec("-X");
        let des = PreDecrement(X);
        assert_eq!(res, Ok(("", des)));

        let res = parse_pre_inc("+X");
        let des = PreIncrement(X);
        assert_eq!(res, Ok(("", des)));

        let res = parse_pre_inc_inc("++X");
        let des = DoublePreIncrement(X);
        assert_eq!(res, Ok(("", des)));

        let res = parse_post_dec_dec("X--");
        let des = DoublePostDecrement(X);
        assert_eq!(res, Ok(("", des)));

        let res = parse_post_dec("X-");
        let des = PostDecrement(X);
        assert_eq!(res, Ok(("", des)));

        let res = parse_post_inc("X+");
        let des = PostIncrement(X);
        assert_eq!(res, Ok(("", des)));

        let res = parse_post_inc_inc("X++");
        let des = DoublePostIncrement(X);
        assert_eq!(res, Ok(("", des)));
    }


    #[test]
    fn test_opcode_with_expr() {

        let res = opcode_with_arg("lda $100");

        let des_arg = Item::Expr(vec![
            Item::Number(256)
        ]);

        let desired = Item::OpCodeWithArg("lda", Box::new(des_arg));
        assert_eq!(res, Ok(("", desired)));

        let res = opcode_with_arg("lda $100+256*10");

        let des_arg = Item::Expr(vec![
            Item::Number(256),
            Item::Op("+"),
            Item::Number(256),
            Item::Op("*"),
            Item::Number(10),
        ]);

        let desired = Item::OpCodeWithArg("lda", Box::new(des_arg));
        assert_eq!(res, Ok(("", desired)));
    }
}
