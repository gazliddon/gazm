#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(dead_code)]
#![allow(unused_imports)]
mod commands;
mod diss;

// use anyhow::Context;

use emu::mem::MemReader;
use gazm::{commands::parse_command, numbers::*};
use nom_locate::LocatedSpan;

pub fn parse() -> clap::ArgMatches {
    use clap::{Arg, Command};
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
                .validator(get_number_err_usize),
        )
        .arg(
            Arg::new("diss-addr")
                .long("diss-addr")
                .default_value("0")
                .help("disassembly address")
                .takes_value(true)
                .validator(get_number_err_usize),
        )
        .get_matches()
}

fn mk_diss_it(ctx : &mut diss::DissCtx, addr : usize) -> impl Iterator<Item=diss::Disassembly> + '_ {
    let mut addr = addr;
    let mut _i = std::iter::from_fn(move || {
        let diss = diss::Diss::new();
        let x = diss.diss(&mut ctx.data, addr);
        addr = x.decoded.next_addr;
        Some(x)
    });
    _i
}

fn to_hex_str(mem : &[u8]) -> String {
    let v : Vec<_> = mem.iter().map(|b| format!("{:02X}",b)).collect();
    v.join(" ")
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rustyline::error::ReadlineError;
    use rustyline::Editor;
    use commands::parse_command;

    let m = parse();
    let mut ctx = diss::DissCtx::from_matches(m)?;

    let mut rl = Editor::<()>::new();

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let mut addr = 0;
    let mut default_hex = false;

    loop {
        use commands::Command;
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let x = parse_command(&line, default_hex);

                if let Ok((_, x)) = x {
                    match &x {
                        Command::Hex => {
                            println!("Default radix : Hex");
                            default_hex = true;
                        },
                        Command::Dec => {
                            println!("Default radix : Decimal");
                            default_hex = false;
                        }

                        Command::Diss(d_addr) => {
                            addr = d_addr.unwrap_or(addr);
                            println!("Disassembling {addr:04X}");
                            let mut i = mk_diss_it(&mut ctx, addr as usize);

                            for _ in 0..10 {
                                if let Some(ins) = i.next() {
                                    let mem = to_hex_str(&ins.decoded.data);
                                    println!(" {:04X}  {mem:15} {}",addr, ins.text);
                                    addr = ins.decoded.next_addr as isize;
                                } else {
                                    break;
                                }
                            }
                        }

                        Command::Mem(d_addr) => {
                            addr = d_addr.unwrap_or(addr);
                            let mut i = MemReader::new(&mut ctx.data);
                            i.set_addr(addr as usize);

                            for _ in 0..8 {
                                print!(" {:04X} ",i.get_addr());
                                for _ in 0..8 {
                                    if let Ok(b) = i.next_byte() {
                                        print!("{b:02X} ");
                                    } else {
                                        print!("?? ");
                                    }
                                }
                                print!("\n");
                            }

                            addr = i.get_addr() as isize;
                        }

                        Command::Quit => {
                            break;
                        },

                        _ => {
                            println!("Unexpected command {line}")
                        } 
                    }
                } else {
                    println!("Syntax error {line}")
                }

                rl.add_history_entry(line.as_str());
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    rl.save_history("history.txt").unwrap();
    Ok(())
}
