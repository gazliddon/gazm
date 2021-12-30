use std::path::{Path, PathBuf,};
use clap::Parser;
use clap::{App, Arg};

pub struct Context {
    pub verbose: bool,
    pub file : PathBuf,
    pub out: Option<String>,
    pub dump_ast : bool,
    pub pretty_dump_ast : bool,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            verbose: false,
            file: "No FIle".into(),
            out: None,
            dump_ast: false,
            pretty_dump_ast : false,
        }
    }
}

impl From<clap::ArgMatches> for Context {
    fn from(m : clap::ArgMatches) -> Self {

        let file : PathBuf = m.value_of("file").unwrap().to_string().into();
        let out = m.value_of("out").map(|f| f.to_string());

        let ret = Self {
            verbose : m.is_present("verbose"),
            out,
            file,
            pretty_dump_ast : m.is_present("pretty-dump-ast"),
            dump_ast : m.is_present("dump-ast"),
        };

        ret
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
        .arg(Arg::new("dump-ast")
             .help("dump the ast")
             .short('d'))
        .arg(Arg::new("out")
             .help("dump the ast")
             .takes_value(true)
             .use_delimiter(false)
             .short('o'))
        .arg(Arg::new("verbose")
             .help("Verbose mode")
             .use_delimiter(false)
             .short('v'))
        .arg(Arg::new("pretty-dump-ast")
             .help("pretty dump the ast")
             .short('p'))
        .get_matches()
}
