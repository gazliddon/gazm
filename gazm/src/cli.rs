use crate::ctx::BuildType;
use crate::error::ErrorCollector;
use crate::messages::Verbosity;
use crate::{
    binary::{AccessType, Binary},
    ctx::{Context, Opts},
};

use crate::config;

use clap::{Arg, ArgMatches, Command};
use emu::utils::sources::fileloader::SourceFileLoader;
use nom::ErrorConvert;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::path::{Path, PathBuf};

#[derive(thiserror::Error, Clone)]
pub enum ConfigErrorType {
    #[error("Missing config file argument")]
    MissingConfigArg,
    #[error("Can't change to directory {0}")]
    InvalidDir(PathBuf),
    #[error("Can't find file {0}")]
    MissingConfigFile(PathBuf)

}

impl std::fmt::Debug for ConfigErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub type ConfigError<T> = Result<T, ConfigErrorType>;

fn load_config(m: &ArgMatches) -> ConfigError<config::YamlConfig> {
    use std::env::set_current_dir;

    let x = m.value_of("config-file").ok_or(ConfigErrorType::MissingConfigArg)?;
    let path = PathBuf::from(x);

    if !path.is_file() {
        return Err(ConfigErrorType::MissingConfigFile(path))
    }

    let dir = path.parent();

    if let Some(parent) = dir {
        if parent.is_dir() {
            set_current_dir(&parent).map_err(|_| ConfigErrorType::InvalidDir(parent.to_path_buf()))?;
        }
    }

    Ok(config::YamlConfig::new())
}

fn load_opts_with_build_type(m: &ArgMatches, build_type: BuildType) -> ConfigError<Opts> {
    let mut conf = load_config(m)?;
    conf.opts.build_type = build_type;
    Ok( conf.opts )
}

impl Opts {
    pub fn from_arg_matches(orig_matches : clap::ArgMatches) -> ConfigError<Opts> {
        let ret = match orig_matches.subcommand() {
            Some(("build", m)) => load_opts_with_build_type(m, BuildType::Build)?,

            Some(("lsp", m)) => load_opts_with_build_type(m, BuildType::LSP)?,

            Some(("asm", m)) => {
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
                    build_async: m.is_present("build-async"),
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
                    while let Some((var, value)) =
                        it.next().and_then(|var| it.next().map(|val| (var, val)))
                    {
                        opts.vars.push((var.to_string(), value.to_string()));
                    }
                }
                opts
            }
            _ => {
                panic!()
            }
        };

        Ok(ret)
    }
}

pub fn parse() -> clap::ArgMatches {
    Command::new("gazm")
        .about("6809 assembler")
        .author("gazaxian")
        .version("0.1")
        .bin_name("gazm")
        .subcommand_required(true)
        .subcommand(
            Command::new("build").about("use the config file").arg(
                Arg::new("config-file")
                    .help("load config file")
                    .multiple_values(false)
                    .index(1)
                    .required(false)
                    .default_value("gazm.toml"),
            ),
        )
        .subcommand(
            Command::new("asm")
                .arg(
                    Arg::new("project-file")
                        .multiple_values(false)
                        // .index(1)
                        .required(true),
                )
                .arg(
                    Arg::new("build-async")
                        .help("Build asynchronously")
                        .long("async-build")
                        .multiple_values(false)
                        .takes_value(false)
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
                ),
        )
        .get_matches()
}
