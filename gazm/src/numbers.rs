use crate::{
    error::{IResult, ParseError},
    locate::Span,
    item::ParsedFrom,
};

use emu::utils::sources::AsmSource;

use nom::{
    branch::alt,
    bytes::complete::{is_a, tag},
    character::complete::{alphanumeric1, anychar},
    combinator::recognize,
    multi::many1,
    sequence::preceded,
    AsBytes, UnspecializedInput,
};


mod new {
    pub enum Literal {
        Int(NumberLiteral),
        QuotedString(String),
        Character(char),
    }
    use nom::{
        branch::alt,
        bytes::complete::{is_a, tag, tag_no_case},
        character::complete::{alphanumeric1, anychar, hex_digit0, hex_digit1},
        character::is_hex_digit,
        combinator::recognize,
        error::context,
        error::ContextError,
        error::ErrorKind,
        error::ParseError,
        multi::many1,
        sequence::preceded,
        InputTake,
    };

    use nom_locate::LocatedSpan;

    pub type Span<'a, X = ()> = LocatedSpan<&'a str, X>;
    pub type IResult<'a, X, O> = nom::IResult<Span<'a, X>, O>;

    #[derive(Clone, Debug, PartialEq)]
    pub enum NumberLiteralKind {
        Decimal,
        Hexadecimal,
        Binary,
        Character,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct NumberLiteral {
        pub kind: NumberLiteralKind,
        pub val: i64,
    }

    impl NumberLiteral {
        pub fn new(val : i64, kind: NumberLiteralKind) -> Self {
            Self {
                val, kind
            }
        }

    }

    fn num_get<X: Clone>(input: Span<X>) -> IResult<X, Span<X>> {
        recognize(many1(alt((alphanumeric1, is_a("_")))))(input)
    }

    fn get_hex<X: Clone>(input: Span<X>) -> IResult<X, (NumberLiteralKind, i64)> {
        let (rest, _) = alt((tag("0x"), tag("$")))(input)?;
        let (rest, num_str) = num_get(rest)?;
        let num = i64::from_str_radix(&num_str.replace('_', ""), 16).map_err(|_| panic!())?;

        Ok((rest, (NumberLiteralKind::Hexadecimal, num)))
    }

    fn get_bin<X: Clone>(input: Span<X>) -> IResult<X, (NumberLiteralKind, i64)> {
        let (rest, _) = alt((tag("0b"), tag("%")))(input)?;
        let (rest, num_str) = num_get(rest)?;
        let num = i64::from_str_radix(&num_str.replace('_', ""), 2).map_err(|_| panic!())?;
        Ok((rest, (NumberLiteralKind::Binary, num)))
    }

    fn get_dec<X: Clone>(input: Span<X>) -> IResult<X, (NumberLiteralKind, i64)> {
        let (rest, num_str) = num_get(input)?;
        let num = num_str.replace('_', "").parse::<i64>().map_err(|_| panic!())?;
        Ok((rest, (NumberLiteralKind::Decimal, num)))
    }

    /// Parse a span into a NumberLiteral
    pub fn parse_number<X: Clone>(input: Span<X>) -> IResult<X, (Span<X>, NumberLiteral)> {
        let (rest, (kind, val)) = alt((get_hex, get_bin, get_dec))(input.clone())?;
        let span = input.take(input.len() - rest.len());
        Ok((rest, (span, NumberLiteral { val, kind })))
    }
}

// pub fn tag<T, Input, Error: ParseError<Input>>(
// tag: T,
// ) -> impl Fn(Input) -> IResult<Input, Input, Error>
// where
// Input: InputTake + Compare<T>,
// T: InputLength + Clone,
// {
// move |i: Input| {
//     let tag_len = tag.input_len();
//     let t = tag.clone();
//     let res: IResult<_, _, Error> = match i.compare(t) {
//     CompareResult::Ok => Ok(i.take_split(tag_len)),
//     _ => {
//         let e: ErrorKind = ErrorKind::Tag;
//         Err(Err::Error(Error::from_error_kind(i, e)))
//     }
//     };
//     res
// }
// }

// pub fn separated_list1<I, O, O2, E, F, G>(
//     mut sep: G,
//     mut f: F,
// ) -> impl FnMut(I) -> nom::IResult<I, Vec<O>, E>
// where
//     I: Clone + nom::InputLength,
//     F: nom::Parser<I, O, E>,
//     G: nom::Parser<I, O2, E>,
//     E: nom::error::ParseError<I>,
// {
//     panic!()
// }

// pub type IResult<'a, O> = nom::IResult<Span<'a>, O, ParseError>;

// fn num_get(input: Span) -> IResult<Span> {
//     recognize(many1(alt((alphanumeric1, is_a("_")))))(input)
// }

mod newp {
    use super::*;
    use nom_locate::LocatedSpan;

    type Span<'a, X> = LocatedSpan<&'a str, X>;
    type IResult<'a, O, X, E> = nom::IResult<Span<'a, X>, O, E>;
    use nom::error::ParseError;

    pub fn num_get<'a, X, E>(input: Span<'a, X>) -> IResult<'a, Span<X>, X, E>
    where
        E: nom::error::ParseError<Span<'a, X>>,
        X: Clone,
    {
        recognize(many1(alt((alphanumeric1, is_a("_")))))(input)
    }

    pub fn get_char<'a, X, E>(input: Span<'a, X>) -> IResult<'a, i64, X, E>
    where
        E: nom::error::ParseError<Span<'a, X>>,
        X: Clone,
    {
        let (rest, matched) = preceded(tag("'"), anychar)(input)?;
        let (rest, _) = tag("'")(rest)?;
        let mut s = String::new();
        s.push(matched);
        let num_bytes = s.as_bytes();
        let num_of_bytes = num_bytes[0] as i64;
        Ok((rest, num_of_bytes))
    }
}

fn num_get(input: Span) -> IResult<Span> {
    newp::num_get(input)
    // recognize(many1(alt((alphanumeric1, is_a("_")))))(input)
}

fn num_parse_err(input: Span, radix: &str, e: std::num::ParseIntError) -> nom::Err<ParseError> {
    let e = format!("Parsing {radix}: {e}");
    nom::Err::Error(ParseError::new(e, &input, true))
}

fn get_hex(input: Span) -> IResult<( i64, ParsedFrom )> {
    let (rest, _) = alt((tag("0x"), tag("0X"), tag("$")))(input)?;
    let (rest, num_str) = num_get(rest)?;

    let num = i64::from_str_radix(&num_str.replace('_', ""), 16)
        .map_err(|e| num_parse_err(num_str, "hex", e))?;

    Ok((rest, ( num, ParsedFrom::Hex )))
}

fn get_binary(input: Span) -> IResult<( i64, ParsedFrom )> {
    let (rest, _) = alt((tag("%"), tag("0b"), tag("0B")))(input)?;
    let (rest, num_str) = num_get(rest)?;
    let num = i64::from_str_radix(&num_str.replace('_', ""), 2)
        .map_err(|e| num_parse_err(num_str, "binary", e))?;

    Ok((rest, ( num, ParsedFrom::Bin )))
}

fn get_char(input: Span) -> IResult<( i64, ParsedFrom )> {
    let (rest, matched) = preceded(tag("'"), anychar)(input)?;
    let (rest, _) = tag("'")(rest)?;
    let mut s = String::new();
    s.push(matched);
    let num_bytes = s.as_bytes();
    let num_of_bytes = num_bytes[0];
    Ok((rest, ( num_of_bytes as i64, ParsedFrom::Char(matched) )))
}

fn get_dec(input: Span) -> IResult<( i64, ParsedFrom )> {
    let (rest, num_str) = num_get(input)?;

    let num = num_str
        .replace('_', "")
        .parse::<i64>()
        .map_err(|e| num_parse_err(num_str, "Decimal", e))?;

    Ok((rest, ( num, ParsedFrom::Dec )))
}

pub fn get_number(input: Span) -> IResult<( i64, ParsedFrom )> {
    alt((get_hex, get_binary, get_dec, get_char))(input)
}


////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {

    use super::{*, new::{NumberLiteralKind, NumberLiteral}};
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
            ("8723872", 8_723_872),
            ("4096", 4096),
            ("12", 12),
            ("0___0_112210", 112_210),
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
        F: Fn(Span) -> IResult<( i64, ParsedFrom  )>,
    {
        for (input, desired) in arr.iter() {
            let res = func((*input).into());
            println!("Testing: {input:?}");

            if let Ok((_, ( number,_ ))) = res {
                assert_eq!(number, *desired)
            } else {
                println!("Could not parse {input} {res:?}");
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
    struct Test {}

    fn test_it(txt: &str, expected : NumberLiteral) {
        use new::*;

        let span = Span::new(txt);
        let (_, (sp, lit)) = parse_number(span).expect("Parse failure");
        assert_eq!(lit, expected);
        assert_eq!(&sp.to_string(), txt);
    }

    #[test]
    fn test_new() {
        test_it("$101010", NumberLiteral::new(0x101010, NumberLiteralKind::Hexadecimal));
        test_it("%101010", NumberLiteral::new(0b101010, NumberLiteralKind::Binary));
        test_it("0b101010", NumberLiteral::new(0b101010, NumberLiteralKind::Binary));
        test_it("$ff",  NumberLiteral::new(255,NumberLiteralKind::Hexadecimal));
    }
}
