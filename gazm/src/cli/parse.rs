use super::*;
use crate::{messages::Verbosity, opts::*};
use clap::{builder::PathBufValueParser, value_parser, Arg, ArgAction, ArgMatches, Command};
use std::path::PathBuf;

////////////////////////////////////////////////////////////////////////////////

fn load_config(m: &ArgMatches) -> ConfigError<TomlConfig> {
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

    TomlConfig::new_from_file(path)
}

fn load_opts_with_build_type(m: &ArgMatches, build_type: BuildType) -> ConfigError<Opts> {
    let mut conf = load_config(m)?;
    conf.opts.build_type = build_type;
    Ok(conf.opts.clone())
}

////////////////////////////////////////////////////////////////////////////////

impl Opts {
    pub fn from_arg_matches(orig_matches: ArgMatches) -> ConfigError<Opts> {
        let mut opts = match orig_matches.subcommand() {
            Some(("build", m)) => load_opts_with_build_type(m, BuildType::Build)?,
            Some(("check", m)) => load_opts_with_build_type(m, BuildType::Check)?,
            Some(("lsp", m)) => load_opts_with_build_type(m, BuildType::Lsp)?,

            Some(("fmt", m)) => {
                let mut o = load_opts_with_build_type(m, BuildType::Format)?;
                o.project_file = m.get_one::<String>("fmt-file").map(PathBuf::from).unwrap();
                o
            }

            Some(("test", m)) => {
                let opts = Opts {
                    build_type: BuildType::Test,
                    project_file: m.get_one::<PathBuf>("project-file").unwrap().into(),
                    assemble_dir: Some(std::env::current_dir().unwrap()),
                    ..Default::default()
                };

                opts
            }

            Some(("asm", m)) => {
                let mut opts = Opts {
                    deps_file: m.get_one::<String>("deps").map(PathBuf::from),
                    source_mapping: m.get_one::<String>("source-mapping").map(PathBuf::from),
                    as6809_sym: m.get_one::<String>("as6809-sym").map(PathBuf::from),
                    ignore_relative_offset_errors: m.contains_id("ignore-relative-offset-errors"),
                    project_file: m.get_one::<String>("project-file").unwrap().into(),
                    ast_file: m.get_one::<String>("ast-file").map(PathBuf::from),
                    assemble_dir: Some(std::env::current_dir().unwrap()),
                    ..Default::default()
                };

                let to_usize = |s: &String| s.parse::<usize>().ok();

                if m.contains_id("mem-size") {
                    opts.mem_size = m.get_one::<String>("mem-size").and_then(to_usize).unwrap();
                }

                if m.contains_id("max-errors") {
                    opts.max_errors = m
                        .get_one::<String>("max-errors")
                        .and_then(to_usize)
                        .unwrap();
                }

                if let Some(vals) = m.get_occurrences("set") {
                    let vals: Vec<Vec<&String>> = vals.map(Iterator::collect).collect();
                    for x in vals {
                        opts.vars
                            .set_var(x.get(0).unwrap().as_str(), x.get(1).unwrap().as_str())
                    }
                }
                opts
            }
            _ => {
                panic!()
            }
        };

        // Global opts
        opts.verbose = match orig_matches.get_count("verbose") {
            0 => Verbosity::Silent,
            1 => Verbosity::Normal,
            2 => Verbosity::Info,
            3 => Verbosity::Interesting,
            _ => Verbosity::Debug,
        };

        opts.no_async = *orig_matches.get_one("no-async").unwrap();
        opts.use_new_indexed = *orig_matches.get_one("new-index").unwrap();

        opts.update_vars();
        let _ = opts.update_paths();

        Ok(opts)
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

////////////////////////////////////////////////////////////////////////////////

pub fn parse_command_line() -> ArgMatches {
    use super::styling::{get_banner, get_styles};

    Command::new(clap::crate_name!())
        .styles(get_styles())
        .about(get_banner())
        .author(clap::crate_authors!(" : "))
        .version(clap::crate_version!())
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .help("Verbose mode")
                .global(true)
                .action(ArgAction::Count)
                .short('v'),
        )
        .arg(
            Arg::new("no-async")
                .action(ArgAction::SetTrue)
                .global(true)
                .long("no-async")
                .help("Disable async build"),
        )
        .arg(
            Arg::new("new-index")
                .action(ArgAction::SetTrue)
                .global(true)
                .long("new-index")
                .short('n')
                .help("Use new index parser"),
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
            Command::new("test").about("Some test shit").arg(
                Arg::new("project-file")
                    .value_parser(PathBufValueParser::new())
                    .required(true),
            ),
        )
        .subcommand(
            Command::new("asm")
                .about("Assemble using command line switches")
                .arg(
                    Arg::new("project-file")
                        .value_parser(PathBufValueParser::new())
                        .required(true),
                )
                .arg(
                    Arg::new("symbol-file")
                        .value_parser(PathBufValueParser::new())
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
                    Arg::new("as6809-sym")
                        .value_parser(PathBufValueParser::new())
                        .long("as6809-sym")
                        .help("Load in AS609 sym file to compare against")
                        .num_args(1),
                )
                .arg(
                    Arg::new("deps")
                        .value_parser(PathBufValueParser::new())
                        .long("deps")
                        .help("Write a Makefile compatible deps file")
                        .num_args(1),
                )
                .arg(
                    Arg::new("set")
                        .long("set")
                        .value_names(["var", "value"])
                        .help("Set a value"),
                )
                .arg(
                    Arg::new("max-errors")
                        .default_value("5")
                        .help("Maxium amount of non fatal errors allowed before failing")
                        .long("max-errors")
                        .num_args(1)
                        .value_parser(value_parser!(usize))
                        .short('m'),
                )
                .arg(
                    Arg::new("ast-file")
                        .value_parser(PathBufValueParser::new())
                        .help("Output AST")
                        .long("ast-file")
                        .num_args(1),
                )
                .arg(
                    Arg::new("lst-file")
                        .value_parser(PathBufValueParser::new())
                        .help("Output list file")
                        .long("lst-file")
                        .short('l')
                        .num_args(1),
                )
                .arg(
                    Arg::new("mem-size")
                        .value_parser(value_parser!(usize))
                        .default_value("65536")
                        .help("Size of output binary")
                        .long("mem-size")
                        .num_args(1),
                ),
        )
        .get_matches()
}
