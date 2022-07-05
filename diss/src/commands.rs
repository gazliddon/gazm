use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use nom::{Compare, InputTake, branch::alt, bytes::complete::*, character::complete::*, combinator::{all_consuming, eof}, combinator::recognize, combinator::{not, opt, rest, cut}, multi::many1, sequence::{preceded, separated_pair, tuple}};

use nom::combinator::map;
use nom_locate::LocatedSpan;

pub type IResult<'a, O> = nom::IResult<Span<'a>, O, ParseError<'a>>;
pub type Span<'a> = LocatedSpan<&'a str, bool>;

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Diss(Option<isize>),
    Mem(Option<isize>),
    Help,
    Quit,
    Hex,
    Dec,
    LoadBin(PathBuf, usize),
    LoadSym(PathBuf),
    SetReg,
    Regs,
    Step,
    Reset,
    Go(isize),
}

////////////////////////////////////////////////////////////////////////////////
enum CommandEnums {
    Diss,
    Mem,
    Help,
    Quit,
    Hex,
    Dec,
    LoadBin,
    LoadSym,
}

enum ArgTypes {
    Number,
    Text,
    File,
}

enum ArgValues {
    Number(usize),
    Text(String),
    File(PathBuf),
}

/// Definition of command
pub struct CommandSpec {
    command: CommandEnums,
    /// Name for the command
    name: String,
    /// Abbreviation
    short: Option<String>,
    /// Type for each arg
    args: Vec<ArgTypes>,
    /// Minimum number of args needed
    /// Args at positions >= min_args are optional
    min_args: usize,
}

pub struct ParsedArgs {
    args: Vec<ArgValues>,
}

////////////////////////////////////////////////////////////////////////////////

use std::collections::HashMap;

pub struct Help {
    text: String,
}

lazy_static::lazy_static! {
    pub static ref HELP : HashMap<String,Help>  = {
        let mut m = HashMap::new();
        m.insert("diss".to_string(), Help { text: "".to_string() });
        m
    };
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

fn get_hex_no_tag(input: Span) -> IResult<isize> {
    let (rest, num_str) = num_get(input)?;
    let num = isize::from_str_radix(&num_str.replace('_', ""), 16)
        .map_err(|e| num_parse_err(num_str, "hex", e))?;

    Ok((rest, num))
}

fn get_hex(input: Span) -> IResult<isize> {
    let (rest, _) = alt((tag("0x"), tag("0X"), tag("$")))(input)?;
    get_hex_no_tag(rest)
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
    if input.extra {
        alt((get_hex, get_hex_no_tag, get_dec))(input)
    } else {
        alt((get_hex, get_binary, get_dec))(input)
    }
}

fn parse_diss(input: Span) -> IResult<Command> {
    let arg = preceded(space1, get_number);
    let (rest, matched) = preceded(tag("d"), opt(arg))(input)?;
    Ok((rest, Command::Diss(matched)))
}

fn parse_mem(input: Span) -> IResult<Command> {
    let arg = preceded(many1(char(' ')), get_number);
    let (rest, matched) = preceded(tag("m"), opt(arg))(input)?;
    Ok((rest, Command::Mem(matched)))
}

fn text_get(input: Span) -> IResult<Span> {
    preceded(space1, recognize(many1(anychar)))(input)
}

fn file_name_get(input: Span) -> IResult<PathBuf> {
    let (rest, matched) = text_get(input)?;

    let x : &str = matched.as_ref();
    let matched = shellexpand::tilde(x);

    Ok((rest, PathBuf::from_str(&matched).unwrap()))
}

fn parse_load_bin(input: Span) -> IResult<Command> {
    let arg = separated_pair(file_name_get, space1, get_number);
    let (rest, (file, addr)) = preceded(tuple((tag("lb"), space1)), arg)(input)?;
    Ok((rest, Command::LoadBin(file, addr as usize)))
}

fn parse_load_sym(input: Span) -> IResult<Command> {
    let (rest, file) = preceded(tag("load"), file_name_get)(input)?;
    Ok((rest, Command::LoadSym(file)))
}

fn parse_single(input: Span) -> IResult<Command> {
    all_consuming(alt((
        map(tag("hex"), |_| Command::Hex),
        map(tag("dec"), |_| Command::Dec),
        map(tag("quit"), |_| Command::Quit),
        map(tag("step"), |_| Command::Step),
        map(tag("regs"), |_| Command::Regs),
        map(tag("reset"), |_| Command::Reset),
    )))(input)
}

pub fn parse_command(input: &str, default_hex: bool) -> IResult<Command> {
    let sp = Span::new_extra(input, default_hex);

    alt((
        parse_single,
        parse_diss,
        parse_load_sym,
        parse_load_bin,
        parse_mem,
    ))(sp)
}

