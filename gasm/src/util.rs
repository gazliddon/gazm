use crate::item::{ Item, Node };
use crate::numbers;
use crate::labels;
use crate::expr::{self, parse_expr};

use crate::error::{IResult, ParseError};
use crate::locate::{Span, matched_span, mk_span};

use nom::{InputTake, Offset};
// use nom::error::ParseError;
use nom::bytes::complete::{
    escaped,
    tag,
    tag_no_case,
    take_while,
};

use nom::character::complete::{
    char as nom_char, multispace0, multispace1,
    one_of, 
};
use nom::multi::separated_list1;
use nom::sequence::{preceded, separated_pair, terminated, tuple};
use nom::combinator::cut;
use nom_locate::position;

pub static LIST_SEP: & str = ",";

pub fn ws<'a,F,O,E>(mut inner : F) -> impl FnMut(Span<'a>) -> IResult<O>
where
  F: nom::Parser<Span<'a>, O,ParseError<'a>>
{
    move |input : Span| -> IResult<O>{
        let (input, _) = multispace0(input)?;
        let (input, matched) = inner.parse(input)?;
        let (input, _) = multispace0(input)?;
        Ok((input, matched))
    }
}
// pub fn parse_str<'a, O, F>(
//     mut inner: F
//     ) -> impl FnMut(&str) -> IResult<O>
// where
// F: nom::Parser<Span<'a>, O,ParseError<'a>>
// {
//     move |input: &'a str| {
//         inner.parse(input.into())
//     }
// }

pub fn wrapped_chars<'a, O, F>(
    open: char,
    mut inner: F,
    close : char,
    ) -> impl FnMut(Span<'a>) -> IResult<O>
where
F: nom::Parser<Span<'a>, O,ParseError<'a>>
{
    move |input: Span| {
        let (input,_) = nom_char(open)(input)?;
        let (input,matched) = inner.parse(input)?;
        let (input,_) = nom_char(close)(input)?;
        Ok((input, matched))
    }
}

pub fn sep_list1<'a, F, O>(
    inner: F
    ) -> impl FnMut(Span<'a>) -> IResult<Vec<O>>
where
F: nom::Parser<Span<'a>, O,ParseError<'a>> + Copy {
    move |input: Span| {
        let sep = tuple((multispace0, tag(LIST_SEP), multispace0));
        separated_list1(sep, inner)(input)
    }
}

pub fn parse_assignment(input: Span) -> IResult< Node> {
    use labels::parse_label;

    let sep = tuple((multispace1,tag_no_case("equ"), multispace1));

    let (rest, (label,arg)) = separated_pair(parse_label, sep, cut(parse_expr))(input)?;

    let matched_span = matched_span(input, rest);

    let ret = Node::from_item(Item::Assignment, matched_span)
        .with_children(vec![label, arg]);

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

////////////////////////////////////////////////////////////////////////////////
// Number

pub fn parse_escaped_str(input: Span) -> IResult< Item> {
    let (rest, matched) = match_escaped_str(input)?;
    Ok((rest, Item::QuotedString(matched.to_string())))
}


pub fn parse_number<'a>(input: Span<'a>) -> IResult< Node> {
    let (rest, num) = numbers::number_token(input)?;
    let matched = super::locate::matched_span(input,rest);
    let ret = Node::from_number(num, matched);
    Ok((rest, ret))
}

////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {

    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn test_parse_number() {
        let input = Span::new("1000 ;; ");
        let (rest,matched) = parse_number(input).unwrap();
        assert_eq!(*rest, " ;; ");
        assert_eq!(&matched.to_string(), "1000");
    }

    #[test]
    fn test_assignment() {
        let input = Span::new("hello equ $1000");
        let (_,matched) = parse_assignment(input).unwrap();
        assert_eq!(&matched.to_string(), "hello equ 4096");

        let input = Span::new("hello equ * ;;");
        let (rest,matched) = parse_assignment(input).unwrap();
        println!("original: {:?}", *input);
        println!("Rest: {:?}", rest);
        println!("Po: {:?}", matched.ctx);
        assert_eq!(*rest, " ;;");
        assert_eq!(&matched.to_string(),"hello equ *");
    }
}


