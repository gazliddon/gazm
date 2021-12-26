use crate::item::{ Item, Node };
use crate::numbers;
use crate::labels;
use crate::expr;

use nom::IResult;
use nom::error::ParseError;
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

pub fn ws<'a, F, O, E,T>( mut inner: F,) -> impl FnMut(T) -> IResult<T, O, E>
where
  F: nom::Parser<T, O, E> + 'a,
  T: nom::InputTakeAtPosition,
  <T as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
  E: ParseError<T>,
{
  move |input: T| {
    let (input, _) = multispace0(input)?;
    let (input, out) = inner.parse(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input,out))
  }
}

// pub fn denode<'a, F, E>( mut inner: F,) -> impl FnMut(&'a str) -> IResult<&'a str, Item, E>
// where
//   F: nom::Parser<&'a str, Node, E> + 'a,
//   E: ParseError<& 'a str>,
// {
//   move |input: &'a str| {
//     let (input, out) = inner.parse(input)?;
//     Ok((input,out.into()))
//   }
// }

pub fn parse_assignment(input: &str) -> IResult<&str, Node> {
    use labels::parse_label;
    let (rest, (label, _, _, _, arg)) = tuple((
            parse_label,
            multispace1,
            tag_no_case("equ"),
            multispace1,
            expr::parse_expr
            ))(input)?;

    let ret = Node::from_item(Item::Assignment).with_children(vec![label, arg]);

    Ok((rest, ret))
}

pub fn wrapped<I, O1, OUT, O3, E, F, INNER, S>(

  mut first: F,
  mut inner: INNER,
  mut second: S,
) -> impl FnMut(I) -> IResult<I, OUT, E>
where
  E: ParseError<I>,
  F: nom::Parser<I, O1, E> ,
  INNER: nom::Parser<I, OUT, E> ,
  S: nom::Parser<I, O3, E> ,
  I: nom::InputTakeAtPosition,
  <I as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
{
  move |input: I| {
    let (input, _) = first.parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, out) = inner.parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = second.parse(input)?;
    Ok((input,out))
  }
}

pub fn wrapped_chars<I,  O2,  E: ParseError<I> , G>(
    open : char, yes_please: G, close: char)
-> impl FnMut(I) -> IResult<I, O2, E>
where
  I: nom::InputTakeAtPosition + nom::InputIter + nom::Slice<std::ops::RangeFrom<usize>>,
  <I as nom::InputIter>::Item: nom::AsChar + Clone,
  <I as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
G: nom::Parser<I, O2, E>  {
    wrapped(nom_char(open),yes_please, nom_char(close))
}

pub fn sep_list1<'a, F, O, E: ParseError<&'a str>>(
    inner: F
    ) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<O>, E>
where
F: nom::Parser<&'a str, O,E>  + Copy {
    move |input: &'a str| {
        let sep = tuple((multispace0, tag(LIST_SEP), multispace0));
        separated_list1(sep, inner)(input)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Escaped string

pub fn match_str(input: &str) -> IResult<&str, &str> {
    let term = "\"n\\";
    let body = take_while(move |c| !term.contains(c));
    escaped(body, '\\', one_of(term))(input)
}

pub fn match_escaped_str(input: &str) -> IResult<&str, &str> {
    preceded(nom_char('\"'), cut(terminated(match_str, nom_char('\"'))))(input)
}

////////////////////////////////////////////////////////////////////////////////
// Number

pub fn parse_escaped_str(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = match_escaped_str(input)?;
    Ok((rest, Item::QuotedString(matched.to_string())))
}
pub fn parse_number(input: &str) -> IResult<&str, Node> {
    let (rest, (num, _text)) = numbers::number_token(input)?;
    Ok((rest, Node::from_number(num)))
}


////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {

    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn test_parse_str() {
        let res = parse_escaped_str("\"kjskjbb\"");
        println!("res : {:?}", res);
        assert!(res.is_ok())
    }
    #[test]
    fn test_assignment() {
        let input = "hello equ $1000";
        let res = parse_assignment(input);
        assert!(res.is_ok());

        let (rest, matched) = res.unwrap();

        let args : Vec<_> = vec![
            Node::from_item(Item::Label("hello".to_string())),
            Node::from_number(4096)
        ];

        let desired = Node::from_item(Item::Assignment).with_children(args);

        assert_eq!(desired, matched);
        assert_eq!(rest, "");
    }

}


