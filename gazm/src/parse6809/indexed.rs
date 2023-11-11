use super::register::{get_index_reg, get_pc_reg, get_reg};

use crate::{
    item6809::IndexParseType,
    error::IResult,
    parse::expr::parse_expr,
    item::{Item, Node},
    parse::locate::{matched_span, Span},
    parse::util::{wrapped_chars, ws},
    item6809::MC6809::{self, OperandIndexed},
};

use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::multispace0,
    sequence::{pair, preceded, separated_pair, tuple},
};

// Addr Modes and Parsing Order
//  ,--R    SubSub(RegEnum)         parse_pre_dec_dec
//  ,-R     Sub(RegEnum)            parse_pre_dec       *NOT ALLOWED INDIRECT*
//  ,R++    PlusPlus(RegEnum)       parse_post_inc
//  ,R+     Plus(RegEnum)           parse_post_inc
//  ,R      Zero(RegEnum)           parse_zero          *NOT ALLOWED INDIRECT*
// A,R      AddA(RegEnum)           parse_add_a
// B,R      AddB(RegEnum)           parse_add_b
// D,R      AddD(RegEnum)           parse_add_d
// n,PC     PCOffset                parse_pc_offset
// n,R      ConstantOffset(RegEnum) parse_offset

// Post inc / dec
fn get_post_inc(input: Span) -> IResult<IndexParseType> {
    let (rest, (_, _, matched, _)) =
        tuple((tag(","), multispace0, get_index_reg, tag("+")))(input)?;

    let index_type = IndexParseType::Plus(matched);
    Ok((rest, index_type))
}

fn get_post_inc_inc(input: Span) -> IResult<IndexParseType> {
    let (rest, (_, _, matched, _)) =
        tuple((tag(","), multispace0, get_index_reg, tag("++")))(input)?;
    let index_type = IndexParseType::PlusPlus(matched);
    Ok((rest, index_type))
}

fn get_pre_dec_dec(input: Span) -> IResult<IndexParseType> {
    let (rest, (_, _, _, matched)) =
        tuple((tag(","), multispace0, tag("--"), get_index_reg))(input)?;
    let index_type = IndexParseType::SubSub(matched);
    Ok((rest, index_type))
}

fn get_pre_dec(input: Span) -> IResult<IndexParseType> {
    let (rest, (_, _, _, matched)) =
        tuple((tag(","), multispace0, tag("-"), get_index_reg))(input)?;
    let index_type = IndexParseType::Sub(matched);
    Ok((rest, index_type))
}

fn get_zero(input: Span) -> IResult<IndexParseType> {
    let sep = pair(tag(","), multispace0);
    let (rest, matched) = preceded(sep, get_reg)(input)?;
    let index_type = IndexParseType::Zero(matched);
    Ok((rest, index_type))
}

fn get_add_a(input: Span) -> IResult<IndexParseType> {
    let sep = separated_pair(multispace0, tag(","), multispace0);
    let (rest, (_, matched)) = separated_pair(tag_no_case("a"), sep, get_index_reg)(input)?;
    let index_type = IndexParseType::AddA(matched);
    Ok((rest, index_type))
}

fn get_add_b(input: Span) -> IResult<IndexParseType> {
    let sep = separated_pair(multispace0, tag(","), multispace0);
    let (rest, (_, matched)) = separated_pair(tag_no_case("b"), sep, get_index_reg)(input)?;
    let index_type = IndexParseType::AddB(matched);
    Ok((rest, index_type))
}

fn get_add_d(input: Span) -> IResult<IndexParseType> {
    let sep = separated_pair(multispace0, tag(","), multispace0);
    let (rest, (_, matched)) = separated_pair(tag_no_case("d"), sep, get_index_reg)(input)?;
    let index_type = IndexParseType::AddD(matched);
    Ok((rest, index_type))
}

fn get_no_arg_indexed(input: Span) -> IResult<IndexParseType> {
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
    Ok((rest, matched))
}
#[allow(dead_code)]
fn get_no_arg_indexed_allowed_indirect(input: Span) -> IResult<IndexParseType> {
    let (rest, matched) = alt((
        get_pre_dec_dec,
        get_post_inc_inc,
        get_zero,
        get_add_a,
        get_add_b,
        get_add_d,
    ))(input)?;
    Ok((rest, matched))
}

fn parse_offset(input: Span) -> IResult<Node> {
    let sep = separated_pair(multispace0, tag(","), multispace0);
    let (rest, (expr, reg)) = separated_pair(parse_expr, sep, get_index_reg)(input)?;

    let offset = IndexParseType::ConstantOffset(reg);
    let ctx = matched_span(input, rest);
    let item = MC6809::operand_from_index_mode(offset, false);

    let matched = Node::from_item_span(item, ctx).with_child(expr);
    Ok((rest, matched))
}

fn parse_pc_offset(input: Span) -> IResult<Node> {
    let sep = ws(tag(","));
    let (rest, (expr, _)) = separated_pair(parse_expr, sep, get_pc_reg)(input)?;

    let offset = IndexParseType::PCOffset;
    let ctx = matched_span(input, rest);
    let item = MC6809::operand_from_index_mode(offset, false);

    let matched = Node::from_item_span(item, ctx).with_child(expr);
    Ok((rest, matched))
}

fn parse_extended_indirect(input: Span) -> IResult<Node> {
    let (rest, matched) = wrapped_chars('[', ws(parse_expr), ']')(input)?;
    let item = MC6809::operand_from_index_mode(IndexParseType::ExtendedIndirect, false);
    let ctx = matched_span(input, rest);
    let matched = Node::from_item_span(item, ctx).with_child(matched);
    Ok((rest, matched))
}

fn parse_no_arg_indexed(input: Span) -> IResult<Node> {
    let (rest, matched) = get_no_arg_indexed(input)?;
    let ctx = matched_span(input, rest);
    let matched = Node::from_item_span(OperandIndexed(matched, false), ctx);
    Ok((rest, matched))
}

fn parse_no_arg_indexed_allowed_indirect(input: Span) -> IResult<Node> {
    use crate::error::parse_failure;
    let (rest, matched) = get_no_arg_indexed(input)?;
    let ctx = matched_span(input, rest);

    match matched {
        IndexParseType::Plus(_) => {
            let err = "Post-increment indexing not valid indirectly";
            Err(parse_failure(err, ctx))
        }
        IndexParseType::Sub(_) => {
            let err = "Pre-decrement indexing not valid indirectly";
            Err(parse_failure(err, ctx))
        }
        _ => {
            let matched = Node::from_item_span(OperandIndexed(matched, false), ctx);
            Ok((rest, matched))
        }
    }
}

fn parse_indexed_indirect(input: Span) -> IResult<Node> {
    let indexed_indirect = alt((
        parse_no_arg_indexed_allowed_indirect,
        parse_pc_offset,
        parse_offset,
    ));
    let (rest, mut matched) = wrapped_chars('[', ws(indexed_indirect), ']')(input)?;

    if let Item::Cpu( OperandIndexed(amode, _) ) = matched.item {
        matched.item = OperandIndexed(amode, true).into();
    } else {
        panic!("Should not happen")
    };

    Ok((rest, matched))
}

fn parse_indexed_direct(input: Span) -> IResult<Node> {
    let mut indexed = alt((parse_no_arg_indexed, parse_pc_offset, parse_offset));
    indexed(input)
}

pub fn parse_indexed(input: Span) -> IResult<Node> {
    alt((
        parse_indexed_indirect,
        parse_extended_indirect,
        parse_indexed_direct,
    ))(input)
}

////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
#[cfg(test)]
mod test {
    use super::*;
    use emu6809::cpu::RegEnum::*;
    use nom::combinator::{all_consuming, opt, recognize};
    use nom::multi::many0_count;
    use pretty_assertions::{assert_eq, assert_ne};

    fn test_get_no_arg_indexed() {
    }

    #[test]
    fn test_fails() {
        let to_try = vec!["[,-Y]", "[,Y+]", "100,y+", "a,100"];

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
            (false, ",--Y", IndexParseType::SubSub(Y)),
            (false, ",-U", IndexParseType::Sub(U)),
            (false, ",Y", IndexParseType::Zero(Y)),
            (false, "A,X", IndexParseType::AddA(X)),
            (false, "B,U", IndexParseType::AddB(U)),
            (false, "D,U", IndexParseType::AddD(U)),
            (false, ",--X", IndexParseType::SubSub(X)),
            (false, ",S++", IndexParseType::PlusPlus(S)),
            (false, ",S+", IndexParseType::Plus(S)),
            (false, "100,PC", IndexParseType::PCOffset),
            (false, "100,U", IndexParseType::ConstantOffset(U)),
            (true, "[100,U]", IndexParseType::ConstantOffset(U)),
            (true, "[,--Y]", IndexParseType::SubSub(Y)),
            (true, "[ ,Y ]", IndexParseType::Zero(Y)),
            (true, "[ A,X ]", IndexParseType::AddA(X)),
            (true, "[ B,U ]", IndexParseType::AddB(U)),
            (true, "[ D,U ]", IndexParseType::AddD(U)),
            (true, "[ ,--X ]", IndexParseType::SubSub(X)),
            (true, "[ ,S++ ]", IndexParseType::PlusPlus(S)),
            (true, "[ 100,PC ]", IndexParseType::PCOffset),
            (true, "[ 100,U ]", IndexParseType::ConstantOffset(U)),
        ];

        for (indirect, input, desired) in to_try {
            let desired = MC6809::operand_from_index_mode(desired, indirect);
            // println!("Testing {} -> {:?}", input, desired);
            let res = parse_indexed(input.into());
            // println!("{:#?}", res);
            let (_, matched) = res.unwrap();
            assert_eq!(matched.item, desired);
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
