use std::path::{Path, PathBuf,};
use clap::Parser;
use clap::{App, Arg};

use crate::messages::Verbosity;

#[derive(Debug, PartialEq, Clone)]
pub struct Context {
    pub verbose: Verbosity,
    pub file : PathBuf,
    pub out: Option<String>,
    pub syms : Option<String>,
    pub trailing_comments : bool,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            verbose: Verbosity::NORMAL,
            file: "No FIle".into(),
            out: None,
            syms : None,
            trailing_comments : false,
        }
    }
}

impl From<clap::ArgMatches> for Context {
    fn from(m : clap::ArgMatches) -> Self {
        let file : PathBuf = m.value_of("file").unwrap().to_string().into();
        let out = m.value_of("out-file").map(|f| f.to_string());
        let syms = m.value_of("symbol-file").map(|f| f.to_string());

        let trailing_comments = m.is_present("trailing-comments");

        let verbose = if m.is_present("verbose") {
            Verbosity::INFO
        } else {
            Verbosity::NORMAL
        };

        Self {
            verbose,
            out,
            file,
            syms,
            trailing_comments,
        }
    }
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
             .help("out file")
             .takes_value(true)
             .use_delimiter(false)
             .short('o'))
        .arg(Arg::new("symbol-file")
             .help("symbol file")
             .takes_value(true)
             .use_delimiter(false)
             .short('s'))
        .arg(Arg::new("verbose")
             .help("Verbose mode")
             .use_delimiter(false)
             .short('v'))
        .arg(Arg::new("trailing-comments")
             .help("Treat text after an opcode as a comment")
             .use_delimiter(false)
             .short('t'))
        .get_matches()
}
