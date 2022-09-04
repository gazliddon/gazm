use crate::error::ErrorCollector;
use crate::messages::Verbosity;
use crate::{
    binary::{AccessType, Binary},
    ctx::{Context, Opts},
};
use clap::{Arg, Command};
use emu::utils::sources::SourceFileLoader;
use std::collections::HashMap;
use std::hash::Hash;
use std::path::{Path, PathBuf};

impl From<clap::ArgMatches> for Opts {
    fn from(m: clap::ArgMatches) -> Self {
        let mut opts = Opts {
            deps_file: m.value_of("deps").map(|f| f.to_string()),
            syms_file: m.value_of("symbol-file").map(|f| f.to_string()),
            as6809_lst: m.value_of("as6809-lst").map(|f| f.to_string()),
            as6809_sym: m.value_of("as6809-sym").map(|f| f.to_string()),
            trailing_comments: m.is_present("trailing-comments"),
            star_comments: m.is_present("star-comments"),
            ignore_relative_offset_errors: m.is_present("ignore-relative-offset-errors"),
            project_file: m.value_of("project-file").unwrap().into(),
            lst_file: m.value_of("lst-file").map(|f| f.to_string()),
            ast_file: m.value_of("ast-file").map(|f| PathBuf::from(f.to_string())),
            ..Default::default()
        };
        opts.verbose = match m.occurrences_of("verbose") {
            0 => Verbosity::Silent,
            1 => Verbosity::Normal,
            2 => Verbosity::Info,
            3 => Verbosity::Interesting,
            _ => Verbosity::Debug,
        };

        if m.is_present("mem-size") {
            opts.mem_size = m
                .value_of("mem-size")
                .map(|s| s.parse::<usize>().unwrap())
                .unwrap();
        }

        if m.is_present("max-errors") {
            opts.max_errors = m
                .value_of("mem-size")
                .map(|s| s.parse::<usize>().unwrap())
                .unwrap();
        }

        if let Some(mut it) = m.values_of("set") {
            let mut x : Vec<_> = vec![];
            while let Some((var, value)) = it.next().and_then(|var| it.next().map(|val| (var, val))) {
                x.push(( var.to_string(), value.to_string() ));
            }
        }

        opts
    }
}

impl From<clap::ArgMatches> for Context {
    fn from(m: clap::ArgMatches) -> Self {
        let mut ret = Self {
            ..Default::default()
        };

        if m.is_present("max-errors") {
            let max_errors = m
                .value_of("max-errors")
                .map(|s| s.parse::<usize>().unwrap())
                .unwrap();
            ret.errors = ErrorCollector::new(max_errors);
        }

        if m.is_present("mem-size") {
            let mem_size = m
                .value_of("mem-size")
                .map(|s| s.parse::<usize>().unwrap())
                .unwrap();
            ret.binary = Binary::new(mem_size, AccessType::ReadWrite);
        }

        if let Some(mut it) = m.values_of("set") {
            while let Some((var, value)) = it.next().and_then(|var| it.next().map(|val| (var, val)))
            {
                ret.vars.set_var(var.to_string(), value.to_string());
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
            Arg::new("project-file")
                .multiple_values(false)
                .index(1)
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
            Arg::new("ast-file")
                .help("Output AST")
                .long("ast-file")
                .takes_value(true)
                .use_value_delimiter(false),
        )
        .arg(
            Arg::new("lst-file")
                .help("Output list file")
                .long("lst-file")
                .short('l')
                .takes_value(true)
                .use_value_delimiter(false),
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
