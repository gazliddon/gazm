use regex::Regex;

use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;

struct Opts {
    pub file : String,
}

struct Chunk {
    pub addr : u16,
    pub data : Vec<u8>,
    pub source_file : String,
    pub line : usize
}

use std::fmt;

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            writeln!(f, "0x{:04x}", self.addr)?;
            writeln!(f, "\t{}", self.source_file)?;
            writeln!(f, "\t{}", self.line)?;
            write!(f,"\t{:02X?}", self.data)
    }
}

fn str_to_bytes(txt : &str) -> Vec<u8> {
    txt
        .as_bytes()
        .chunks(2)
        .map(std::str::from_utf8)
        .map(|v| 
            u8::from_str_radix(v.unwrap() , 16).unwrap()
        ).collect()
}

fn main() -> io::Result<()> {

    let opts = Opts {
        file : "asm/out/all.syms".to_string()
    };

    let f = File::open(opts.file)?;
    let f = BufReader::new(f);

    let addr_re = Regex::new(r"^(?P<addr>[0-9A-F]{4})\s+(?P<data>[0-9A-F]+)\s+\(\s*(?P<source_file>.*)\):(?P<line>\d+)").unwrap();

    let mut result : Vec<Chunk> = vec![];

    for line in f.lines() {
        if let Some(matches) = addr_re.captures(&line?) {
            let addr = u16::from_str_radix(&matches["addr"], 16).unwrap();

            let line = matches["line"].parse::<usize>().unwrap();
            let txt_data = &matches["data"];
            let data = str_to_bytes(txt_data);
            let source_file = matches["source_file"].to_string();

            let chunk = Chunk {
                addr, data, source_file, line
            };

            println!("{}\n", chunk);

            result.push(chunk);
        }
    }

    Ok(())
}


