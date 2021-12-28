use crate::item::{ Item, Node };
use crate::numbers;
use crate::labels;
use crate::expr;

use crate::error::{IResult, ParseError};
use crate::locate::{ Span, AsSpan };

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
use nom::sequence::{ terminated,preceded, tuple, };
use nom::combinator::cut;

pub static LIST_SEP: & str = ",";

pub fn ws<'a,F,O,E>(mut inner : F) -> impl FnMut(Span<'a>) -> IResult<O>
where
  F: nom::Parser<Span<'a>, O,ParseError>
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
F: nom::Parser<Span<'a>, O,ParseError>
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
F: nom::Parser<Span<'a>, O,ParseError> + Copy {
    move |input: Span| {
        let sep = tuple((multispace0, tag(LIST_SEP), multispace0));
        separated_list1(sep, inner)(input)
    }
}

pub fn parse_assignment(input: Span) -> IResult< Node> {
    use labels::parse_label;
    let (rest, (label, _, _, _, arg)) = tuple((
            parse_label,
            multispace1,
            tag_no_case("equ"),
            multispace1,
            expr::parse_expr
            ))(input)?;

    let ret = Node::from_item(Item::Assignment)
        .with_children(vec![label, arg])
        .with_pos(input, rest);

    Ok((rest, ret))
}



pub fn wrapped<'a, O1, O, O3, E, F, INNER, S>(
    _first: F,
    _inner: INNER,
    _second: S,
    ) -> impl FnMut(Span) -> IResult<O>
where
F: nom::Parser<Span<'a>, O1, ParseError> ,
INNER: nom::Parser<Span<'a>, O, ParseError> ,
S: nom::Parser<Span<'a>, O3, ParseError> ,
{
    move |_input: Span| {
        panic!()
    }
}



// pub fn sep_list1<'a, F, O, E: ParseError<Span<'a>>>(
//     inner: F
//     ) -> impl FnMut(Span<'a>) -> IResult<Vec<O>>
// where
// F: nom::Parser<Span<'a>, O,E>  {
//     move |input: Span<'a>| {
//         let sep = tuple((multispace0, tag(LIST_SEP), multispace0));
//         separated_list1(
//             sep,
//             inner)(input)
//     }
// }


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
pub fn parse_number(input: Span) -> IResult< Node> {
    let (rest, num) = numbers::number_token(input)?;
    let ret = Node::from_number(num).with_pos(input,rest);
    Ok((rest, ret))
}

////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {

    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn test_parse_str() {
        let res = parse_escaped_str("\"kjskjbb\"".into());
        println!("res : {:?}", res);
        assert!(res.is_ok())
    }
    #[test]
    fn test_assignment() {
        let input = "hello equ $1000";
        let res = parse_assignment(input.as_span());

        let op_start = 0;
        let op_end = input.len();
        let equ_pos = op_start + 6;
        let num_start = equ_pos + 4;

        assert!(res.is_ok());

        let (_rest, matched) = res.unwrap();

        let args : Vec<_> = vec![
            Node::from_item(Item::Label("hello".to_string())).with_upos(op_start, op_start + 5),
            Node::from_number(4096).with_upos(num_start, op_end)
        ];

        let desired = Node::from_item(Item::Assignment)
            .with_children(args)
            .with_upos(op_start, op_end);

        assert_eq!(desired, matched);
    }
}


