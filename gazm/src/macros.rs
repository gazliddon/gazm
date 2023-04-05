use std::collections::HashMap;

use crate::labels::get_just_label;
use crate::locate::{matched_span, span_to_pos, Span};
use crate::parse::util::{self, get_block, sep_list0, wrapped_chars, ws};

use nom::multi::separated_list0;
use nom::{
    bytes::complete::{is_not, tag},
    character::complete::{multispace0, multispace1},
    combinator::recognize,
    multi::many1,
    sequence::separated_pair,
};

use emu::utils::sources::{Position, SourceFiles};

use crate::error::{IResult, UserError};
use crate::item::Node;

#[derive(Debug, PartialEq, Clone)]
pub struct MacroDef {
    pub name: String,
    pub params: Vec<String>,
    pub pos: Position,
    pub nodes: Vec<Node>,
}

use regex::Regex;

impl MacroDef {
    pub fn new(name: String, params: Vec<String>, pos: Position, nodes: Vec<Node>) -> Self {
        Self {
            name,
            params,
            pos,
            nodes,
        }
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

    /// Expands this macro
    /// args = a vec of positions of the arguments
    /// returns a string of the expanded macro and the position of the original macro text
    pub fn expand(&self, sources: &SourceFiles, args: Vec<Position>) -> (Position, String) {
        if args.len() != self.params.len() {
            panic!("Wrong number of args")
        }

        let args = args
            .iter()
            .map(|pos| sources.get_source_info(pos).unwrap().fragment);

        let x = sources.get_source_info(&self.pos).unwrap();

        let pairs = self.params.iter().zip(args);

        let mut ret = x.fragment.to_string();

        for (param, arg) in pairs {
            let param = format!("|{param}|");
            ret = ret.replace(&param, arg);
        }
        (self.pos.clone(), ret)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Macros
/// Gets the strings for a macro definition
/// returns (name, array of args, macro body)
pub fn get_macro_def(input: Span<'_>) -> IResult<(Span, Vec<Span>, Span)> {
    let rest = input;
    let (rest, (_, name)) = ws(separated_pair(tag("macro"), multispace1, get_just_label))(rest)?;
    let (rest, params) = wrapped_chars('(', sep_list0(get_just_label), ')')(rest)?;
    let (rest, body) = get_block(rest)?;
    Ok((rest, (name, params, body)))
}

pub fn get_scope_block(input: Span<'_>) -> IResult<(Span, Span)> {
    let rest = input;
    let (rest, (_, name)) = ws(separated_pair(tag("scope2"), multispace1, get_just_label))(rest)?;
    let (rest, body) = get_block(rest)?;
    Ok((rest, (name, body)))
}

fn parse_raw_args(input: Span<'_>) -> IResult<Vec<Span<'_>>> {
    let sep = ws(tag(","));
    let arg = ws(recognize(many1(is_not(",)"))));
    let (rest, matched) = ws(wrapped_chars('(', separated_list0(sep, arg), ')'))(input)?;

    Ok((rest, matched))
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacroCall {
    pub name: Position,
    pub args: Vec<Position>,
}

pub fn parse_macro_call(input: Span) -> IResult<Node> {
    use crate::expr::parse_expr;
    use crate::item::Item;
    use util::sep_list1;
    let sep = ws(tag(","));

    let rest = input;
    let (rest, name) = get_just_label(rest)?;
    let (rest, args) = ws(wrapped_chars(
        '(',
        ws(separated_list0(sep, parse_expr)),
        ')',
    ))(rest)?;

    let matched_span = span_to_pos(matched_span(input, rest));
    let ret = Node::new_with_children(Item::MacroCall(name.to_string()), args, matched_span);

    Ok((rest, ret))
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Macros {
    macro_defs: HashMap<String, MacroDef>,
}

impl Macros {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_def(&mut self, def: MacroDef) {
        self.macro_defs.insert(def.name.clone(), def);
    }

    /// Expands a macro and returns a position of the macro body text
    /// an expanded version of the macro ready to tokenize
    /// returns an the position of the original macro definition and the expanded macro
    pub fn expand_macro(
        &self,
        sources: &SourceFiles,
        macro_call: MacroCall,
    ) -> Result<(Position, String), UserError> {
        let si = sources.get_source_info(&macro_call.name).unwrap();
        let name = si.fragment;

        let def = self.macro_defs.get(name).ok_or_else(|| {
            let x = format!("Couldn't find a macro definition for {name}");
            UserError::from_text(x, &sources.get_source_info(&macro_call.name).unwrap(), true)
        })?;

        Ok(def.expand(sources, macro_call.args))
    }
}

#[allow(unused_imports)]
mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    #[test]
    fn test_scopes() {
        let body = "scope2 background {a} aka kj akj a";
        let sp = Span::new_extra(body, emu::utils::sources::AsmSource::FromStr);

        if let Ok((rest, (name, body))) = get_scope_block(sp) {
            assert_eq!(&name.to_string(), "background");
            assert_eq!(&body.to_string(), "a");
            assert_eq!(&rest.to_string(), "aka kj akj a");
        } else {
            assert!(false)
        }
    }

    #[test]
    fn test_expansion() {
        // let body = "Hello my name is { arg1   } I am {arg2}";
        // let params = vec!["arg1", "arg2"];
        // let args = vec!["Gaz", "Ace"];
        // let desired = "Hello my name is Gaz I am Ace";

        // let name = "test";
        // let params = params.iter().map(|x| x.to_string());
        // // let mac = MacroDef::new(name.to_string(), params.collect(), body.to_string());
        // let res = mac.expand(args);
        // println!("{}", res);

        // assert_eq!(res, desired.to_string());
    }
}
