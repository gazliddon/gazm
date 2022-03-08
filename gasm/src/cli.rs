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

use crate::ctx::Context;

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
