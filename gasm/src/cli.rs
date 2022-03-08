use clap::Parser;
use clap::{Arg, Command};
use romloader::sources::{FileIo, SourceFileLoader, Sources, SymbolTable, SymbolTree};
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

    pub fn expand_vars<P: Into<String>>(&self, val: P) -> String {
        let mut ret = val.into();
        for (k, to) in &self.vars {
            let from = format!("$({k})");
            ret = ret.replace(&from, to);
        }
        ret
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Settings {}

#[derive(Debug, Clone)]
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
    pub deps_file: Option<String>,
    pub memory_image_size: usize,
    pub vars: Vars,
    pub symbols: SymbolTree,
    source_file_loader: SourceFileLoader,
}
use anyhow::{anyhow, Result};

impl Context {
    pub fn get_source_file_loader(&self) -> &SourceFileLoader {
        &self.source_file_loader
    }

    pub fn sources(&self) -> &Sources {
        &self.source_file_loader.sources
    }

    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&mut self, path: P, data: C) -> PathBuf {
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        self.source_file_loader.write(path, data)
    }

    pub fn get_size<P: AsRef<Path>>(&self, path: P) -> Result<usize> { 
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        self.source_file_loader.get_size(path)
    }

    pub fn read_source<P: AsRef<Path>>(&mut self, path : P) -> Result<(PathBuf, String, u64)> {
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        self.source_file_loader.read_source(&path.into())
    }

    pub fn read_binary_chunk<P: AsRef<Path>>(
        &mut self,
        path: P,
        r: std::ops::Range<usize>,
    ) -> Result<(PathBuf, Vec<u8>)> {
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        self.source_file_loader.read_binary_chunk(path, r)
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            files: Default::default(),
            verbose: Verbosity::SILENT,
            syms_file: None,
            trailing_comments: false,
            star_comments: false,
            max_errors: 5,
            ignore_relative_offset_errors: false,
            as6809_lst: None,
            as6809_sym: None,
            memory_image_size: 0x10000,
            vars: Vars::new(),
            symbols: SymbolTree::new(),
            source_file_loader: SourceFileLoader::new(),
            deps_file: None,
        }
    }
}

impl From<clap::ArgMatches> for Context {
    fn from(m: clap::ArgMatches) -> Self {
        let mut ret = Self {
            deps_file: m.value_of("deps").map(|f| f.to_string()),
            syms_file: m.value_of("symbol-file").map(|f| f.to_string()),
            as6809_lst: m.value_of("as6809-lst").map(|f| f.to_string()),
            as6809_sym: m.value_of("as6809-sym").map(|f| f.to_string()),
            trailing_comments: m.is_present("trailing-comments"),
            star_comments: m.is_present("star-comments"),
            ignore_relative_offset_errors: m.is_present("ignore-relative-offset-errors"),
            ..Default::default()
        };

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

         ret.verbose = match m.occurrences_of("verbose") {
            0 => Verbosity::SILENT, 
            1 => Verbosity::NORMAL, 
            2 => Verbosity::INFO, 
            3 => Verbosity::INTERESTING, 
            _ => Verbosity::DEBUG,
        };

        if let Some(it) = m.values_of("file") {
            ret.files = it.map(|x| x.into()).collect();
        }

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


        if ret.files.len() != 0 {
            let file = ret.files[0].clone();
            if let Some(dir) = file.parent() {
                ret.source_file_loader = SourceFileLoader::from_search_paths(&[dir]);
            }
        }

        ret
    }
}

pub fn parse() -> clap::ArgMatches {
    Command::new("gasm")
        .about("6809 assembler")
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
                .multiple_occurrences(true)
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
            Arg::new("deps")
                .long("deps")
                .help("Write a Makefile compatible deps file")
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
                .use_value_delimiter(false)
                .validator(|s| s.parse::<usize>())
                .short('m'),
        )
        .arg(
            Arg::new("mem-size")
                .default_value("65536")
                .help("Size of output binary")
                .long("mem-size")
                .takes_value(true)
                .use_value_delimiter(false)
                .validator(|s| s.parse::<usize>()),
        )
        .get_matches()
}
