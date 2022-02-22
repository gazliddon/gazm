use clap::Parser;
use clap::{App, Arg};
use romloader::ResultExt;
use std::os::unix::prelude::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::usize;

use crate::messages::Verbosity;

#[derive(Debug, PartialEq, Clone)]
pub struct WriteBin {
    pub file: PathBuf,
    pub start: usize,
    pub size: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinRef {
    pub file: PathBuf,
    pub start: usize,
    pub size: usize,
    pub dest: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Context {
    pub verbose: Verbosity,
    pub files: Vec<PathBuf>,
    pub out: Option<String>,
    pub syms: Option<String>,
    pub trailing_comments: bool,
    pub star_comments: bool,
    pub max_errors: usize,
    pub ignore_relative_offset_errors: bool,
    pub as6809_lst: Option<String>,
    pub as6809_sym: Option<String>,
    pub bin_refs: Vec<BinRef>,
    pub to_write: Vec<WriteBin>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            files: Default::default(),
            verbose: Verbosity::NORMAL,
            out: None,
            syms: None,
            trailing_comments: false,
            star_comments: false,
            max_errors: 5,
            ignore_relative_offset_errors: false,
            as6809_lst: None,
            as6809_sym: None,
            bin_refs: Default::default(),
            to_write: Default::default(),
        }
    }
}

impl From<clap::ArgMatches> for Context {
    fn from(m: clap::ArgMatches) -> Self {
        let mut ret = Self {
            out: m.value_of("out-file").map(|f| f.to_string()),
            syms: m.value_of("symbol-file").map(|f| f.to_string()),
            as6809_lst: m.value_of("as6809-lst").map(|f| f.to_string()),
            as6809_sym: m.value_of("as6809-sym").map(|f| f.to_string()),
            trailing_comments: m.is_present("trailing-comments"),
            star_comments: m.is_present("star-comments"),
            ignore_relative_offset_errors: m.is_present("ignore-relative-offset-errors"),
            ..Default::default()
        };

        if let Some(mut it) = m.values_of("write-bin") {
            loop {
                if let Some(file) = it.next() {
                    let writer = WriteBin {
                        file: file.into(),
                        start : usize::from_str_radix(it.next().unwrap(), 16).unwrap(),
                        size : usize::from_str_radix(it.next().unwrap(), 16).unwrap()
                    };
                    ret.to_write.push(writer);
                    
                } else {
                    break;
                }
            }
        }

        if let Some(mut it) = m.values_of("bin-ref") {
            loop {
                if let Some(file) = it.next() {
                    let start = it.next().unwrap();
                    let size = it.next().unwrap();
                    let dest = it.next().unwrap();
                    let start = usize::from_str_radix(start, 16).unwrap();
                    let size = usize::from_str_radix(size, 16).unwrap();
                    let dest = usize::from_str_radix(dest, 16).unwrap();
                    let bin_ref = BinRef {
                        file: file.into(),
                        start,
                        size,
                        dest,
                    };
                    ret.bin_refs.push(bin_ref);
                } else {
                    break;
                }
            }
        }

        if let Some(it) = m.values_of("file") {
            ret.files = it.map(|x| x.into()).collect();
        }

        if m.is_present("verbose") {
            ret.verbose = Verbosity::DEBUG;
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
            Arg::new("ignore-relative-offset-errors")
                .long("ignore-relative-offset-errors")
                .help("ignore relative offset errors"),
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
            Arg::new("as6809-lst")
                .long("as6809-lst")
                .help("Load in AS609 lst file to compare against")
                .takes_value(true),
        )
        .arg(
            Arg::new("as6809-sym")
                .long("as6809-sym")
                .help("Load in AS609 sym file to compare against")
                .takes_value(true),
        )
        .arg(
            Arg::new("bin-ref")
                .long("bin-ref")
                .value_names(&["file", "start", "size", "dest"])
                .multiple_occurrences(true)
                .help("Load in binary chunk to compare against")
                .takes_value(true),
        )
        .arg(
            Arg::new("write-bin")
                .long("write-bin")
                .value_names(&["file", "start", "size"])
                .multiple_occurrences(true)
                .help("Write out a binary file"),
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
