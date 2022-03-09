use crate::expr::parse_expr;
use crate::item::{Item, Node};
use crate::labels;
use crate::numbers;

use crate::error::{IResult, ParseError};
use crate::locate::{matched_span, Span};

use nom::bytes::complete::{escaped, is_not, tag, tag_no_case, take_while};

use nom::character::complete::{char as nom_char, multispace0, multispace1, one_of};
use nom::combinator::cut;
use nom::multi::{separated_list0, separated_list1};
use nom::sequence::{preceded, separated_pair, terminated, tuple};

pub static LIST_SEP: &str = ",";

pub fn ws<'a, F, O>(mut inner: F) -> impl FnMut(Span<'a>) -> IResult<O>
where
    F: nom::Parser<Span<'a>, O, ParseError>,
{
    move |input: Span| -> IResult<O> {
        let (input, _) = multispace0(input)?;
        let (input, matched) = inner.parse(input)?;
        let (input, _) = multispace0(input)?;
        Ok((input, matched))
    }
}

pub fn wrapped_chars<'a, O, F>(
    open: char,
    mut inner: F,
    close: char,
) -> impl FnMut(Span<'a>) -> IResult<O>
where
    F: nom::Parser<Span<'a>, O, ParseError>,
{
    move |input: Span| {
        let (input, _) = nom_char(open)(input)?;
        let (input, matched) = inner.parse(input)?;
        let (input, _) = nom_char(close)(input)?;
        Ok((input, matched))
    }
}

pub fn sep_list1<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Vec<O>>
where
    F: nom::Parser<Span<'a>, O, ParseError> + Copy,
{
    move |input: Span| {
        let sep = tuple((multispace0, tag(LIST_SEP), multispace0));
        separated_list1(sep, inner)(input)
    }
}

pub fn sep_list0<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Vec<O>>
where
    F: nom::Parser<Span<'a>, O, ParseError> + Copy,
{
    move |input: Span| {
        let sep = tuple((multispace0, tag(LIST_SEP), multispace0));
        separated_list0(sep, inner)(input)
    }
}

pub fn parse_assignment(input: Span) -> IResult<Node> {
    use labels::parse_label;

    let sep = tuple((multispace1, tag_no_case("equ"), multispace1));

    let (rest, (label, arg)) = ws(separated_pair(parse_label, sep, parse_expr))(input)?;

    let matched_span = matched_span(input, rest);

    let item = match label.item {
        Item::Label(name) => Item::Assignment(name),
        Item::LocalLabel(name) => Item::LocalAssignment(name),
        _ => panic!(),
    };

    let ret = Node::from_item_span(item, matched_span).with_child(arg);

    Ok((rest, ret))
}

////////////////////////////////////////////////////////////////////////////////
// Escaped string

pub fn match_str(input: Span) -> IResult<Span> {
    let term = "\"n\\";
    let body = take_while(move |c| !term.contains(c));
    escaped(body, '\\', one_of(term))(input)
}

pub fn match_escaped_str(input: Span) -> IResult<Span> {
    preceded(nom_char('\"'), cut(terminated(match_str, nom_char('\"'))))(input)
}
pub fn match_file_name(input: Span) -> IResult<Span> {
    let body = take_while(move |c| c != '"');
    wrapped_chars('"', body, '"')(input)
}

////////////////////////////////////////////////////////////////////////////////
// Number

pub fn parse_number(input: Span) -> IResult<Node> {
    let (rest, num) = numbers::number_token(input)?;
    let matched = super::locate::matched_span(input, rest);
    let ret = Node::from_number(num, matched);
    Ok((rest, ret))
}

////////////////////////////////////////////////////////////////////////////////
// Compile a string as a fake file

pub fn get_block(input: Span<'_>) -> IResult<Span> {
    ws(wrapped_chars('{', is_not("}"), '}'))(input)
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ByteSizes {
    Zero,
    Bits5(i8),
    Byte(i8),
    Word(i16),
}

impl ByteSizes {
    pub fn promote(&mut self) {
        *self = match self {
            Self::Zero => Self::Zero,
            Self::Bits5(v) => Self::Byte(*v),
            Self::Byte(v) => Self::Word(*v as i16),
            Self::Word(v) => Self::Word(*v as i16),
        };
    }
}

pub trait ByteSize {
    fn byte_size(&self) -> ByteSizes;
}

impl ByteSize for i64 {
    fn byte_size(&self) -> ByteSizes {
        let v = *self;
        if v == 0 {
            ByteSizes::Zero
        } else if v > -16 && v < 16 {
            ByteSizes::Bits5(v as i8)
        } else if v > -128 && v < 128 {
            ByteSizes::Byte(v as i8)
        } else {
            ByteSizes::Word(v as i16)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {

    use romloader::sources::AsmSource;

    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    #[test]
    fn test_byte_sizes() {
        let x: i64 = 7;
        let y = x.byte_size();
        assert_eq!(y, ByteSizes::Bits5(7))
    }

    #[test]
    fn test_parse_number() {
        let input = Span::new_extra("1000 ;; ", AsmSource::FromStr);
        let (rest, matched) = parse_number(input).unwrap();
        assert_eq!(*rest, " ;; ");
        assert_eq!(&matched.to_string(), "1000");
    }

    #[test]
    fn test_assignment() {
        let input = Span::new_extra("hello equ $1000", AsmSource::FromStr);
        let (_, matched) = parse_assignment(input).unwrap();
        assert_eq!(&matched.to_string(), "hello equ 4096");

        let input = Span::new_extra("hello equ * ;;", AsmSource::FromStr);
        let (rest, matched) = parse_assignment(input).unwrap();
        println!("original: {:?}", *input);
        println!("Rest: {:?}", rest);
        println!("Po: {:?}", matched.ctx);
        assert_eq!(&matched.to_string(), "hello equ *");
    }
}
