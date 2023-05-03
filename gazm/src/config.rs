use crate::ctx::{CheckSum, Opts};
use crate::lsp::LspConfig;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Default)]
struct LoadedYamlConfig {
    pub opts: Option<Opts>,
    vars: Option<HashMap<String, String>>,
    // project: Project,
    checksums: Option<HashMap<String, CheckSum>>,
    lsp: Option<LspConfig>,
}

pub struct YamlConfig {
    pub file: PathBuf,
    pub opts: Opts,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Project {
    pub name: String,
}

impl Default for YamlConfig {
    fn default() -> Self {
        let config_file = "./gazm.toml";
        Self::new_from_file(config_file)
    }
}

impl YamlConfig {
    pub fn new_from_file<P: AsRef<std::path::Path>>(file: P) -> Self {

        let file = file.as_ref();

        let run_dir = file.parent().and_then(|p| {
            (p.to_string_lossy() != "")
                .then_some(p)
                .map(|p| p.to_path_buf())
        });

        let f = std::fs::read_to_string(file).expect("can't read");
        let toml: LoadedYamlConfig = toml::from_str(&f).unwrap();

        let mut opts = toml.opts.clone().unwrap_or_default();
        opts.vars = toml
            .vars
            .clone()
            .unwrap_or_default()
            .into_iter()
            .collect::<Vec<(String, String)>>()
            .into();
        opts.checksums = toml.checksums.clone().unwrap_or_default();
        opts.assemble_dir = run_dir;
        opts.lsp_config = toml.lsp.unwrap_or_default();

        YamlConfig {
            file : file.to_path_buf(),
            opts,
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
