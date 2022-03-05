use clap::Parser;
use clap::{App, Arg};
use romloader::sources::{SourceFileLoader, SymbolTable, SymbolTree};
use romloader::ResultExt;
use std::collections::HashMap;
use std::hash::Hash;
use std::os::unix::prelude::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::{usize, vec};

use crate::binary::BinRef;
use crate::messages::Verbosity;

#[derive(Debug, PartialEq, Clone)]
pub struct WriteBin {
    pub file: PathBuf,
    pub start: usize,
    pub size: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Vars {
    vars: HashMap<String, String>,
}

impl Vars {
    pub fn new() -> Self {
        Self {
            vars: Default::default(),
        }
    }

    pub fn set_var<V: Into<String>>(&mut self, var: V, val: V) {
        self.vars.insert(var.into(), val.into());
    }

    pub fn get_var(&self, v: &str) -> Option<&String> {
        self.vars.get(v)
    }

    pub fn expand_vars<P: Into<String>>(&self, val : P) -> String {
        let mut ret = val.into();
        for (k,to) in &self.vars {
            let from= format!("$({k})");
            ret = ret.replace(&from, to);
        }
        ret
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Context {
    pub verbose: Verbosity,
    pub files: Vec<PathBuf>,
    pub syms_file: Option<String>,
    pub trailing_comments: bool,
    pub star_comments: bool,
    pub max_errors: usize,
    pub ignore_relative_offset_errors: bool,
    pub as6809_lst: Option<String>,
    pub as6809_sym: Option<String>,
    pub memory_image_size: usize,
    pub bin_ref_search_paths: Vec<PathBuf>,
    pub vars: Vars,
    pub syms: SymbolTree
}

impl Context {
    pub fn make_source_file_loader(&self) -> SourceFileLoader {
        let file = self.files[0].clone();
        let mut paths = vec![];

        if let Some(dir) = file.parent() {
            paths.push(dir);
        }

        let mut fl = SourceFileLoader::from_search_paths(&paths);
        fl.add_bin_search_paths(&self.bin_ref_search_paths);
        fl
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            files: Default::default(),
            verbose: Verbosity::NORMAL,
            syms_file: None,
            trailing_comments: false,
            star_comments: false,
            max_errors: 5,
            ignore_relative_offset_errors: false,
            as6809_lst: None,
            as6809_sym: None,
            memory_image_size: 0x10000,
            bin_ref_search_paths: Default::default(),
            vars: Vars::new(),
            syms: SymbolTree::new()
        }
    }
}

impl From<clap::ArgMatches> for Context {
    fn from(m: clap::ArgMatches) -> Self {
        let mut ret = Self {
            syms_file: m.value_of("symbol-file").map(|f| f.to_string()),
            as6809_lst: m.value_of("as6809-lst").map(|f| f.to_string()),
            as6809_sym: m.value_of("as6809-sym").map(|f| f.to_string()),
            trailing_comments: m.is_present("trailing-comments"),
            star_comments: m.is_present("star-comments"),
            ignore_relative_offset_errors: m.is_present("ignore-relative-offset-errors"),
            ..Default::default()
        };


        if let Some(it) = m.values_of("bin-ref-search-paths") {
            ret.bin_ref_search_paths = it.map(|p| PathBuf::from(p)).collect();
        }

        if let Some(mut it) = m.values_of("set") {
            loop {
                if let Some((var, value)) =
                    it.next().and_then(|var| it.next().map(|val| (var, val)))
                {
                    ret.vars.set_var(var.to_string(), value.to_string());
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

        if m.is_present("mem-size") {
            ret.memory_image_size = m
                .value_of("mem-size")
                .map(|s| s.parse::<usize>().unwrap())
                .unwrap();
        }

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
            Arg::new("bin-ref-search-paths")
                .long("bin-ref-search-paths")
                .takes_value(true)
                .multiple_values(true)
                .required(false),
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
            Arg::new("set")
                .long("set")
                .value_names(&["var", "value"])
                .takes_value(true)
                .multiple_occurrences(true)
                .help("Set a value"),
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
        .arg(
            Arg::new("mem-size")
                .default_value("65536")
                .help("Size of output binary")
                .long("mem-size")
                .takes_value(true)
                .use_delimiter(false)
                .validator(|s| s.parse::<usize>()),
        )
        .get_matches()
}
