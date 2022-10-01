use crate::ctx::{ Opts, CheckSum };
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct YamlConfig {
    #[serde(skip)]
    pub file : PathBuf,
    #[serde(default)]
    pub opts : Opts,
    #[serde(default)]
    vars: HashMap<String,String>,
    project: Project,
    #[serde(default)]
    checksums: HashMap<String, CheckSum>
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub name : String,
}

impl YamlConfig {
    pub fn new() -> Self {
        use toml::Value;
        let config_file = format!("./gazm.toml");
        Self::new_from_file(&config_file)

    }

    pub fn new_from_file<P: AsRef<std::path::Path>>(file : P) -> Self {
        use toml::Value;
        let f = std::fs::read_to_string(&file).expect("can't read");
        let mut val : Self = toml::from_str(&f).unwrap();

        val.opts.vars = val.vars.clone().into_iter().collect();
        val.file = file.as_ref().to_path_buf().clone();
        val.opts.checksums = val.checksums.clone();
        val
    }

    fn get_toml_file() -> Option<PathBuf> {
        panic!()
    }
}



#[allow(unused_imports)]
mod test {
    use super::*;

    #[test]
    fn yaml_test() {
        let _y = YamlConfig::new();
        print!("{:#?}", _y);
    }
}


