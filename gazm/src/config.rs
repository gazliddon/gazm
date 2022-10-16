use crate::ctx::{CheckSum, Opts};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct YamlConfig {
    #[serde(skip)]
    pub file: PathBuf,
    pub opts: Opts,

    vars: HashMap<String, String>,
    project: Project,
    checksums: HashMap<String, CheckSum>,
}

#[derive(Debug, Clone, Deserialize)]
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
        let mut val: Self = toml::from_str(&f).unwrap();

        val.opts.vars = val
            .vars
            .clone()
            .into_iter()
            .collect::<Vec<(String, String)>>()
            .into();
        val.file = file.as_ref().to_path_buf().clone();
        val.opts.checksums = val.checksums.clone();
        val.opts.assemble_dir = run_dir;
        val
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
