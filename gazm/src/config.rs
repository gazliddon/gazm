use crate::ctx::{CheckSum, Opts};
use crate::lsp::LspConfig;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Default)]
struct LoadedYamlConfig {
    #[serde(skip)]
    pub file: PathBuf,
    pub opts: Option<Opts>,

    vars: Option<HashMap<String, String>>,
    project: Project,
    checksums: Option<HashMap<String, CheckSum>>,
    lsp: Option<LspConfig>,
}

pub struct YamlConfig {
    pub file: PathBuf,
    pub opts: Opts,
    project: Project,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Project {
    pub name: String,
}

impl YamlConfig {
    pub fn new() -> Self {
        use toml::Value;
        let config_file = format!("./gazm.toml");
        Self::new_from_file(&config_file)
    }

    pub fn new_from_file<P: AsRef<std::path::Path>>(file: P) -> Self {
        let run_dir = file.as_ref().parent().and_then(|p| {
            (p.to_string_lossy() != "")
                .then_some(p)
                .map(|p| p.to_path_buf())
        });

        use toml::Value;
        let f = std::fs::read_to_string(&file).expect("can't read");
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

        let ret = YamlConfig {
            file :file.as_ref().to_path_buf().clone(),
            opts,
            project: toml.project,
        };

        ret
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
