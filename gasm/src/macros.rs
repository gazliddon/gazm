use crate::expr::parse_expr;
use crate::labels::{get_just_label, parse_just_label};
use crate::locate::{matched_span, Span};
use crate::util::{self, get_block, sep_list1, wrapped_chars, ws};

use nom::{
    branch::alt,
    bytes::complete::{is_not, take_until},
    character::complete::{line_ending, multispace0, multispace1},
    combinator::{all_consuming, eof, not, opt, recognize},
    multi::{many0, many1},
    sequence::{pair, preceded, separated_pair, terminated},
    AsBytes,
};

use crate::error::{IResult, ParseError, UserError};

use crate::item::{Item, Node};

#[derive(Debug, PartialEq, Clone)]
pub struct MacroDef {
    name: String,
    params: Vec<String>,
    body: String,
}

impl MacroDef {
    pub fn expand(&self, args: &Vec<String>) -> String {
        use regex::Regex;

        let to_regex = |v: &String| {
            let start = r"\{\s*";
            let end = r"\s*\}";
            let re = format!("{}{}{}", start, v, end);
            regex::Regex::new(&re).unwrap()
        };

        if args.len() != self.params.len() {
            panic!("Wrong number of args")
        }

        let pairs = self.params.iter().map(to_regex).zip(args.clone());

        let mut ret = self.body.clone();

        for (rex, sub) in pairs {
            ret = rex.replace_all(&ret,sub).to_string();
        }

        ret
    }
}

////////////////////////////////////////////////////////////////////////////////
// Macros
pub fn get_macro_def(input: Span<'_>) -> IResult<(Span, Vec<Span>, Span)> {
    use nom::bytes::complete::tag;
    let rest = input;
    let (rest, (_, name)) = ws(separated_pair(tag("macro"), multispace1, get_just_label))(rest)?;
    let (rest, params) = wrapped_chars('(', sep_list1(get_just_label), ')')(rest)?;
    let (rest, body) = get_block(rest)?;
    Ok((rest, (name, params, body)))
}

pub fn parse_macro_definition(input: Span<'_>) -> IResult<Node> {
    let (rest, (name, params, body)) = get_macro_def(input)?;

    let matched_span = matched_span(input, rest);

    let def = MacroDef {
        name: name.to_string(),
        body: body.to_string(),
        params: params.iter().map(|x| x.to_string()).collect(),
    };

    let ret = Node::from_item_span(Item::MacroDef(def), matched_span);

    Ok((rest, ret))
}

pub fn parse_macro_call(input: Span<'_>) -> IResult<Node> {
    let args = wrapped_chars('(', sep_list1(parse_expr), ')');

    let rest = input;
    let (rest, (name, args)) = separated_pair(parse_just_label, multispace0, args)(rest)?;

    println!("Found macro invocation!");

    let matched_span = matched_span(input, rest);
    let ret =
        Node::from_item_span(Item::MacroCall(name.to_string()), matched_span).with_children(args);

    Ok((rest, ret))
}
