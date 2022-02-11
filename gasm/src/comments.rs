use crate::item::{ Item, Node };
use nom::character::complete::{anychar, multispace0, not_line_ending};
use nom::multi::{many0, many1, };

use nom::bytes::complete::take_until ;

use nom::combinator::recognize;
use nom::branch::alt;

use nom::sequence::{preceded, tuple, pair};
use nom::bytes::complete::tag;

use crate::error::IResult;
use crate::locate::Span;

static COMMENT: & str = ";";

fn get_comment(input: Span) -> IResult<Span> {
    let comments = alt((tag(";"), tag("//")));
    recognize(pair(comments, not_line_ending))(input)
}
fn get_star_comment(input: Span) -> IResult<Span> {
    let comments = tag("*");
    recognize(pair(comments, not_line_ending))(input)
}

fn parse_comment(input: Span) -> IResult<Node> {
    use Item::*;

    let (rest, matched) = get_comment(input)?;
    let ret = Node::from_item_span(Comment(matched.to_string()),input);
    Ok((rest, ret))
}

pub fn strip_star_comment(input: Span) -> IResult<Node> {
    use crate::util::ws;
    let (rest, matched) = ws(get_star_comment)(input)?;
    let ret = Node::from_item_span(Item::Comment(matched.to_string()),input);
    Ok((rest, ret))
}

// Strips comment if there
pub fn strip_comments(input: Span) -> IResult<Option<Node>> {
    let not_comment_1 = take_until(COMMENT);
    let not_comment_2 = take_until("//");
    let not_comment = alt((not_comment_2, not_comment_1));

    let res = tuple((not_comment, parse_comment))(input);

    if let Ok((_rest, (pre_comment, node))) = res {
        Ok((pre_comment, Some(node)))
    } else {
        Ok((input, None))
    }
}


////////////////////////////////////////////////////////////////////////////////
// tests
#[allow(unused_imports)]
mod test {
    use romloader::sources::Position;

    use super::*;
    use pretty_assertions::assert_eq;

//     fn test_comments(comment: &str, pre_amble: &str) {
//         let line = format!("{}{}", pre_amble, comment);
//         println!("comment:   {:?}", comment);
//         println!("pre_amble: {:?}", pre_amble);
//         println!("line:      {:?}", line.as_str());

//         let start = pre_amble.len();
//         let end = start + comment.len();
//         let des_ctx = Position::new(start,end);

//         let line = mk_span("", &line);
//         let (rest, com) = strip_comments(line).unwrap();
//         assert!(com.is_some());
//         println!("{:?}", com);
//         let des = Node::from_item(Item::Comment(comment.to_string()), line);
//         let rest : &str = rest.as_ref();
//         assert_eq!(rest, pre_amble);
//         assert_eq!(des, com.unwrap());
//     }

//     #[test]
//     fn test_strip_comments_3() {
//         let comment = &";lda kskjkja".to_string();
//         let pre_amble = &"   ";
//         test_comments(comment, pre_amble);
//     }

//     #[test]
//     fn test_strip_comments_2() {
//         let comment = &"; lda kskjkja".to_string();
//         let pre_amble = &" skljk  kds lk ";
//         test_comments(comment, pre_amble);

//         let comment = &";;;; lda kskjkja".to_string();
//         let pre_amble = &"skljk  kds lk ";
//         test_comments(comment, pre_amble);
//     }

//     #[test]
//     fn test_strip_comments() {
//         let comment = &";;; kljlkaslksa";
//         let pre_amble = &"    ";
//         test_comments(comment, pre_amble);

//         let comment = &";";
//         let pre_amble = &"    ";
//         test_comments(comment, pre_amble);
//     }
}
