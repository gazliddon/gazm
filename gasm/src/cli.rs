use std::path::{Path, PathBuf,};
use clap::Parser;
use clap::{App, Arg};
use romloader::ResultExt;

use crate::messages::Verbosity;

#[derive(Debug, PartialEq, Clone)]
pub struct Context {
    pub verbose: Verbosity,
    pub file : PathBuf,
    pub out: Option<String>,
    pub syms : Option<String>,
    pub trailing_comments : bool,
    pub star_comments: bool,
    pub max_errors : usize,
}

impl Default for Context {
    fn default() -> Self {

        Self {
            verbose: Verbosity::NORMAL,
            file: "No FIle".into(),
            out: None,
            syms : None,
            trailing_comments : false,
            star_comments : false,
            max_errors : 5,
        }
    }
}

impl From<clap::ArgMatches> for Context {
    fn from(m : clap::ArgMatches) -> Self {
        let file : PathBuf = m.value_of("file").unwrap().to_string().into();
        let out = m.value_of("out-file").map(|f| f.to_string());
        let syms = m.value_of("symbol-file").map(|f| f.to_string());

        let trailing_comments = m.is_present("trailing-comments");
        let star_comments = m.is_present("star-comments");

        let verbose = if m.is_present("verbose") {
            Verbosity::INFO
        } else {
            Verbosity::NORMAL
        };

        let max_errors = m.value_of("max-errors").map(|s| s.parse::<usize>().unwrap()).unwrap_or(5);

        Self {
            verbose,
            out,
            file,
            syms,
            trailing_comments,
            star_comments,
            max_errors
        }
    }
}

fn is_usize(s : &str) -> Result<(), String> {
    let _ = s.parse::<usize>().map_err(|_| "not a number".to_string())?;
    Ok(())
}

pub fn parse() -> clap::ArgMatches {
    App::new("gasm")
        .about("6809 assembler")
        .author("gazaxian")
        .version("0.1")
        .arg(Arg::new("file")
             .index(1)
             .use_delimiter(false)
             .required(true))
        .arg(Arg::new("out-file")
             .long("out-file")
             .help("out file")
             .takes_value(true)
             .short('o'))
        .arg(Arg::new("symbol-file")
             .long("symbol-file")
             .help("symbol file")
             .takes_value(true)
             .short('s'))
        .arg(Arg::new("verbose")
             .help("Verbose mode")
             .short('v'))
        .arg(Arg::new("trailing-comments")
             .help("Treat text after an opcode as a comment")
             .short('t'))
        .arg(Arg::new("star-comments")
             .long("star-comments")
             .help("Treat text after an opcode as a comment")
             .short('q'))
        .arg(Arg::new("max-errors")
             .help("Treat text after an opcode as a comment")
             .long("max-errors")
             .takes_value(true)
             .use_delimiter(false)
             .validator(is_usize)
             .short('m'))
        .get_matches()
}
