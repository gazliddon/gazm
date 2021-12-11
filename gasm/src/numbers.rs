use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::one_of;
use nom::combinator::map_res;
use nom::combinator::recognize;
use nom::multi::{many0, many1};
use nom::sequence::{preceded, terminated};
use nom::IResult;

use nom::character::complete::char as nom_char;

pub fn parse_hex(input: &str) -> IResult<&str, (u64, &str)> {
    let num_tags = (tag("0x"), tag("0X"), tag("$"));
    let valid_chars = "0123456789abcdefABCDEF";
    let conv = |matched| u64::from_str_radix(&str::replace(matched, "_", ""), 16).unwrap();

    let (rest, matched) = preceded(
        alt(num_tags),
        recognize(many1(terminated(one_of(valid_chars), many0(nom_char('_'))))),
    )(input)?;

    Ok((rest, (conv(matched), matched)))
}

fn parse_binary(input: &str) -> IResult<&str, (u64, &str)> {
    let num_tags = (tag("0b"), tag("0B"));
    let valid_chars = "01";
    let conv = |matched| u64::from_str_radix(&str::replace(matched, "_", ""), 2).unwrap();

    let (rest, matched) = preceded(
        alt(num_tags),
        recognize(many1(terminated(one_of(valid_chars), many0(nom_char('_'))))),
    )(input)?;

    Ok((rest, (conv(matched), matched)))
}

fn parse_dec(input: &str) -> IResult<&str, (u64, &str)> {
    use nom::character::complete::char;
    let (rest, text_str) =
        recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(input)?;
    let num = u64::from_str_radix(&str::replace(text_str, "_", ""), 10).unwrap();
    Ok((rest, (num, text_str)))
}

pub fn parse_number(input: &str) -> IResult<&str, (u64, &str)> {
    alt((parse_hex, parse_binary, parse_dec))(input)
}

////////////////////////////////////////////////////////////////////////////////
// Tests

mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    use lazy_static::lazy_static;

    lazy_static! {
        static ref TEST_BIN: Vec<(&'static str, u64)> = vec![
            ("0b111", 7),
            ("0b1111111", 127),
            ("0b101_01010", 0xaa),
            ("0b10010001", 0x91),
            ("0b101_0101010010001", 0xaa91),
        ];
        static ref TEST_HEX: Vec<(&'static str, u64)> = vec![
            ("0xffff", 0xffff),
            ("0x12", 0x12),
            ("$abcd", 0xabcd),
            ("0X0", 0),
        ];
        static ref TEST_DEC: Vec<(&'static str, u64)> = vec![
            ("8723872", 8723872),
            ("4096", 4096),
            ("12", 12),
            ("0___0_112210", 0___0_112210),
        ];
        static ref TEST_ALL: Vec<(&'static str, u64)> = {
            let mut all = vec![];
            all.extend(TEST_BIN.iter());
            all.extend(TEST_HEX.iter());
            all.extend(TEST_DEC.iter());
            all
        };
    }

    fn test_nums<F>(input: &Vec<(&'static str, u64)>, func: F)
    where
        F: Fn(&str) -> IResult<&str, ( u64, &str )>,
    {
        for (input, desired) in input.iter() {
            let (_, ( number, _text )) = func(input).unwrap();
            println!("Desired: {:x} ", desired);
            println!("Matched: {:x} ", number);
            assert_eq!(number, *desired);
        }
    }

    #[test]
    fn test_bin() {
        test_nums(&TEST_BIN, parse_binary);
    }

    #[test]
    fn text_hex() {
        test_nums(&TEST_HEX, parse_hex);
    }
    #[test]
    fn test_dec() {
        test_nums(&TEST_DEC, parse_dec);
    }

    #[test]
    fn test_all() {
        test_nums(&TEST_ALL, parse_number);
    }
}
