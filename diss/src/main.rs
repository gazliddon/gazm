#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(dead_code)]
#![allow(unused_imports)]
mod diss;

// use anyhow::Context;

use clap::{Arg, Command};

pub fn parse() -> clap::ArgMatches {
    Command::new("diss")
        .about("6809 diss")
        .author("gazaxian")
        .version("0.1")
        .arg(
            Arg::new("file")
                .multiple_values(true)
                .index(1)
                .use_value_delimiter(false)
                .required(true),
        )
        .arg(
            Arg::new("base-addr")
                .long("base-addr")
                .default_value("0")
                .help("load address")
                .takes_value(true)
                .validator(|s| s.parse::<usize>()),
        )
        .get_matches()
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = parse();
    let mut ctx = diss::DissCtx::from_matches(m)?;
    let mut diss = diss::Diss::new(&mut ctx.data);

    for _ in 0..1000 {
    let x = diss.diss_next();

    let hex_str : Vec<_> = x.decoded.data.iter().map(|b| format!("{b:02X}")).collect();

    println!("{:04X}   {:30} {}", x.decoded.addr, hex_str.join(" "), x.text);

    }

    Ok(())
}

