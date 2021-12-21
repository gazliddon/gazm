use crate::item::Item;
use nom::character::complete::{anychar, multispace0, not_line_ending};
use nom::multi::{many0, many1, };
use nom::IResult;

use nom::bytes::complete::take_until ;

use nom::combinator::recognize;

use nom::sequence::{preceded, tuple};
use nom::bytes::complete::tag;

pub static COMMENT: & str = ";";

pub fn parse_comment(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = preceded(many1(tag(COMMENT)), not_line_ending)(input)?;
    Ok((rest, Item::Comment(matched.to_string())))
}

// Strips comments and preceding white space
pub fn strip_comments(input: &str) -> IResult<&str, Option<Item>> {
    let not_comment = take_until(COMMENT);

    let res = tuple((not_comment, parse_comment))(input);

    if let Ok((_rest, (line, comment))) = res {
        Ok((line, Some(comment)))
    } else {
        Ok((input, None))
    }
}

pub fn parse_any_thing(input: &str) -> IResult<&str, &str> {
    recognize(many0(anychar))(input)
}

pub fn strip_comments_and_ws(input: &str) -> IResult<&str,Option<Item>> {
    let (rest, comment) = strip_comments(input)?;
    let (_, text_matched) = preceded(multispace0, parse_any_thing)(rest).unwrap();
    Ok((text_matched,comment))
}

////////////////////////////////////////////////////////////////////////////////
// tests
#[allow(unused_imports)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn comments() {
        let com_text = " sdkjkdsjkdsj   ".to_string();
        let input = format!(";{}\n", com_text);

        println!("Input: {}", input);
        let (rest, matched) = parse_comment(&input).unwrap();
        assert_eq!(matched, Item::Comment(com_text));
        assert_eq!(rest, "\n");
    }

    fn strip_single_comment<'a, F: 'a>(line: &'a str, f: F) -> IResult<&'a str, Option<Item>>
    where
        F: Fn(&'a str) -> IResult<&'a str, Option<Item>>,
    {
        let (rest, com) = f(line)?;
        println!("\nline:    {:?}", line);
        println!("rest:    {:?}", rest);
        println!("comment: {:?}", com);
        Ok((rest, com))
    }

    #[test]
    fn test_strip_comments_3() {
        let comment = "lda kskjkja".to_string();
        let spaces = "   ";
        let pre_amble = "skljk  kds lk ";
        let line = &format!("{}{};{}", spaces, pre_amble, comment);
        let (rest, com) = strip_single_comment(line, strip_comments_and_ws).unwrap();
        assert_eq!(rest, pre_amble);
        assert_eq!(com, Some(Item::Comment(comment)));
    }

    #[test]
    fn test_strip_comments_2() {
        let comment = "lda kskjkja".to_string();
        let pre_amble = " skljk  kds lk ";
        let line = &format!("{};{}", pre_amble, comment);
        let (rest, com) = strip_single_comment(line, strip_comments).unwrap();
        assert_eq!(rest, pre_amble);
        assert_eq!(com, Some(Item::Comment(comment)));

        let comment = "lda kskjkja".to_string();
        let pre_amble = "skljk  kds lk ";
        let line = &format!("{};{}", pre_amble, comment);
        let (rest, com) = strip_single_comment(line, strip_comments_and_ws).unwrap();
        assert_eq!(rest, pre_amble);
        assert_eq!(com, Some(Item::Comment(comment)));
    }

    #[test]
    fn test_strip_comments() {
        let comment = "kljlkaslksa".to_string();
        let pre_amble = "    ";
        let line = &format!("{};{}", pre_amble, comment);
        let (rest, com) = strip_single_comment(line, strip_comments).unwrap();
        assert_eq!(rest, pre_amble);
        assert_eq!(com, Some(Item::Comment(comment)));

        let comment = "".to_string();
        let pre_amble = "    ";
        let line = &format!("{};{}", pre_amble, comment);
        let (rest, com) = strip_single_comment(line, strip_comments).unwrap();
        assert_eq!(rest, pre_amble);
        assert_eq!(com, Some(Item::Comment(comment)));
    }
}
