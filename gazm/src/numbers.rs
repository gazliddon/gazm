use nom::AsBytes;
use nom::branch::alt;
use nom::bytes::complete::{is_a, tag};
use nom::combinator::recognize;
use nom::multi::many1;

use nom::character::complete::{ alphanumeric1, anychar };
use nom::sequence::preceded;
use emu::utils::sources::AsmSource;

use crate::error::{IResult, ParseError};
use crate::locate::Span;

fn num_get(input: Span) -> IResult<Span> {
    recognize(many1(alt((alphanumeric1, is_a("_")))))(input)
}

fn num_parse_err(input: Span, radix: &str, e: std::num::ParseIntError) -> nom::Err<ParseError> {
    let e = format!("Parsing {}: {}", radix, e);
    nom::Err::Error(ParseError::new(e, &input, true))
}

fn get_hex(input: Span) -> IResult<i64> {
    let (rest, _) = alt((tag("0x"), tag("0X"), tag("$")))(input)?;
    let (rest, num_str) = num_get(rest)?;

    let num = i64::from_str_radix(&num_str.replace('_', ""), 16)
        .map_err(|e| num_parse_err(num_str, "hex", e))?;

    Ok((rest, num))
}

fn get_binary(input: Span) -> IResult<i64> {
    let (rest, _) = alt((tag("%"), tag("0b"), tag("0B")))(input)?;
    let (rest, num_str) = num_get(rest)?;
    let num = i64::from_str_radix(&num_str.replace('_', ""), 2)
        .map_err(|e| num_parse_err(num_str, "binary", e))?;

    Ok((rest, num))
}

fn get_char(input: Span) -> IResult<i64> {
    let (rest,matched) = preceded(tag("'"), anychar)(input)?;
    let (rest,_) = tag("'")(rest)?;
    let mut s = String::new();
    s.push(matched);
    let num_bytes = s.as_bytes();
    let ret = num_bytes[0];
    Ok((rest,ret as i64))
}

fn get_dec(input: Span) -> IResult<i64> {
    let (rest, num_str) = num_get(input)?;

    let num = num_str
        .replace('_', "")
        .parse::<i64>()
        .map_err(|e| num_parse_err(num_str, "Decimal", e))?;

    Ok((rest, num))
}

pub fn get_number(input: Span) -> IResult<i64> {
    alt((get_hex, get_binary, get_dec, get_char))(input)
}

pub fn get_number_err(input: &str) -> Result<isize, String> {
    let x = AsmSource::FromStr;
    let s = Span::new_extra(input, x);
    let n = get_number(s);

    match n {
        Err(_) => Err(format!("Couldn't parse {input} as a number")),
        Ok((_, num)) => Ok(num as isize),
    }
}

pub fn get_number_err_usize(input: &str) -> Result<usize, String> {
    let x = get_number_err(input)?;
    if x < 0 {
        Err(format!("{} doesn't map to a usize", x))
    } else {
        Ok(x.try_into().unwrap())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests

#[allow(unused_imports)]
mod test {

    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    use lazy_static::lazy_static;

    lazy_static! {
        static ref TEST_BIN: Vec<(&'static str, i64)> = vec![
            ("0b111", 7),
            ("0b1111111", 127),
            ("0b101_01010", 0xaa),
            ("0b10010001", 0x91),
            ("0b101_0101010010001", 0xaa91),
        ];
        static ref TEST_HEX: Vec<(&'static str, i64)> = vec![
            ("0xffff", 0xffff),
            ("0x12", 0x12),
            ("$abcd", 0xabcd),
            ("0X0", 0),
        ];
        static ref TEST_DEC: Vec<(&'static str, i64)> = vec![
            ("8723872", 8723872),
            ("4096", 4096),
            ("12", 12),
            ("0___0_112210", 112210),
        ];
        static ref TEST_ALL: Vec<(&'static str, i64)> = {
            let mut all = vec![];
            all.extend(TEST_BIN.iter());
            all.extend(TEST_HEX.iter());
            all.extend(TEST_DEC.iter());
            all
        };
    }

    fn test_nums<F>(arr: &[(&'static str, i64)], func: F)
    where
        F: Fn(Span) -> IResult<i64>,
    {
        for (input, desired) in arr.iter() {
            let res = func((*input).into());
            println!("Testing: {:?}", input);

            if let Ok((_, number)) = res {
                assert_eq!(number, *desired)
            } else {
                println!("Could not parse {} {:?}", input, res);
                assert!(res.is_ok())
            }
        }
    }

    #[test]
    fn test_bin() {
        test_nums(&TEST_BIN, get_binary);
    }

    #[test]
    fn text_hex() {
        test_nums(&TEST_HEX, get_hex);
    }
    #[test]
    fn test_dec() {
        println!("Testing decimal");
        test_nums(&TEST_DEC, get_dec);
    }

    #[test]
    fn test_all() {
        test_nums(&TEST_ALL, get_number);
    }
}
