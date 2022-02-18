// Load an AS6809 map file and create a binary

use std::fs;
use std::path::{Path, PathBuf};

pub struct MapFile {
    text: String,
    pub data : Vec<Record>,
}

pub fn read_to_string<P: AsRef<Path>>(path: P) -> anyhow::Result<String> {
    // let str_path = path.as_ref().to_string_lossy();
    let ret = fs::read_to_string(path)?;
    Ok(ret)
}

use nom::AsBytes;
use regex::Regex;
use romloader::sources::SymbolTable;

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub addr : u16,
    pub data : Vec<u8>,
}

pub fn add_reference_syms<P: AsRef<Path>>(file_name : P, syms : &mut SymbolTable) -> anyhow::Result<()> {
    let text = read_to_string(file_name)?;

    let equ_rex = Regex::new(r"^(?P<label>\S+)\s*equ\s*(?P<data>[0-9]+)").unwrap();

    for line in text.lines() {
        if let Some(matches) = equ_rex.captures(line) {
            let data = i64::from_str_radix(&matches["data"], 10).unwrap();
            let label = &matches["label"];
            syms.add_reference_symbol(label, data);
        }
    }

    Ok(())
}

fn make_records(x: &String) -> Vec<Record>{
    let data_rex = Regex::new(r"^(?P<addr>[0-9A-F]{4})\s+(?P<data>[0-9A-F]{2}+)").unwrap();

    let mut ret = vec![];

    for line in x.lines() {
        if let Some(matches) = data_rex.captures(line) {
            let addr = u16::from_str_radix(&matches["addr"], 16).unwrap();
            let data = &matches["data"];

            let data: Vec<u8> = (0..data.len())
                .step_by(2)
                .map(|x| u8::from_str_radix(&data[x..x + 2], 16).unwrap())
                .collect();
            ret.push(Record{addr,data})
        }
    }
        ret
}

impl MapFile {
    pub fn new<P: AsRef<Path>>(file_name: P) -> anyhow::Result<Self> {
        let text = read_to_string(file_name)?;
        let data = make_records(&text);
        let ret = Self { text, data };
        Ok(ret)
    }
}
