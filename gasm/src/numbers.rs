use nom::branch::alt;
use nom::bytes::complete::{ tag, is_a };
use nom::combinator::{ recognize, opt,  };
use nom::multi::{many0, many1};
use nom::sequence::{preceded, terminated, pair, tuple};

use nom::character::complete::{alpha1, alphanumeric1, char as nom_char, one_of};

use crate::error::{IResult, ParseError};
use crate::locate::Span;

fn num_get(input : Span) -> IResult<Span> {
    recognize(many1(alt((alphanumeric1, is_a("_")))))(input)
}

fn num_parse_err<'a>(input : Span<'a>, radix : &str, e : std::num::ParseIntError ) -> nom::Err<ParseError<'a>> {
    let e = format!("Parsing {}: {}",radix, e);
    nom::Err::Error(ParseError::new(e, &input))
}

pub fn parse_hex(input: Span) -> IResult<i64> {
    let (rest,_) = alt(( tag("0x"), tag("0X"), tag("$") ))(input)?;
    let (rest,num_str) = num_get(rest)?;

    let num = i64::from_str_radix(&num_str.replace("_", ""), 16)
        .map_err(|e| num_parse_err(num_str, "hex", e))?;

    Ok((rest, num))
}

fn parse_binary(input: Span) -> IResult<i64> {
    let (rest,_) = alt(( tag("%"),tag("0b"), tag("0B") ))(input)?;
    let (rest,num_str) = num_get(rest)?;
    let num = i64::from_str_radix(&num_str.replace( "_", ""), 2)
        .map_err(|e| num_parse_err(num_str, "binary", e))?;

    Ok((rest, num))
}

fn parse_dec(input: Span) -> IResult<i64> {
    let (rest, num_str) = num_get(input)?;

    let num = i64::from_str_radix(&num_str.replace( "_", ""), 10)
        .map_err(|e| num_parse_err(num_str, "Decimal", e))?;

    Ok((rest, num))
}

pub fn number_token(input: Span) -> IResult<i64> {
    alt((parse_hex, parse_binary, parse_dec))(input)
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

    fn test_nums<F>(arr: &[ (&'static str, i64) ], func: F)
        where
            F: Fn(Span) -> IResult<i64>,
        {
            for (input, desired) in arr.iter() {
                let span = Span::new("Test");
                let res = func(span);

                if let Ok(( _, number )) = res {
                    assert_eq!(number, *desired)
                } else {
                    println!("Could not parse {} {:?}", input, res);
                    assert!(res.is_ok())
                }
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
        println!("Testing decimal");
        test_nums(&TEST_DEC, parse_dec);
    }

    #[test]
    fn test_all() {
        test_nums(&TEST_ALL, number_token);
    }
}
