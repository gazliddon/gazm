use crate::register::parse_index_reg;
use crate::locate::{ Span, matched_span };
use super::item::{ Item,Node };
use super::error::{IResult, ParseError};
use super::register::get_reg;
use super::expr::parse_expr;

use emu::cpu::RegEnum;
use nom::bytes::complete::{ tag, tag_no_case };
use nom::sequence::{ pair, preceded, separated_pair, terminated, tuple};
use nom::combinator::{ recognize, opt, all_consuming };
use nom::branch::alt;

use nom::character::complete::{
    alpha1, multispace0, multispace1
};

#[derive(Debug, PartialEq, Clone)]
pub enum IndexParseType {
    ConstantOffset(RegEnum),    //               arg,R
    Plus(RegEnum),     //               ,R+              2 0 |
    PlusPlus(RegEnum), //               ,R++             3 0 |
    Sub(RegEnum),      //               ,-R              2 0 |
    SubSub(RegEnum),   //               ,--R             3 0 |
    Zero(RegEnum),     //               ,R               0 0 |
    AddB(RegEnum),     //             (+/- B),R          1 0 |
    AddA(RegEnum),     //             (+/- A),R          1 0 |
    AddD(RegEnum),     //             (+/- D),R          4 0 |
    PCOffset,            //      (+/- 7 bit offset),PC     1 1 |
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

// The new stuff!

mod new {
    use std::ops::Index;
    use crate::register::{ get_index_reg, get_pc_reg };

    use crate::expr::parse_expr;
    use crate::locate::matched_span;

    use super::tag_no_case;

    use nom::sequence::separated_pair;

    use super::RegEnum;
    use super::alt;
    use super::IResult;
    use super::Span;
    use super::tag;
    use super::get_reg;
    use super::terminated;
    use super::preceded;
    use super::multispace0;
    use super::Node;
    use super::IndexParseType;
    use super::tuple;
    use super::pair;
    use super::Item;


    // Post inc / dec
    pub fn get_post_inc(input: Span) -> IResult<IndexParseType> {
        let (rest, (_,_,matched,_)) = tuple((
                tag(","),
                multispace0,
                get_index_reg ,
                tag("+")))(input)?;

        let ret = IndexParseType::Plus(matched);
        Ok((rest,ret))
    }

    pub fn get_post_inc_inc(input: Span) -> IResult<IndexParseType> {
        let (rest, (_,_,matched,_)) = tuple((
                tag(","),
                multispace0,
                get_index_reg ,
                tag("++")))(input)?;
        let ret = IndexParseType::PlusPlus(matched);
        Ok((rest,ret))
    }

    pub fn get_pre_dec_dec(input: Span) -> IResult<IndexParseType> {
        let (rest, (_,_,_,matched)) = tuple((
                tag(","),
                multispace0,
                tag("--"),
                get_index_reg ,
                ))(input)?;
        let ret = IndexParseType::SubSub(matched);
        Ok((rest, ret))
    }

    pub fn get_pre_dec(input: Span) -> IResult<IndexParseType> {
        let (rest, (_,_,_,matched)) = tuple((
                tag(","),
                multispace0,
                tag("-"),
                get_index_reg,
                ))(input)?;
        let ret = IndexParseType::Sub(matched);
        Ok((rest, ret))
    }

    pub fn get_zero(input: Span) -> IResult<IndexParseType> {
        let sep = pair(tag(","), multispace0);
        let (rest, matched) = preceded(sep, get_reg )(input)?;
        let ret = IndexParseType::Zero(matched);
        Ok((rest, ret))
    }

    pub fn get_add_a(input : Span) -> IResult<IndexParseType> {
        let sep = separated_pair(multispace0, tag(","), multispace0);
        let (rest, (_,matched)) = separated_pair(tag_no_case("a"), sep, get_index_reg)(input)?;
        let ret = IndexParseType::AddA(matched);
        Ok((rest, ret))
    }

    pub fn get_add_b(input : Span) -> IResult<IndexParseType> {
        let sep = separated_pair(multispace0, tag(","), multispace0);
        let (rest, (_,matched)) = separated_pair(tag_no_case("b"), sep, get_index_reg)(input)?;
        let ret = IndexParseType::AddB(matched);
        Ok((rest, ret))
    }

    pub fn get_add_d(input : Span) -> IResult<IndexParseType> {
        let sep = separated_pair(multispace0, tag(","), multispace0);
        let (rest, (_,matched)) = separated_pair(tag_no_case("d"), sep, get_index_reg)(input)?;
        let ret = IndexParseType::AddD(matched);
        Ok((rest, ret))
    }

    pub fn get_no_arg_indexed(input : Span) -> IResult<IndexParseType> {
        let (rest, matched) = alt((
                get_pre_dec_dec,
                get_post_inc_inc,
                get_pre_dec,
                get_post_inc,
                get_zero,
                get_add_a,
                get_add_b,
                get_add_d,
                ))(input)?;
        Ok((rest,matched))
    }
    pub fn get_no_arg_indexed_allowed_indirect(input : Span) -> IResult<IndexParseType> {
        let (rest, matched) = alt((
                get_pre_dec_dec,
                get_post_inc_inc,
                get_zero,
                get_add_a,
                get_add_b,
                get_add_d,
                ))(input)?;
        Ok((rest,matched))
    }

    pub fn parse_offset(input: Span) -> IResult<Node> {
        let sep = separated_pair(multispace0, tag(","), multispace0);
        let (rest,(expr,reg)) = separated_pair(parse_expr, sep, get_index_reg)(input)?;

        let offset = IndexParseType::ConstantOffset(reg);
        let ctx = matched_span(input, rest);

        let matched = Node::from_item(Item::IndexMode(offset), ctx).with_child(expr);
        Ok((rest,matched))
    }

    pub fn parse_pc_offset(input: Span) -> IResult<Node> {
        use crate::util::ws;

        let sep = ws(tag(","));
        let (rest,(expr,reg)) = separated_pair(parse_expr, sep, get_pc_reg)(input)?;

        let offset = IndexParseType::ConstantOffset(reg);
        let ctx = matched_span(input, rest);

        let matched = Node::from_item(Item::IndexMode(offset), ctx).with_child(expr);
        Ok((rest,matched))
    }

    fn parse_no_arg_indexed(input: Span) -> IResult<Node> {
        let (rest, matched) = get_no_arg_indexed(input)?;
        let ctx = matched_span(input, rest);
        let matched = Node::from_item(Item::IndexMode(matched), ctx);
        Ok((rest,matched))
    }

    fn parse_no_arg_indexed_allowed_indirect(input: Span) -> IResult<Node> {
        use crate::error::{error, failure};
        let (rest, matched) = get_no_arg_indexed(input)?;
        let ctx = matched_span(input, rest);
        use crate::error::ParseError;

        match matched {
                IndexParseType::Plus(_) => {
                    let err = format!("Post-increment indexing not valid indirectly");
                    Err(failure(&err, ctx))
                },
                IndexParseType::Sub(_) => {
                    let err = format!("Pre-decrement indexing not valid indirectly");
                    Err(failure(&err, ctx))
                },
            _=> {
                let matched = Node::from_item(Item::IndexMode(matched), ctx);
                Ok((rest,matched))
            },
        }
    }

    fn parse_indexed_indirect(input: Span) -> IResult<Node> {
        use crate::util::{ wrapped_chars,ws };
        let indexed_indirect = alt((parse_no_arg_indexed_allowed_indirect,parse_pc_offset, parse_offset));
        wrapped_chars('[', ws(indexed_indirect),']')(input)
    }
    fn parse_indexed_direct(input: Span) -> IResult<Node> {
        let mut indexed = alt((parse_no_arg_indexed,parse_pc_offset, parse_offset));
        indexed(input)
    }

    pub fn parse_indexed(input: Span) -> IResult<Node> {
        use crate::util::{ wrapped_chars,ws };
        alt((parse_indexed_indirect,parse_indexed_direct))(input)
    }

    // Addr Modes and Parsing Order
    //  ,--R    SubSub(RegEnum)         parse_pre_dec_dec
    //  ,-R     Sub(RegEnum)            parse_pre_dec
    //  ,R++    PlusPlus(RegEnum)       parse_post_inc
    //  ,R+     Plus(RegEnum)           parse_post_inc
    //  ,R      Zero(RegEnum)           parse_zero
    // A,R      AddA(RegEnum)           parse_add_a
    // B,R      AddB(RegEnum)           parse_add_b
    // D,R      AddD(RegEnum)           parse_add_d
    // n,PC     PCOffset                parse_pc_offset
    // n,R      ConstantOffset(RegEnum) parse_offset

    // Indirect Parsing Order
    //  ,R--    SubSub(RegEnum)
    //  ,R++    PlusPlus(RegEnum)
    //  ,R      Zero(RegEnum)
    // A,R      AddA(RegEnum)
    // B,R      AddB(RegEnum)
    // D,R      AddD(RegEnum)
    // n,R      ConstantOffset(RegEnum)
    // n,PC     PCOffset
}

////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {
    use super::IndexParseType;
    use super::new::*;
    use nom::multi::many0_count;
    use pretty_assertions::{assert_eq, assert_ne};
    use emu::cpu::RegEnum::*;
    use nom::combinator::{ recognize, opt, all_consuming };

    #[test]
    fn test_get_no_arg_indexed() {
        let to_try = vec![
            (",--Y", IndexParseType::SubSub(Y)),
            (",-U", IndexParseType::Sub(U)),
            (",Y", IndexParseType::Zero(Y)),
            ("A,X", IndexParseType::AddA(X)),
            ("B,U", IndexParseType::AddB(U)),
            ("D,U", IndexParseType::AddD(U)),
            (",--X", IndexParseType::SubSub(X)),
            (",S++", IndexParseType::PlusPlus(S)),
            (",S+", IndexParseType::Plus(S)),
        ];

        for (input, desired) in to_try {
            println!("Testing {} -> {:?}", input, desired);
            let (_, matched) = get_no_arg_indexed(input.into()).unwrap();
            assert_eq!(matched, desired);
        }
    }

    #[test]
    fn test_fails() {
        let to_try = vec![
            "[,-Y]",
            "[,Y+]",
            "100,y+",
            "a,100",
        ];

        for input in to_try {
            println!("Testing {}", input);
            let res = all_consuming(parse_indexed)(input.into());
            println!("{:?}", res);
            assert!(res.is_err())
        }
    }

    #[test]
    fn test_parse_indexed() {
        let to_try = vec![
            (",--Y", IndexParseType::SubSub(Y)),
            (",-U", IndexParseType::Sub(U)),
            (",Y", IndexParseType::Zero(Y)),
            ("A,X", IndexParseType::AddA(X)),
            ("B,U", IndexParseType::AddB(U)),
            ("D,U", IndexParseType::AddD(U)),
            (",--X", IndexParseType::SubSub(X)),
            (",S++", IndexParseType::PlusPlus(S)),
            (",S+", IndexParseType::Plus(S)),

            ("[,--Y]", IndexParseType::SubSub(Y)),
            ("[ ,Y ]", IndexParseType::Zero(Y)),
            ("[ A,X ]", IndexParseType::AddA(X)),
            ("[ B,U ]", IndexParseType::AddB(U)),
            ("[ D,U ]", IndexParseType::AddD(U)),
            ("[ ,--X ]", IndexParseType::SubSub(X)),
            ("[ ,S++ ]", IndexParseType::PlusPlus(S)),
        ];

        for (input, desired) in to_try {
            use crate::item::Item;
            println!("Testing {} -> {:?}", input, desired);
            let (_, matched) = parse_indexed(input.into()).unwrap();
            if let Item::IndexMode(index_type) = matched.item {
                assert_eq!(index_type, desired);
            } else {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_zero() {
        let test = ",U";
        let desired = IndexParseType::Zero(U);
        let (_, matched) = get_zero(test.into()).unwrap();
        assert_eq!(matched, desired);
    }

    #[test]
    fn test_get_pre_dec_dec() {
        let test = ",--Y";
        let desired = IndexParseType::SubSub(Y);
        let (_, matched) = get_pre_dec_dec(test.into()).unwrap();
        assert_eq!(matched, desired);
    }

    #[test]
    fn test_get_pre_dec() {
        let test = ",-Y";
        let desired = IndexParseType::Sub(Y);
        let (_, matched) = get_pre_dec(test.into()).unwrap();
        assert_eq!(matched, desired);
    }

    #[test]
    fn test_get_post_inc() {
        let test = ",X+";
        let desired = IndexParseType::Plus(X);
        let (_, matched) = get_post_inc(test.into()).unwrap();
        assert_eq!(matched, desired);
    }

    #[test]
    fn test_get_post_inc_inc() {
        let test = ",S++";
        let desired = IndexParseType::PlusPlus(S);
        let (_, matched) = get_post_inc_inc(test.into()).unwrap();
        assert_eq!(matched, desired);
    }
    #[test]
    fn test_get_zero() {
        let test = ",S";
        let desired = IndexParseType::Zero(S);
        let (_, matched) = get_zero(test.into()).unwrap();
        assert_eq!(matched, desired);
    }
    #[test]
    fn test_get_add_a() {
        let test = "A,S";
        let desired = IndexParseType::AddA(S);
        let (_, matched) = get_add_a(test.into()).unwrap();
        assert_eq!(matched, desired);
    }
    #[test]
    fn test_get_add_b() {
        let test = "B,S";
        let desired = IndexParseType::AddB(S);
        let (_, matched) = get_add_b(test.into()).unwrap();
        assert_eq!(matched, desired);
    }
    #[test]
    fn test_get_add_d() {
        let test = "D,Y";
        let desired = IndexParseType::AddD(Y);
        let (_, matched) = get_add_d(test.into()).unwrap();
        assert_eq!(matched, desired);
    }
}
