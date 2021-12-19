use super::util;
use super::item::Item;

use nom::branch::alt;
use nom::IResult;
use nom::bytes::complete::tag_no_case;
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::multi::{ many1, separated_list1,separated_list0,  };
use nom::character::complete::{
    alpha1, alphanumeric1, anychar, char as nom_char, line_ending, multispace0, multispace1,
    not_line_ending, one_of, satisfy, space1,
};

use nom::bytes::complete::tag;

// Register parsing

fn get_reg(input: &str) -> IResult<&str, emu::cpu::RegEnum> {
    let (rest, matched) = alt((
            tag_no_case("dp"),
            tag_no_case("cc"),
            tag_no_case("pc"),
            tag_no_case("a"),
            tag_no_case("b"),
            tag_no_case("x"),
            tag_no_case("y"),
            tag_no_case("u"),
            tag_no_case("s"),
            tag_no_case("d")))(input)?;

    use emu::cpu::RegEnum::*;

    let matched_lower = String::from(matched).to_ascii_lowercase();

    let reg = match matched_lower.as_str() {
            "dp" => DP,
            "cc" => CC,
            "pc" => PC,
            "a" => A,
            "b" => B,
            "x" => X,
            "y" => Y,
            "u" => U,
            "s" => S,
            "d" => D,
        _ => panic!("Should not happen"),
    };

    Ok((rest, reg))
}

fn parse_reg(input: &str) -> IResult<&str, Item> {
    let (rest,matched) = get_reg(input)?;
    Ok((rest, Item::Register(matched)))
}

fn get_reg_list(input: &str) -> IResult<&str, Vec<emu::cpu::RegEnum>> {
    let sep = tuple((multispace0, tag(util::LIST_SEP), multispace0));
    let (rest, matched) = separated_list1(sep, get_reg)(input)?;
    Ok((rest, matched))
}

fn parse_reg_list(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = get_reg_list(input)?;
    Ok((rest, Item::RegisterList(matched)))
}

fn parse_reg_set(_input: &str) -> IResult<&str, Item> {
    todo!()
}

mod test {

    use pretty_assertions::{assert_eq, assert_ne};
    use super::*;

    #[test]
    fn test_register() {
        let res = parse_reg("A");
        let des = emu::cpu::RegEnum::A;
        assert_eq!(res, Ok(("", Item::Register(des))));

        let res = parse_reg("dp");
        let des = emu::cpu::RegEnum::DP;
        assert_eq!(res, Ok(("", Item::Register(des))));
    }

    #[test]
    fn test_register_list() {
        use emu::cpu::RegEnum::*;

        let res = parse_reg_list("A,X,Y");
        let des = vec![A,X,Y];
        assert_eq!(res, Ok(("", Item::RegisterList(des))));

        let res = parse_reg_list("");
        assert!(res.is_err());

        let res = parse_reg_list("A");
        let des = vec![A];
        assert_eq!(res, Ok(("", Item::RegisterList(des))));

        let res = parse_reg_list("A, x, y, u, S, DP, cc, D, dp");
        let des = vec![A, X, Y, U, S, DP, CC, D, DP];
        assert_eq!(res, Ok(("", Item::RegisterList(des))));
    }

}
