use std::path::{Path, PathBuf,};
use clap::Parser;
use clap::{App, Arg};

#[derive(Debug, PartialEq, Clone)]
pub struct Context {
    pub verbose: bool,
    pub file : PathBuf,
    pub out: Option<String>,
    pub syms : Option<String>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            verbose: false,
            file: "No FIle".into(),
            out: None,
            syms : None,
        }
    }
}

impl From<clap::ArgMatches> for Context {
    fn from(m : clap::ArgMatches) -> Self {

        let file : PathBuf = m.value_of("file").unwrap().to_string().into();
        let out = m.value_of("out-file").map(|f| f.to_string());
        let syms = m.value_of("symbol-file").map(|f| f.to_string());

        Self {
            verbose : m.is_present("verbose"),
            out,
            file,
            syms,
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
        .get_matches()
}
