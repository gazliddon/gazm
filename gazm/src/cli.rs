use crate::config;
use crate::messages::Verbosity;
use crate::opts::{BuildType, Opts};

use clap::{Arg, ArgMatches, Command};
use std::path::PathBuf;

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
    let mut path: PathBuf = m
        .get_one::<String>("config-file")
        .cloned()
        .unwrap_or("gazm.toml".to_string())
        .into();

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
        Overides {
            no_async: matches.index_of("no-async").map(|_| true),
            verbosity: Some(match matches.get_count("verbose") {
                0 => Verbosity::Silent,
                1 => Verbosity::Normal,
                2 => Verbosity::Info,
                3 => Verbosity::Interesting,
                _ => Verbosity::Debug,
            }),
        }
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
                o.project_file = m.get_one::<String>("fmt-file").map(PathBuf::from).unwrap();
                o
            }

            Some(("asm", m)) => {
                let mut opts = Opts {
                    deps_file: m.get_one::<String>("deps").map(String::from),
                    source_mapping: m.get_one::<String>("source-mapping").map(String::from),
                    as6809_lst: m.get_one::<String>("as6809-lst").map(String::from),
                    as6809_sym: m.get_one::<String>("as6809-sym").map(String::from),
                    trailing_comments: m.contains_id("trailing-comments"),
                    star_comments: m.contains_id("star-comments"),
                    ignore_relative_offset_errors: m.contains_id("ignore-relative-offset-errors"),
                    project_file: m.get_one::<String>("project-file").unwrap().into(),
                    lst_file: m.get_one::<String>("lst-file").map(String::from),
                    ast_file: m.get_one::<String>("ast-file").map(PathBuf::from),
                    assemble_dir: Some(std::env::current_dir().unwrap()),
                    ..Default::default()
                };

                if m.contains_id("mem-size") {
                    opts.mem_size = m
                        .get_one::<String>("mem-size")
                        .map(|s| s.parse::<usize>().unwrap())
                        .unwrap();
                }

                if m.contains_id("max-errors") {
                    opts.max_errors = m
                        .get_one::<String>("max-errors")
                        .map(|s| s.parse::<usize>().unwrap())
                        .unwrap();
                }

                if let Some(vals) = m.get_occurrences("set") {
                    let vals: Vec<Vec<&String>> = vals.map(Iterator::collect).collect();
                    for x in vals {
                        opts.vars
                            .set_var(x.get(0).unwrap().as_str(), &x.get(1).unwrap().as_str())
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

fn make_config_file_arg() -> Arg {
    Arg::new("config-file")
        .help("load config file")
        // .multiple_values(false)
        .index(1)
        .required(false)
        .default_value("gazm.toml")
}

// fn make_config_file_command(command: &str, about: &str) -> Command {
//     Command::new(command)
//         .about(about)
//         .arg(make_config_file_arg())
// }

////////////////////////////////////////////////////////////////////////////////

pub struct BuildInfo {
    pub version: String,
    pub authors: String,
    pub bin_name: String,
    pub crate_name: String,
}

impl Default for BuildInfo {
    fn default() -> Self {
        const UNKNOWN: &str = "UNKNOWN";
        let version = option_env!("CARGO_PKG_VERSION")
            .unwrap_or(UNKNOWN)
            .to_string();
        let authors = option_env!("CARGO_PKG_AUTHORS")
            .unwrap_or(UNKNOWN)
            .to_string();
        let bin_name = option_env!("CARGO_BIN_NAME").unwrap_or(UNKNOWN).to_string();
        let crate_name = option_env!("CARGO_CRATE_NAME")
            .unwrap_or(UNKNOWN)
            .to_string();

        Self {
            version,
            authors,
            bin_name,
            crate_name,
        }
    }
}

impl BuildInfo {
    pub fn new() -> Self {
        Default::default()
    }
}

////////////////////////////////////////////////////////////////////////////////

lazy_static::lazy_static! {
    static ref BUILD_INFO : BuildInfo = BuildInfo::new();
}

pub fn parse() -> clap::ArgMatches {
    Command::new(BUILD_INFO.crate_name.as_str())
        .about("6809 assembler")
        .author(BUILD_INFO.authors.as_str())
        .version(BUILD_INFO.version.as_str())
        .bin_name(BUILD_INFO.bin_name.as_str())
        // TODO: Look into using groups so replicate this into other subcommands
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .help("Verbose mode")
                .action(clap::ArgAction::Count)
                // .multiple_occurrences(true)
                .short('v'),
        )
        .arg(
            Arg::new("no-async")
                .long("no-async")
                .help("Disable async build"),
        )
        .subcommand_required(true)
        .subcommand(
            Command::new("build")
                .about("build using the config file")
                .arg(make_config_file_arg()),
        )
        .subcommand(
            Command::new("check")
                .about("Check syntax using the config file")
                .arg(make_config_file_arg()),
        )
        .subcommand(
            Command::new("lsp")
                .about("Launch LSP using config file")
                .arg(make_config_file_arg()),
        )
        .subcommand(
            Command::new("asm")
                .about("Assemble using command line switches")
                .arg(
                    Arg::new("project-file")
                        // .multiple_values(false)
                        .required(true),
                )
                .arg(
                    Arg::new("symbol-file")
                        .help("File symbols are written to")
                        .long("symbol-file")
                        .help("symbol file")
                        .num_args(1)
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
                        .num_args(1),
                )
                .arg(
                    Arg::new("as6809-sym")
                        .long("as6809-sym")
                        .help("Load in AS609 sym file to compare against")
                        .num_args(1),
                )
                .arg(
                    Arg::new("deps")
                        .long("deps")
                        .help("Write a Makefile compatible deps file")
                        .num_args(1),
                )
                .arg(
                    Arg::new("set")
                        .long("set")
                        .value_names(&["var", "value"])
                        // .takes_value(true)
                        // .multiple_occurrences(true)
                        .help("Set a value"),
                )
                .arg(
                    Arg::new("max-errors")
                        .default_value("5")
                        .help("Maxium amount of non fatal errors allowed before failing")
                        .long("max-errors")
                        .num_args(1)
                        .use_value_delimiter(false)
                        .value_parser(clap::value_parser!(usize))
                        .short('m'),
                )
                .arg(
                    Arg::new("ast-file")
                        .help("Output AST")
                        .long("ast-file")
                        .num_args(1)
                        .use_value_delimiter(false),
                )
                .arg(
                    Arg::new("lst-file")
                        .help("Output list file")
                        .long("lst-file")
                        .short('l')
                        .num_args(1)
                        .use_value_delimiter(false),
                )
                .arg(
                    Arg::new("mem-size")
                        .default_value("65536")
                        .help("Size of output binary")
                        .long("mem-size")
                        .num_args(1)
                        .use_value_delimiter(false)
                        .value_parser(clap::value_parser!(usize)),
                ),
        )
        .get_matches()
}
