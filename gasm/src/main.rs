#![allow(unused_imports)]
#![allow(dead_code)]

mod expr;
mod comments;
mod item;
mod numbers;
mod commands;
mod util;
mod opcodes;
mod register;
mod labels;
mod fileloader;
mod node;

use labels::parse_label;

use commands::command_token;
use comments::strip_comments_and_ws;
use item::{Item, Node};

use nom::branch::alt;
use nom::bytes::complete::tag_no_case;

use nom::character::complete::{
    line_ending, multispace0, multispace1
};
use nom::character::{is_alphabetic, is_space};
use nom::combinator::{eof, opt, all_consuming};
use nom::sequence::{pair, tuple};
use nom::IResult;

use opcodes::{parse_opcode, opcode_token};
use std::collections::HashMap;
use std::fs;

use std::hash::Hash;
use std::path::{Path, PathBuf,};


pub fn get_offset(master: &str, text: &str) -> usize {
    text.as_ptr() as usize - master.as_ptr() as usize
}

struct DocContext<'a> {
    master: &'a str,
    ranges: Vec<std::ops::Range<usize>>,
    lines: Vec<&'a str>,
    tokens: Vec<Item>,
}

impl<'a> DocContext<'a> {
    pub fn token(&mut self, tok: Item) {
        self.tokens.push(tok)
    }

    pub fn new(master: &'a str) -> Self {

        let mut offsets: Vec<_> = master.lines().map(|l| get_offset(master, l)).collect();
        offsets.push(master.len());

        let mut it2 = offsets.iter();
        it2.next();

        let zip = offsets.iter().zip(it2);

        let ranges: Vec<_> = zip.map(|(s, e)| *s..*e).collect();

        Self {
            master,
            ranges,
            lines: master.lines().collect(),
            tokens: vec![],
        }
    }

    pub fn to_line_number(&self, text: &'a str) -> usize {
        let offset = get_offset(self.master, text);

        for (line, r) in self.ranges.iter().enumerate() {
            if r.contains(&offset) {
                return line;
            }
        }

        panic!("Should not happen {} {:?}", offset, text);
    }

    pub fn to_line(&self, text: &'a str) -> (usize, &'a str) {
        let line = self.to_line_number(text);
        (line, self.lines.get(line).unwrap())
    }
}

pub fn parse<'a>(source : &'a str) -> IResult<&'a str, Vec<Node>> {
    use commands::parse_command;

    let mut items : Vec<Node> = vec![];

    let mut push_some = |x : &Option<Node> | {
        if let Some(x) = x {
            items.push(x.clone())
        }
    };

    use util::ws;

    for line in source.lines() {

        let (input, comment) = strip_comments_and_ws(line)?;
        push_some(&comment);

        if input.is_empty() {
            continue;
        }

        if let Ok((_,equate )) = all_consuming(ws(parse_assignment))(input) {
            push_some(&Some(equate));
            continue;
        }

        if let Ok((_,label)) = all_consuming(ws(parse_label))(input) {
            push_some(&Some(label.into()));
            continue;
        }

        let body = alt(( ws( parse_opcode ),ws( parse_command ) ));

        let res = all_consuming( pair(opt(parse_label),body))(input);

        if let Ok((_, (label,body))) = res {
            push_some(&label);
            push_some(&Some(body));
        } else {
            println!("{:?}", res);
            println!("Input: {:?}", input);
        }
    }

    // filter out empty comments
    let items = items
        .into_iter()
        .filter(|n| !n.is_empty_comment())
        .collect();

    Ok(("", items))
}


use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Context {
#[clap(long)]
    verbose: bool,
    #[clap(short, long)]
    file : PathBuf,
    #[clap(short, long)]
    out: Option<String>
}

struct SourceFile {
    name : PathBuf,
    loaded_name: PathBuf,
    tokens: Vec<Item>,
    children: HashMap<PathBuf, SourceFile>
}
use std::env;
use std::io;

pub fn tokenize_file<P: AsRef<Path>>(fl : &fileloader::FileLoader, file_name : P) -> Result<Node, Box<dyn std::error::Error>> {
    let file_name = file_name.as_ref().to_path_buf();

    println!("abnout to tokenize {:?}", file_name.as_path());

    let (loaded_name,source) = fl.read_to_string(&file_name)?;

    let (_rest, mut matched) = parse(&source).unwrap();

    println!("tokenized {:?}", loaded_name);

    for tok in &mut matched {
        if let Item::Include(file) = tok.item() {
            let inc_source = tokenize_file(fl, file.clone())?;
            *tok = inc_source;
        }
    }

    let ret = Node::from_item(Item::File(loaded_name)).with_children(matched);

    Ok(ret)
}

fn tokenize( ctx : &Context ) -> Result<Node, Box<dyn std::error::Error>> {
    use fileloader::FileLoader;

    let file = ctx.file.clone();

    let mut paths = vec![];

    if let Some(dir) = file.parent() {
        paths.push(dir);
    }

    let fl = FileLoader::from_search_paths(&paths);
    tokenize_file(&fl, file)
}

fn truncate(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

fn dump_with_depth(depth: usize, node : &Node) {
    let prefix = " ".repeat(depth* 4);
    let x = format!("{:?}", node);
    println!("{}{}", prefix, truncate(&x,100));

    for i in node.iter() {
        dump_with_depth(depth+1, i);
    }
}

fn dump(item : &Node) {
    dump_with_depth(0, item)
}

fn main() {
    let ctx = Context::parse();
    let res = tokenize(&ctx);

    match res {
        Ok(n) => dump(&n),
        _ => ()
    }
}


pub fn parse_assignment(input: &str) -> IResult<&str, Node> {
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

////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {

#[allow(unused_imports)]
    use pretty_assertions::{assert_eq, assert_ne};

    struct Line<'a> {
        label: Option<&'a String>,
        opcode: Option<Item>,
    }

    use super::*;

    // fn line_parse(input: &str) -> IResult<&str, Item> {

    //     // get rid of preceding ws
    //     // let (_,rest) = strip_ws(input)?;
    //     let (rest, (_, matched, _)) = tuple((multispace0, parse_label, multispace0))(input)?;
    //     Ok((rest, matched))
    // }

    #[test]
    fn test_number() {
        let pnum = parse_number;
        let input = "0x1000";
        let desired = Item::Number(0x1000);
        let (_, matched) = pnum(input).unwrap();
        assert_eq!(matched, desired);
    }

    #[test]
    fn test_id_ok() {
        let junk = &"ksjakljksjakjsakjskaj ";
        let good_ids = &["ThisIsFine", "alphaNum019292", "_startsWithUscore"];

        let check_it = |id: &str, junk: &str| {
            let id = String::from(id);
            let str1 = format!("{} {}", id, junk);
            let res = line_parse(&str1);
            println!("res:  {:?}", res);
            let (rest, matched) = res.unwrap();

            println!("matched: {:?}", matched);
            println!("rest: {:?}", rest);

            assert_eq!(matched, Item::Label(id));
            assert_eq!(rest, junk);
        };

        for id in good_ids {
            check_it(id, junk);
        }
    }

    #[test]
    fn test_assignment() {
        let input = "hello equ $1000";
        let res = parse_assignment(input);
        assert!(res.is_ok());

        let (rest, matched) = res.unwrap();

        let args : Vec<_> = vec![
            Item::Label("hello".to_string()).into(),
            Node::from_item(Item::Expr).with_child(Item::Number(100).into())
        ];

        let desired = Node::from_item(Item::Expr).with_children(args);

        assert_eq!(desired, matched);
        assert_eq!(rest, "");
    }

    #[test]
    fn test_id_fail() {
        let junk = &"ksjakljksjakj s akjs kaj ";

        let bad_ids = &[
            "0canstartwithanumber",
            "manyillegal-chars-!;:",
            "has spaces in",
        ];

        for id in bad_ids {
            let str1 = format!("{} {}", id, junk);
            let res = line_parse(&str1);

            if res.is_ok() {
                let (_, matched) = res.unwrap();
                assert_ne!(matched, Item::Label(id.to_string()));
            }
        }
    }
}
