use clap::Parser;
use clap::{App, Arg};
use romloader::ResultExt;
use std::path::{Path, PathBuf};

use crate::messages::Verbosity;

#[derive(Debug, PartialEq, Clone)]
pub struct Context {
    pub verbose: Verbosity,
    pub files: Vec<PathBuf>,
    pub out: Option<String>,
    pub syms: Option<String>,
    pub trailing_comments: bool,
    pub star_comments: bool,
    pub max_errors: usize,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            files : Default::default(),
            verbose: Verbosity::NORMAL,
            out: None,
            syms: None,
            trailing_comments: false,
            star_comments: false,
            max_errors: 5,
        }
    }
}

impl From<clap::ArgMatches> for Context {
    fn from(m: clap::ArgMatches) -> Self {
        let mut ret = Self {
            out: m.value_of("out-file").map(|f| f.to_string()),
            syms: m.value_of("symbol-file").map(|f| f.to_string()),
            trailing_comments: m.is_present("trailing-comments"),
            star_comments: m.is_present("star-comments"),
            ..Default::default()
        };

        if let Some(it) = m.values_of("file") {
            ret.files = it.map(|x| x.into()).collect();
        }

        if m.is_present("verbose") {
            ret.verbose = Verbosity::INFO;
        };

        if m.is_present("max-errors") {
            ret.max_errors = m
                .value_of("max-errors")
                .map(|s| s.parse::<usize>().unwrap())
                .unwrap();
        }

        ret
    }
}

pub fn parse() -> clap::ArgMatches {
    App::new("gasm")
        .about("6809 assembler")
        .author("gazaxian")
        .version("0.1")
        .arg(
            Arg::new("file")
            .multiple_values(true)
            .index(1)
            .use_delimiter(false)
            .required(true),
        )
        .arg(
            Arg::new("out-file")
            .help("File output is written to")
            .long("out-file")
            .help("out file")
            .takes_value(true)
            .short('o'),
        )
        .arg(
            Arg::new("symbol-file")
            .help("File symbols are written to")
            .long("symbol-file")
            .help("symbol file")
            .takes_value(true)
            .short('s'),
        )
        .arg(
            Arg::new("verbose")
            .long("verbose")
            .help("Verbose mode")
            .short('v'),
        )
        .arg(
            Arg::new("trailing-comments")
            .long("trailing-comments")
            .help("Text at end of line treated as a comment")
            .short('t'),
        )
        .arg(
            Arg::new("star-comments")
            .long("star-comments")
            .help("Lines that start with '*' parsed as comments")
            .short('q'),
        )
        .arg(
            Arg::new("max-errors")
            .default_value("5")
            .help("Maxium amount of non fatal errors allowed before failing")
            .long("max-errors")
            .takes_value(true)
            .use_delimiter(false)
            .validator(|s| s.parse::<usize>())
            .short('m'),
        )
        .get_matches()
}
