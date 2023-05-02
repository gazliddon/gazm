use crate::ctx::BuildType;
use crate::error::ErrorCollector;
use crate::messages::Verbosity;
use crate::{
    binary::{AccessType, Binary},
    ctx::{Context, Opts},
};

use crate::lsp::LspConfig;
use crate::config;

use clap::{Arg, ArgMatches, Command};
use emu::utils::sources::fileloader::SourceFileLoader;
use nom::ErrorConvert;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::path::{Path, PathBuf};

////////////////////////////////////////////////////////////////////////////////
pub type ConfigError<T> = Result<T, ConfigErrorType>;

#[derive(thiserror::Error, Clone)]
pub enum ConfigErrorType {
    #[error("Missing config file argument")]
    MissingConfigArg,
    #[error("Can't change to directory {0}")]
    InvalidDir(PathBuf),
    #[error("Can't find file {0}")]
    MissingConfigFile(PathBuf),
}

impl std::fmt::Debug for ConfigErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

////////////////////////////////////////////////////////////////////////////////

fn load_config(m: &ArgMatches) -> ConfigError<config::YamlConfig> {
    // Get the config file or use the default gazm.toml
    let mut path: PathBuf = m.value_of("config-file").unwrap_or("gazm.toml").into();

    // If the file is a directory then add gazm.toml to the file
    if path.is_dir() {
        path.push("gazm.toml")
    }

    if !path.is_file() {
        return Err(ConfigErrorType::MissingConfigFile(path));
    }

    let ret = config::YamlConfig::new_from_file(path);

    Ok(ret)
}

fn load_opts_with_build_type(
    m: &ArgMatches,
    build_type: BuildType,
    overides: &Overides,
) -> ConfigError<Opts> {
    let mut conf = load_config(m)?;
    conf.opts.build_type = build_type;
    let opts = overides.apply_overides(&conf.opts);
    Ok(opts)
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Default)]
struct Overides {
    pub no_async: Option<bool>,
    pub verbosity: Option<Verbosity>,
}
impl Overides {
    pub fn new(matches: &clap::ArgMatches) -> Self {
        let mut ret = Overides::default();
        ret.no_async = matches.is_present("no-async").then(|| true);
        ret.verbosity = matches.is_present("verbose").then(||
        match matches.occurrences_of("verbose") {
            0 => Verbosity::Silent,
            1 => Verbosity::Normal,
            2 => Verbosity::Info,
            3 => Verbosity::Interesting,
            _ => Verbosity::Debug,
        }
    );
        ret
    }

    pub fn apply_overides(&self, opts: &Opts) -> Opts {
        let mut ret = opts.clone();

        if let Some(v) = self.verbosity {
            ret.verbose = v
        }

        if let Some(v) = self.no_async {
            ret.no_async = v
        }

        ret
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Opts {
    pub fn from_arg_matches(orig_matches: clap::ArgMatches) -> ConfigError<Opts> {
        let overides = Overides::new(&orig_matches);

        let ret = match orig_matches.subcommand() {
            Some(("build", m)) => load_opts_with_build_type(m, BuildType::Build, &overides)?,
            Some(("check", m)) => load_opts_with_build_type(m, BuildType::Check, &overides)?,
            Some(("lsp", m)) => load_opts_with_build_type(m, BuildType::Lsp, &overides)?,

            Some(("fmt", m)) => {
                let mut o = load_opts_with_build_type(m, BuildType::Format, &overides)?;
                o.project_file = m.value_of("fmt-file").map(PathBuf::from).unwrap();
                o
            }

            Some(("asm", m)) => {
                let mut opts = Opts {
                    deps_file: m.value_of("deps").map(String::from),
                    syms_file: m.value_of("symbol-file").map(String::from),
                    as6809_lst: m.value_of("as6809-lst").map(String::from),
                    as6809_sym: m.value_of("as6809-sym").map(String::from),
                    trailing_comments: m.is_present("trailing-comments"),
                    star_comments: m.is_present("star-comments"),
                    ignore_relative_offset_errors: m.is_present("ignore-relative-offset-errors"),
                    project_file: m.value_of("project-file").unwrap().into(),
                    lst_file: m.value_of("lst-file").map(String::from),
                    ast_file: m.value_of("ast-file").map(PathBuf::from),
                    assemble_dir: Some(std::env::current_dir().unwrap()),
                    ..Default::default()
                };

                if m.is_present("mem-size") {
                    opts.mem_size = m
                        .value_of("mem-size")
                        .map(|s| s.parse::<usize>().unwrap())
                        .unwrap();
                }

                if m.is_present("max-errors") {
                    opts.max_errors = m
                        .value_of("max-errors")
                        .map(|s| s.parse::<usize>().unwrap())
                        .unwrap();
                }

                if let Some(mut it) = m.values_of("set") {

                    while let Some((var, value)) =
                        it.next().and_then(|var| it.next().map(|val| (var, val)))
                    {
                        opts.vars.set_var(var, value)
                    }
                }

                overides.apply_overides(&opts)
            }
            _ => {
                panic!()
            }
        };
        Ok(ret)
    }
}

fn make_config_file_arg<'a>() -> Arg<'a> {
    Arg::new("config-file")
        .help("load config file")
        .multiple_values(false)
        .index(1)
        .required(false)
        .default_value("gazm.toml")
}

fn make_config_file_command<'a>(command: &'a str, about: &'a str) -> Command<'a> {
    Command::new(command)
        .about(about)
        .arg(make_config_file_arg())
}

pub fn parse() -> clap::ArgMatches {
    Command::new("gazm")
        .about("6809 assembler")
        .author("gazaxian")
        .version("0.2.0")
        .bin_name("gazm")
        // TODO: Look into using groups so replicate this into other subcommands
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .help("Verbose mode")
                .multiple_occurrences(true)
                .short('v'),
        )
        .arg(
            Arg::new("no-async")
                .long("no-async")
                .help("Disable async build")
                .multiple_occurrences(false),
        )
        .subcommand_required(true)
        .subcommand(make_config_file_command(
            "build",
            "Build using the config file",
        ))
        .subcommand(make_config_file_command(
            "check",
            "Check syntax using the config file",
        ))
        .subcommand(make_config_file_command(
            "lsp",
            "Launch LSP using config file",
        ))
        .subcommand(
            Command::new("fmt")
                .about("Format a file")
                .arg(
                    Arg::new("config-file")
                        .help("load config file")
                        .multiple_values(false)
                        .index(1)
                        .required(true),
                )
                .arg(
                    Arg::new("fmt-file")
                        .help("file to format")
                        .multiple_values(false)
                        .index(2)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("asm")
                .about("Assemble using command line switches")
                .arg(
                    Arg::new("project-file")
                        .multiple_values(false)
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
