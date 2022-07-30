use nom::branch::alt;
use nom::bytes::complete::{is_a, tag};
use nom::combinator::recognize;
use nom::multi::many1;
use nom::AsBytes;

use emu::utils::sources::AsmSource;
use nom::character::complete::{alphanumeric1, anychar};
use nom::sequence::preceded;

use crate::error::{IResult, ParseError};
use crate::locate::Span;

mod new {

    use nom::branch::alt;
    use nom::bytes::complete::{is_a, tag};
    use nom::character::complete::{alphanumeric1, anychar, hex_digit0, hex_digit1};
    use nom::character::is_hex_digit;
    use nom::combinator::recognize;
    use nom::error::ErrorKind;
    use nom::error::ParseError;
    use nom::error::context;
    use nom::error::ContextError;
    use nom::multi::many1;
    use nom::sequence::preceded;
    use nom::AsChar;
    use nom::Compare;
    use nom::CompareResult;
    use nom::Err;
    use nom::FindToken;
    use nom::IResult;
    use nom::InputIter;
    use nom::InputLength;
    use nom::InputTake;
    use nom::InputTakeAtPosition;
    use nom::Offset;
    use nom::Slice;
    use nom::UnspecializedInput;

    use std::ops::{RangeFrom, RangeTo};

    pub fn is_ok_char<T, E>(input: T, ok_chars: &str) -> IResult<T, T, E>
    where
        T: InputTakeAtPosition,
        <T as InputTakeAtPosition>::Item: AsChar,
        E: ParseError<T>,
    {
        input.split_at_position1_complete(
            |item| {
                let c = item.as_char();

                for ok_c in ok_chars.chars() {
                    if c == ok_c {
                        return true;
                    }
                }
                false
            },
            ErrorKind::Fail,
        )
    }
    fn is_one_of_these<I,E>(input: &str) -> impl Fn(I) -> IResult<I, I, E>
        where
        E: ParseError<I>,
        I: ITrait<I>,
        // I: for<'a> Compare<&'a str>,
        <I as InputIter>::Item: AsChar,
    { 
        let cmp = String::from(input);

        move |input : I| {
            is_ok_char(input, &cmp)
        }
    }

    pub fn char_2<T, I, Error: ParseError<I>>(c: T) -> impl Fn(I) -> IResult<I, char, Error>
    where
        T: AsChar + Clone,
        I: Slice<RangeFrom<usize>> + InputIter,
        <I as InputIter>::Item: AsChar,
    {
        let to_match = c.as_char();

        move |i: I| match (i).iter_elements().next().map(|t| {
            let b = t.as_char() == to_match;
            (&to_match, b)
        }) {
            Some((c, true)) => Ok((i.slice(c.len()..), c.as_char())),
            _ => Err(Err::Error(Error::from_char(i, to_match))),
        }
    }

    pub fn is_num_char<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
    where
        T: InputTakeAtPosition,
        <T as InputTakeAtPosition>::Item: AsChar,
    {
        is_ok_char(input, "0123456789abcdef_")
    }

    trait ITrait<I>:
        Slice<RangeTo<usize>>
        + Offset
        + Clone
        + UnspecializedInput
        + InputTake
        + InputIter
        + InputLength
    {
    }

    #[derive(Copy, Clone, Debug)]
    enum NumberLiteralKind {
        Hex,
        Decimal,
        Binary,
    }

    impl NumberLiteralKind {
        fn base(&self) -> usize {
            match self {
                Self::Hex => 16, 
                Self::Decimal => 10,
                Self::Binary => 2,
            }
        }
        fn valid_chars(&self) -> &'static str {
            match self {
                Self::Hex => "abcdefABCDEF0123456789_", 
                Self::Decimal => "0123456789_",
                Self::Binary => "01_",
            }
        }
    }

    struct Number<I> {
        kind: NumberLiteralKind,
        val: i64,
        prefix_text : I,
        number_text: I,
    }

    impl<I: Clone> Number<I> {
        pub fn new(kind: NumberLiteralKind, val : i64, prefix_text : &I, number_text : &I) -> Self {
            Number {
                kind,
                val,
                prefix_text: prefix_text.clone(), 
                number_text: number_text.clone(),
            }
        }
    }

    // Num getter
    // prefix parser
    // conversion routine
    // kind

    fn hex_get<I,E>() -> impl Fn(I) -> IResult<I, Number<I>, E>
        where
        E: ContextError<I> + ParseError<I>,
        I: ITrait<I>,
        I: for<'a> Compare<&'a str>,
        <I as InputIter>::Item: AsChar,
    {
        move | input : I| {

            let p = is_one_of_these("0123456789abcdefABCDEF_");

            let (input, prefix) = alt((tag("0x"), tag("$")))(input)?;
            let (input, matched) = context("hex digits", p)(input)?;

            let mut res : i64 = 0;

            for c in matched.iter_elements().map(|i| i.as_char()).filter(|c| *c == '-') {
                if let Some(v) = c.to_digit(16) {
                    res = ( res << 8 ) + v as i64;
                } else {
                    panic!()
                }
            }

            let ret  = Number::new(NumberLiteralKind::Hex, res, &prefix, &matched);
            Ok((input,ret))
        }
    }

    fn to_radix<I,E>(input : I, r : NumberLiteralKind) -> IResult<I, i64, E> 
        where
        E: ContextError<I> + ParseError<I>,
        I: ITrait<I>,
        I: for<'a> Compare<&'a str>,
        <I as InputIter>::Item: AsChar,

    {
            let base = r.base() as i64;
            let mut res : i64 = 0;

            for c in input.iter_elements().map(|i| i.as_char()).filter(|c| *c == '_') {
                if let Some(v) = c.to_digit(base as u32) {
                    res = ( res * base ) + v as i64;
                } else {
                    panic!()
                }
            }

            Ok((input,res))
    }

    fn num_get<I,E>(lit : NumberLiteralKind) -> impl Fn(I) -> IResult<I, Number<I>, E>
        where
        E: ContextError<I> + ParseError<I>,
        I: ITrait<I>,
        I: for<'a> Compare<&'a str>,
        <I as InputIter>::Item: AsChar,
    {
        move | input : I| {
            let mut prefix_chars_p = match lit {
                NumberLiteralKind::Decimal => alt((tag(""), tag(""))),
                NumberLiteralKind::Hex => alt((tag("0x"), tag("%"))),
                NumberLiteralKind::Binary => alt((tag("0b"), tag("%"))),
            };

            let num_chars_p = is_one_of_these(lit.valid_chars());

            let (input, prefix) = prefix_chars_p(input)?;
            let (input, matched) = context("digits", num_chars_p)(input)?;
            let (input,res) = to_radix(input,lit)?;
            let ret  = Number::new(lit, res, &prefix, &matched);
            Ok((input,ret))
        }
    }
}

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
    let (rest, matched) = preceded(tag("'"), anychar)(input)?;
    let (rest, _) = tag("'")(rest)?;
    let mut s = String::new();
    s.push(matched);
    let num_bytes = s.as_bytes();
    let ret = num_bytes[0];
    Ok((rest, ret as i64))
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
