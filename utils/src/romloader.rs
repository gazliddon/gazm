use super::{
    rom::{Rom},
    error,
    chunk::*
};

use error::{ResultExt};

use regex::Regex;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader};
// use std::path::PathBuf;

// Convert a string of hex into an array of bytes
fn str_to_bytes(txt: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    let res : Result<Vec<_>,_> = txt.as_bytes()
        .chunks(2)
        .map(std::str::from_utf8)
        .map(|v| u8::from_str_radix(v.unwrap(), 16))
        .collect();

    res
}

fn get_chunks(file : &str) -> error::Result<Vec<Chunk>> {

    let mut loc = Location::new(file, 0);

    let f = File::open(file).context(&loc)?;
    let f = BufReader::new(f);

    let addr_re = Regex::new(
        r"^(?P<addr>[0-9A-F]{4})\s+(?P<data>[0-9A-F]+)\s+\(\s*(?P<source_file>.*)\):(?P<line>\d+)",
    ).unwrap();

    let data_continuation = Regex::new(
        r"^\s+(?P<data>[0-9A-E]+)$",
    ).unwrap();

    let mut chunks: Vec<Chunk> = vec![];

    let mut current_chunk : Option<Chunk> = None;

    for (n, line) in f.lines().enumerate() {

        loc.set_line_number(n);

        let line = &line.context(&loc)?;

        if let Some(the_chunk) = current_chunk.as_mut() {

            if let Some(matches) = data_continuation.captures(line) {
                let data = str_to_bytes(&matches["data"]).context(&loc)?;
                current_chunk.as_mut().unwrap().add_bytes(data);
            } else {
                chunks.push(the_chunk.clone());
                current_chunk = None;
            }
        }

        if current_chunk.is_none() {

            if let Some(matches) = addr_re.captures(line) {
                let mut func = || -> Result<_, std::num::ParseIntError> {

                    let addr = u16::from_str_radix(&matches["addr"], 16)?;

                    let line = matches["line"].parse::<usize>()?;

                    let txt_data = &matches["data"];
                    let data = str_to_bytes(txt_data)?;
                    let source_file = &matches["source_file"];

                    current_chunk = Some(
                        Chunk::new(addr, data, source_file, line)
                    );
                    Ok(())
                };

                func().context(&loc)?
            }
        }
    }

    if let Some(the_chunk) = current_chunk {
        chunks.push(the_chunk);
    } 

    Ok(chunks)
}

impl Rom {
    pub fn from_sym_file(file : &str) -> error::Result<Rom> {
        let chunks = get_chunks(file)?;
        Rom::from_chunks(chunks)
    }
}

