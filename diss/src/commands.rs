use nom::{InputTake, branch::alt, bytes::complete::*, character::complete::*, combinator::recognize, combinator::{opt, rest}, multi::many1, sequence::{preceded, separated_pair}};

use nom_locate::LocatedSpan;

pub type IResult<'a, O> = nom::IResult<Span<'a>, O, ParseError<'a>>;
pub type Span<'a> = LocatedSpan<&'a str, ()>;

pub enum Command {
    Diss(Option<isize>),
    Mem(Option<isize>),
    Help,
    Quit,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParseError<'a> {
    pub span: Span<'a>,
    pub message: Option<String>,
    pub failure: bool,
}

impl<'a> std::fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.message)
    }
}

impl<'a> std::error::Error for ParseError<'a> {}

impl<'a> From<nom::Err<ParseError<'a>>> for ParseError<'a> {
    fn from(i: nom::Err<ParseError<'a>>) -> Self {
        match i {
            nom::Err::Incomplete(_) => panic!(),
            nom::Err::Error(mut e) => {
                e.failure = false;
                e
            }
            nom::Err::Failure(mut e) => {
                e.failure = true;
                e
            }
        }
    }
}
impl<'a> ParseError<'a> {
    pub fn new(message: String, span: Span<'a>, failure: bool) -> ParseError<'a> {
        Self {
            span,
            message: Some(message),
            failure,
        }
    }
}

impl<'a> nom::error::ParseError<Span<'a>> for ParseError<'a> {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        Self::new(format!("parse error {:?}", kind), input, false)
    }

    fn append(_input: Span, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: Span<'a>, c: char) -> Self {
        Self::new(format!("unexpected character '{}'", c), input, false)
    }
}

fn num_parse_err<'a>(
    input: Span<'a>,
    radix: &str,
    e: std::num::ParseIntError,
) -> nom::Err<ParseError<'a>> {
    let e = format!("Parsing {}: {}", radix, e);
    nom::Err::Error(ParseError::new(e, input, true))
}

fn num_get(input: Span) -> IResult<Span> {
    recognize(many1(alt((alphanumeric1, is_a("_")))))(input)
}

fn get_hex(input: Span) -> IResult<isize> {
    let (rest, _) = alt((tag("0x"), tag("0X"), tag("$")))(input)?;
    let (rest, num_str) = num_get(rest)?;
    let num = isize::from_str_radix(&num_str.replace('_', ""), 16)
        .map_err(|e| num_parse_err(num_str, "hex", e))?;

    Ok((rest, num))
}

fn get_binary(input: Span) -> IResult<isize> {
    let (rest, _) = alt((tag("%"), tag("0b"), tag("0B")))(input)?;
    let (rest, num_str) = num_get(rest)?;
    let num = isize::from_str_radix(&num_str.replace('_', ""), 2)
        .map_err(|e| num_parse_err(num_str, "binary", e))?;

    Ok((rest, num))
}

fn get_dec(input: Span) -> IResult<isize> {
    let (rest, num_str) = num_get(input)?;

    let num = num_str
        .replace('_', "")
        .parse::<isize>()
        .map_err(|e| num_parse_err(num_str, "Decimal", e))?;

    Ok((rest, num))
}

pub fn get_number(input: Span) -> IResult<isize> {
    alt((get_hex, get_binary, get_dec))(input)
}

fn parse_diss(input: Span) -> IResult<Command> {
    let arg = preceded(many1(char(' ')), get_number);
    let (rest, matched) = preceded( tag("d"),opt(arg))(input)?;
    Ok((rest, Command::Diss(matched)))
}

fn parse_mem(input: Span) -> IResult<Command> {
    let arg = preceded(many1(char(' ')), get_number);
    let (rest, matched) = preceded( tag("m"),opt(arg))(input)?;
    Ok((rest, Command::Mem(matched)))
}

fn parse_quit(input: Span) -> IResult<Command> {
    let (rest,_) = tag("h")(input)?;
    Ok((rest, Command::Quit ))
}

fn parse_help(input: Span) -> IResult<Command> {
    let (rest,_) = tag("q")(input)?;
    Ok((rest, Command::Quit ))
}

pub fn parse_command(input: Span) -> IResult<Command> {
    alt((parse_diss, parse_mem, parse_quit, parse_help))(input)
}
