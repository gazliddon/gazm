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
}

pub struct TomlConfig {
    pub file: PathBuf,
    pub opts: Opts,
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

        let run_dir = file.parent().and_then(|p| {
            (p.to_string_lossy() != "")
                .then_some(p)
                .map(|p| p.to_path_buf())
        });

        let f = std::fs::read_to_string(file).expect("can't read");
        let toml = toml::from_str::<LoadedTomlConfig>(&f);

        match toml {
            Ok(toml) => {
                let mut opts = toml.opts.clone().unwrap_or_default();
                opts.vars = toml
                    .vars
                    .unwrap_or_default()
                    .into_iter()
                    .collect::<Vec<(String, String)>>()
                    .into();

                opts.checksums = toml.checksums.clone().unwrap_or_default();
                opts.assemble_dir = run_dir;
                opts.lsp_config = toml.lsp.unwrap_or_default();

                let config = TomlConfig {
                    file: file.to_path_buf(),
                    opts,
                };

                Ok(config)
            }

            // TODO : Need to generate more info,
            // line numbers etc
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
