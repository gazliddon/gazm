use std::collections::HashMap;

use crate::expr::parse_expr;
use crate::labels::{get_just_label, parse_just_label};
use crate::locate::{matched_span, Span};
use crate::util::{self, get_block, sep_list1, wrapped_chars, ws};

use nom::multi::separated_list0;
use nom::{
    branch::alt,
    bytes::complete::{is_not, take_until, is_a, tag},
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
    pub name: String,
    pub params: Vec<String>,
    pub body: String,
}

use regex::Regex;

impl MacroDef {
    pub fn new(name: String, params: Vec<String>, body: String) -> Self {
        Self { name, params, body }
    }
    fn mk_regex(&self) -> Vec<Regex> {
        let to_regex = |v: &String| {
            let start = r"\|\s*";
            let end = r"\s*\|";
            let re = format!("{}{}{}", start, v, end);
            regex::Regex::new(&re).unwrap()
        };
        self.params.iter().map(to_regex).collect()
    }

    pub fn expand(&self, args: Vec<&str>) -> String {
        if args.len() != self.params.len() {
            panic!("Wrong number of args")
        }

        let regex = self.mk_regex();

        let pairs = regex.iter().zip(args);

        let mut ret = self.body.clone();

        for (rex, sub) in pairs {
            ret = rex.replace_all(&ret, sub).to_string();
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

fn parse_raw_args(input: Span<'_>)-> IResult<Vec<Node>> {
    let sep = ws(tag(","));
    let arg = ws(recognize(many1(is_not(",)"))));
    let (rest,matched) = ws(wrapped_chars('(',separated_list0(sep, arg),')'))(input)?;

    let ret = matched.iter().map(|i|{
        let item = Item::RawText(i.to_string());
        Node::from_item_span(item, i.clone())
    });

    Ok(( rest,ret.collect() ))
}

pub fn parse_macro_call(input: Span<'_>) -> IResult<Node> {
    let rest = input;
    let (rest, (name, args)) = separated_pair(parse_just_label, multispace0, parse_raw_args)(rest)?;

    println!("Found macro invocation!");

    let matched_span = matched_span(input, rest);
    let ret =
        Node::from_item_span(Item::MacroCall(name.to_string()), matched_span).with_children(args);

    Ok((rest, ret))
}

pub fn expand_macros(
    tokens: &mut Node,
    sources: &mut romloader::sources::Sources,
) -> anyhow::Result<()> {
    use crate::item::Item;
    // get all macro defs into a hash
    let mut name_to_def = HashMap::new();
    let defs = tokens.iter().filter_map(|x| match &x.item {
        Item::MacroDef(mdef) => Some(mdef),
        _ => None,
    });
    for x in defs {
        name_to_def.insert(x.name.clone(), x);
    }

    let calls = tokens.iter().filter_map(|x| {
        match &x.item {
            Item::MacroCall(name) => Some(( name,&x.children )),
            _ => None,
        }
    });

    for (name, args) in calls {
        if let Some(def) = name_to_def.get(name) {

            let args : Vec<&str> = args.iter().filter_map(|x|{
                match &x.item {
                    Item::RawText(value) => Some(value.as_str()),
                    _ => panic!("Shouldn't happen")
                }
            }).collect();

            let expansion = def.expand(args);
            println!("Expansion: {}", expansion);

        } else {
            panic!("can't find macro def {}", name)
        }
    }

    Ok(())
}

#[allow(unused_imports)]
mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn test_expansion() {
        let body = "Hello my name is { arg1   } I am {arg2}";
        let params = vec!["arg1", "arg2"];
        let args = vec!["Gaz", "Ace"];
        let desired = "Hello my name is Gaz I am Ace";

        let name = "test";
        let params = params.iter().map(|x| x.to_string());
        let mac = MacroDef::new(name.to_string(), params.collect(), body.to_string());
        let res = mac.expand(args);
        println!("{}", res);

        assert_eq!(res, desired.to_string());
    }
}
