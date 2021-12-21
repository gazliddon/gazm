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

use commands::command_token;
use comments::{strip_comments, strip_comments_and_ws};
use item::{Item, TextItem};

use nom::branch::alt;
use nom::bytes::complete::{
    escaped, is_a, tag, tag_no_case, take_until, take_until1, take_while, take_while1,
};
use nom::character::complete::{
    alpha1, alphanumeric1, anychar, char as nom_char, line_ending, multispace0, multispace1,
    not_line_ending, one_of, satisfy, space1,
};
use nom::character::{is_alphabetic, is_space};
use nom::combinator::{cut, eof, map_res, opt, recognize, value, not, all_consuming};
use nom::error::{ContextError, Error, ParseError};
use nom::multi::{many0, many0_count, many1, separated_list0};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;

use lazy_static::lazy_static;
use opcodes::{parse_opcode, opcode_token};
use std::collections::HashSet;
use std::fs;

use crate::item::is_empty_comment;

use util::{parse_arg, parse_label, parse_arg_list};


pub fn get_offset(master: &str, text: &str) -> usize {
    text.as_ptr() as usize - master.as_ptr() as usize
}

struct DocContext<'a> {
    master: &'a str,
    ranges: Vec<std::ops::Range<usize>>,
    lines: Vec<&'a str>,
    tokens: Vec<Item<'a>>,
}

impl<'a> DocContext<'a> {
    pub fn token(&mut self, tok: Item<'a>) {
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

pub fn parse<'a>(lines : &'a [ &'a str ]) -> IResult<&'a str, Vec<Item<'a>>> {

    use commands::parse_command;

    let mut items : Vec<Item<'a>> = vec![];

    let mut push_some = |x : &Option<Item<'a>> | {
        if let Some(x) = x {
            items.push(x.clone())
        }
    };
    use util::ws;

    for line in lines {
        let (input, comment) = strip_comments_and_ws(line)?;

        push_some(&comment);

        if input.is_empty() {
            continue;
        }

        // Just a label
        if let Ok((_,label)) = all_consuming(ws(parse_label))(input) {
            push_some(&Some(label));
            continue;
        }

        // Assignment
        if let Ok(( _,equate )) = all_consuming(ws(parse_equate))(input) {
            push_some(&Some(equate));
            continue;
        }

        let body =alt(( ws( parse_opcode ),ws( parse_command ) ));

        let res = all_consuming(pair(opt(parse_label),body))(input);

        if let Ok((_, (label,body)))= res {
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
        .filter(|c| !is_empty_comment(c))
        .collect();

    Ok(("", items))
}

impl<'a> DocContext<'a> {
    pub fn push_some(&mut self, item : &Option<Item<'a>>) {
        if let Some(item) = item {
            self.tokens.push(item.clone())
        }
    }

    pub fn parse(&'a mut self) -> IResult<&'a str,&Vec<Item<'a>>> {

        let (rest, matched) = parse(&self.lines)?;
       self.tokens = matched.clone();
        Ok((rest, &self.tokens))
    }

}

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Context {
#[clap(long)]
   verbose: bool,
#[clap(short, long)]
    file : String,
#[clap(short, long)]
    out: Option<String>
}

fn assemble( ctx : &Context ) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(&ctx.file)?;

    let mut dc = DocContext::new(&source);
    let (_rest, _matched) = dc.parse().unwrap();

    for t in _matched {
        println!("{:?}", t)
    }

    Ok(())
}


fn main() {
    let ctx = Context::parse();
    let res = assemble(&ctx);

    match res {
        Ok(()) => {println!("Compiled!")},
        Err(e) => {println!("Error: {} {}", ctx.file, e)}
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
mod test {
    use pretty_assertions::{assert_eq, assert_ne};

    struct Line<'a> {
        label: Option<&'a String>,
        opcode: Option<Item<'a>>,
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

    // #[test]
    fn test_arg_list() {
        let txt = "1020,hello,0xffff,!!!";
        let (_rest, matched) = parse_arg_list(txt).unwrap();

        let desired = vec![
            Item::Number(1020),
            Item::Label(String::from("hello")),
            Item::Number(0xffff),
            Item::NotSure("!!!"),
        ];

        let desired = Item::ArgList(desired);

        assert_eq!(matched, desired);
    }

    // #[test]
    // fn test_op_code_2() {
    //     let check_op = |op: &str, arg: &str| {
    //         let input = format!("{} {}\n", op, arg);
    //         let desired = Item::OpCodeWithArg(&op, arg);
    //         let (rest, matched) = parse_opcode(&input).unwrap();
    //         assert_eq!(matched, desired);
    //         assert_eq!(rest, "\n");
    //     };

    //     {
    //         check_op("sta", "kljsadlkjl");
    //         check_op("StA", "kljsadlkjl");
    //         check_op("STA", "kljsadlkjl");
    //         check_op("STA", "aaakljsadlkjl");
    //     }

    //     let check_no_arg = |op: &str| {
    //         let desired = Item::OpCode(&op, None);
    //         let res = parse_opcode(op);
    //         println!("res is: {:?}", res);
    //         let (rest, matched) = res.unwrap();
    //         assert_eq!(matched, desired);
    //         assert_eq!(rest, "");
    //     };

    //     {
    //         check_no_arg("lda");
    //         check_no_arg("STA");
    //         check_no_arg("comb");
    //         check_no_arg("sEx");
    //     }

    //     {
    //         let res = parse_opcode("STAkjaskjskaa ");
    //         println!("{:?}", res);
    //         assert!(res.is_err());
    //     }
    // }

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
