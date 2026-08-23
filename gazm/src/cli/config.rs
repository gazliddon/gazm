use std::{collections::HashMap, path::PathBuf};

use crate::{
    lsp::LspConfig,
    opts::{CheckSum, Opts},
};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
struct LoadedTomlConfig {
    opts: Option<Opts>,
    vars: Option<HashMap<String, String>>,
    checksums: Option<HashMap<String, CheckSum>>,
    lsp: Option<LspConfig>,
    targets: Option<Vec<LoadedTarget>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct LoadedTarget {
    #[serde(rename = "name")]
    _name: String,
    #[serde(flatten)]
    opts: Opts,
    vars: Option<HashMap<String, String>>,
    checksums: Option<HashMap<String, CheckSum>>,
}

pub struct TomlConfig {
    pub file: PathBuf,
    pub opts: Opts,
    pub targets: Vec<Opts>,
}

pub(super) type ConfigError<T> = Result<T, ConfigErrorType>;

#[derive(thiserror::Error, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ConfigErrorType {
    #[error("Missing config file argument")]
    MissingConfigArg,
    #[error("Can't change to directory {0}")]
    InvalidDir(PathBuf),
    #[error("Can't find file {0}")]
    MissingConfigFile(PathBuf),
    #[error("Parse Error in config file: {0}\nline: {2}, col: {3}\n{1}")]
    ParseError(PathBuf, String, usize, usize),
}

impl std::fmt::Debug for ConfigErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl TomlConfig {
    pub fn new_from_file<P: AsRef<std::path::Path>>(file: P) -> ConfigError<Self> {
        let file = file.as_ref();

        let run_dir = file
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()));

        let f = std::fs::read_to_string(file).expect("can't read");
        let toml = toml::from_str::<LoadedTomlConfig>(&f);

        match toml {
            Ok(toml) => {
                let common_vars = toml.vars.clone().unwrap_or_default();
                let common_checksums = toml.checksums.clone().unwrap_or_default();
                let mut opts = toml.opts.clone().unwrap_or_default();
                opts.vars = common_vars
                    .clone()
                    .into_iter()
                    .collect::<Vec<(String, String)>>()
                    .into();

                opts.checksums = toml.checksums.clone().unwrap_or_default();
                opts.assemble_dir = run_dir.clone();
                opts.lsp_config = toml.lsp.clone().unwrap_or_default();

                let targets = toml
                    .targets
                    .unwrap_or_default()
                    .into_iter()
                    .map(|target| {
                        let mut target_opts = target.opts;
                        let mut vars = common_vars.clone();
                        vars.extend(target.vars.unwrap_or_default());
                        target_opts.vars = vars.into_iter().collect::<Vec<_>>().into();
                        target_opts.checksums =
                            target.checksums.unwrap_or_else(|| common_checksums.clone());
                        target_opts.assemble_dir = run_dir.clone();
                        target_opts.lsp_config = toml.lsp.clone().unwrap_or_default();
                        target_opts.target_name = Some(target._name);
                        target_opts.update_vars();
                        target_opts
                    })
                    .collect();

                let config = TomlConfig {
                    file: file.to_path_buf(),
                    opts,
                    targets,
                };

                Ok(config)
            }

            Err(err) => {
                use grl_sources::TextFile;

                let td = TextFile::new(&f);
                let sp = err.span().expect("Trying to retrieve span");
                let tp = td
                    .offset_to_text_pos(sp.start)
                    .expect("trying to get line / col");

                Err(ConfigErrorType::ParseError(
                    file.to_path_buf(),
                    err.message().to_owned(),
                    tp.line() + 1,
                    tp.col() + 1,
                ))
            }
        }
    }
}

#[allow(unused_imports)]
mod test {
    use super::*;
    #[test]
    fn yaml_test() {
        // let _y = YamlConfig::new();
        // print!("{:#?}", _y);
    }
}
