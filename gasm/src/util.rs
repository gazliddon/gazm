use crate::item::Item;
use nom::IResult;
use std::collections::HashSet;
use nom::error::{Error, ParseError};

use nom::character::complete::{
    alpha1, alphanumeric1, anychar, char as nom_char, line_ending, multispace0, multispace1,
    not_line_ending, one_of, satisfy, space1,
};
use nom::sequence::{ terminated,preceded };
use nom::combinator::cut;
use nom::bytes::complete::{ escaped, take_while };

pub fn get_token<'a>(input: &'a str, hs: &HashSet<&'static str>) -> IResult<&'a str, &'a str> {
    use nom::error::ErrorKind::NoneOf;
    let (rest, matched) = alpha1(input)?;
    let opcode = String::from(matched).to_lowercase();

    if hs.contains(&opcode.as_str()) {
        Ok((rest, matched))
    } else {
        Err(nom::Err::Error(Error::new(input, NoneOf)))
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

pub fn parse_escaped_str(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = match_escaped_str(input)?;
    Ok((rest, Item::String(matched)))
}

mod test {
    use super::*;

    #[test]
    fn test_parse_str() {
        let res = parse_escaped_str("\"kjskjbb\"");
        println!("res : {:?}", res);
        assert!(res.is_ok())
    }

}


