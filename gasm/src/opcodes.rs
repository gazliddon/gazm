use super::item::Item;
use super::util;
use super::numbers;
use romloader::{Dbase, Instruction};

use nom::branch::alt;
use nom::IResult;
use std::collections::{HashMap, HashSet};
use std::num::IntErrorKind;
use nom::error::ErrorKind::NoneOf;
use nom::error::{Error, ParseError};

use nom::character::complete::{
    alpha1, alphanumeric1, anychar, char as nom_char, line_ending, multispace0, multispace1,
    not_line_ending, one_of, satisfy, space1,
};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};


////////////////////////////////////////////////////////////////////////////////
// opcode parsing
pub struct OpCodes {
    name_to_ins: HashMap<String, Vec<Instruction>>,
    dbase: Dbase,
}

// Some opcodes have an aliase delimited by underscores
fn split_opcodes(_input: &str) -> Option<(&str, &str)> {
    let split: Vec<&str> = _input.split("_").collect();

    if split.len() != 2 {
        None
    } else {
        Some((split[0], split[1]))
    }
}

impl OpCodes {
    pub fn new() -> Self {
        let dbase = Dbase::new();

        let mut name_to_ins: HashMap<String, Vec<Instruction>> = HashMap::new();

        let mut add = |name: &str, i: &Instruction| {
            let i = i.clone();
            let name = String::from(name).to_ascii_lowercase();
            if let Some(rec) = name_to_ins.get_mut(&name) {
                rec.push(i);
            } else {
                name_to_ins.insert(name.to_string(), vec![i]);
            }
        };

        for i in dbase.all_instructions() {
            if let Some((a, b)) = split_opcodes(&i.action) {
                add(&a, i);
                add(&b, i);
            } else {
                add(&i.action, i);
            }
        }
        Self { name_to_ins, dbase }
    }

    pub fn is_opcode(&self, input: &str) -> bool {
        self.get_opcode(&input).is_some()
    }

    pub fn get_opcode(&self, input: &str) -> Option<&Vec<Instruction>> {
        let op = String::from(input).to_lowercase();
        self.name_to_ins.get(&op)
    }
    pub fn get_db(&self) -> &Dbase {
        &self.dbase
    }
}

////////////////////////////////////////////////////////////////////////////////
// opcode parsing
lazy_static::lazy_static! {
    static ref OPCODES_REC: OpCodes = OpCodes::new();
}

pub fn opcode_token(input: &str) -> IResult<&str, &str> {
    let (rest, matched) = alpha1(input)?;

    if OPCODES_REC.is_opcode(matched) {
        Ok((rest, matched))
    } else {
        Err(nom::Err::Error(Error::new(input, NoneOf)))
    }
}

fn parse_opcode_arg(input: &str) -> IResult<&str, Item> {
    let (rest, matched) = util::parse_not_sure(input)?;
    Ok((rest, matched))
}

fn opcode_with_arg(input: &str) -> IResult<&str, Item> {
    let (rest, (op, arg)) = separated_pair(opcode_token, multispace1, not_line_ending)(input)?;
    Ok((rest, Item::OpCodeWithArg(op, arg)))
}

fn opcode_no_arg(input: &str) -> IResult<&str, Item> {
    let (rest, text) = opcode_token(input)?;
    Ok((rest, Item::OpCode(text, None)))
}

pub fn parse_opcode(input: &str) -> IResult<&str, Item> {
    let (rest, item) = alt((opcode_with_arg, opcode_no_arg))(input)?;
    Ok((rest, item))
}
