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

use labels::parse_label;

use commands::command_token;
use comments::strip_comments_and_ws;
use item::{Item, TextItem, Command};

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

struct Ctx
{

}

impl Ctx {

}



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

    pub fn to_text_item(&self, text: &'a str) -> TextItem<'a> {
        let offset = text.as_ptr() as usize - self.master.as_ptr() as usize;
        TextItem { text, offset }
    }
}

pub fn parse<'a>(source : &'a str) -> IResult<&'a str, Vec<Item>> {

    use commands::parse_command;

    let mut items : Vec<Item> = vec![];

    let mut push_some = |x : &Option<Item> | {
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

        if let Ok((_,equate )) = all_consuming(ws(parse_equate))(input) {
            push_some(&Some(equate));
            continue;
        }

        if let Ok((_,label)) = all_consuming(ws(parse_label))(input) {
            push_some(&Some(label));
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
        .filter(|c| !c.is_empty_comment())
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
    tokens: Vec<Item>,
    children: HashMap<PathBuf, SourceFile>
}

impl SourceFile {
    pub fn from_file<P: AsRef<Path>>(fl : &fileloader::FileLoader, file_name : P) -> Result<Self, Box<dyn std::error::Error>> {
        let file_name = file_name.as_ref().to_path_buf();

        let mut children = HashMap::new();

        println!("Compiling {}", file_name.to_str().unwrap());

        let source = fl.read_to_string(&file_name)?;

        let (_rest, matched) = parse(&source).unwrap();

        for tok in &matched {
            if let Item::Command(Command::Include(file)) = tok {
                let inc_source = Self::from_file(fl, file)?;
                children.insert(file.clone(), inc_source);
            }
        }

        let ret = SourceFile {
            name : file_name,
            tokens: matched,
            children 
        };

        Ok(ret)
    }
}

fn assemble( ctx : &Context ) -> Result<SourceFile, Box<dyn std::error::Error>> {
    use fileloader::FileLoader;

    let file = ctx.file.clone();

    let mut paths = vec![];

    if let Some(dir) = file.parent() {
        paths.push(dir);
        println!("Dir is {:?}", dir);
    }

    let fl = FileLoader::from_search_paths(&paths);
    SourceFile::from_file(&fl, file)
}

fn main() {
    let ctx = Context::parse();
    let res = assemble(&ctx);

    match res {
        Ok(_) => {println!("Compiled!")},
        Err(e) => {println!("Error: {:?} {}", ctx.file, e)}
    }
}


pub fn parse_equate(input: &str) -> IResult<&str, Item> {
    let (rest, (label, _, _, _, arg)) = tuple((
            parse_label,
            multispace1,
            tag_no_case("equ"),
            multispace1,
            expr::parse_expr
            ))(input)?;
    Ok((rest, Item::Assignment(Box::new(label), Box::new(arg))))

}

pub fn parse_eof(input: &str) -> IResult<&str, Item> {
    let (rest, _) = eof(input)?;
    Ok((rest, Item::Eof))
}

pub fn parse_operand(_input: &str) -> IResult<&str, &str> {
    let _special = "[],+#";
    todo!()
}

////////////////////////////////////////////////////////////////////////////////
// Number



////////////////////////////////////////////////////////////////////////////////
// Misc

pub fn line_ending_or_eof(input: &str) -> IResult<&str, &str> {
    alt((eof, line_ending))(input)
}

fn is_char_space(chr: char) -> bool {
    is_space(chr as u8)
}
fn is_char_alphabetic(chr: char) -> bool {
    is_alphabetic(chr as u8)
}

fn is_char_end_line(chr: char) -> bool {
    chr == '\n' || chr == '\r'
}

////////////////////////////////////////////////////////////////////////////////
// Args


////////////////////////////////////////////////////////////////////////////////
// Tests
#[allow(unused_imports)]
mod test {
    use pretty_assertions::{assert_eq, assert_ne};

    struct Line<'a> {
        label: Option<&'a String>,
        opcode: Option<Item>,
    }

    use super::*;

    fn line_parse(input: &str) -> IResult<&str, Item> {
        // get rid of preceding ws
        // let (_,rest) = strip_ws(input)?;
        let (rest, (_, matched, _)) = tuple((multispace0, parse_label, multispace0))(input)?;
        Ok((rest, matched))
    }

    #[test]
    fn test_number() {
        let input = "0x1000";
        let desired = Item::Number(0x1000);
        let (_, matched) = util::parse_number(input).unwrap();
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
        let res = parse_equate(input);
        assert!(res.is_ok());

        let (rest, matched) = res.unwrap();

        let label = Box::new(Item::Label("hello".to_string()));
        let arg = Box::new(Item::Expr(vec![Item::Number(4096)]));
        let desired = Item::Assignment(label, arg);

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
