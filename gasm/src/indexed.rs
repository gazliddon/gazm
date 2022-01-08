use crate::register::parse_index_reg;
use crate::locate::{ Span, matched_span };
use super::item::{ Item,Node };
use super::error::{IResult, ParseError};
use super::register::get_reg;
use super::expr::parse_expr;

use emu::cpu::RegEnum;
use nom::bytes::complete::tag;
use nom::sequence::{ pair, preceded, separated_pair, terminated, tuple};
use nom::combinator::{ recognize, opt };
use nom::branch::alt;
use nom::character::complete::{
    alpha1, multispace0, multispace1
};



#[derive(Debug, PartialEq, Clone)]
pub enum IndexMode {
    ROff(RegEnum),      //               
    RPlus(RegEnum),     //               ,R+              2 0 |
    RPlusPlus(RegEnum), //               ,R++             3 0 |
    RSub(RegEnum),      //               ,-R              2 0 |
    RSubSub(RegEnum),   //               ,--R             3 0 |
    RZero(RegEnum),     //               ,R               0 0 |
    RAddB(RegEnum),     //             (+/- B),R          1 0 |
    RAddA(RegEnum),     //             (+/- A),R          1 0 |
    RAddi8(RegEnum),    //    (+/- 7 b  it offset),R      1 1 |
    RAddi16(RegEnum),   //      (+/- 15 bit offset),R     4 2 |
    RAddD(RegEnum),     //             (+/- D),R          4 0 |
    PCAddi8,            //      (+/- 7 bit offset),PC     1 1 |
    PCAddi16,           //      (+/- 15 bit offset),PC    5 2 |
    Ea,
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

pub fn parse_indexed(input : Span) -> IResult< Node> {
    use emu::isa::AddrModeEnum;
    use Item::*;
    use super::util;
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));

    let (rest, (expr,reg)) = separated_pair(
        opt(parse_expr),
        sep,
        parse_index_type
        )(input)?;

    let matched_span = matched_span(input, input);
    let zero = Node::from_item(Expr, matched_span).with_child(Node::from_number(0, input));

    let expr = expr.unwrap_or(zero);

    let ret = Node::from_item(Operand(AddrModeEnum::Indexed), input);

    let ret = ret.with_children(vec![expr, reg]);

    Ok((rest, ret))
}
