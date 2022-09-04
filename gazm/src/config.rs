use crate::ctx::Opts;
use serde::Deserialize;

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct YamlConfig {
    opts : Opts,
    vars: HashMap<String,String>,
    project: Project,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub name : String,
}

impl YamlConfig {
    pub fn new() -> Self {
        use toml::Value;
        let config_dir = "/Users/garyliddon/development/stargate";

        let config_file = format!("{config_dir}/gazm.toml");

        let f = std::fs::read_to_string(config_file).expect("can't read");
        let val : Self = toml::from_str(&f).unwrap();

        println!("{:#?}", val);

        val
    }
}

#[allow(unused_imports)]
mod test {
    use super::*;

    #[test]
    fn yaml_test() {
        let _y = YamlConfig::new();

        assert!(false)
    }
}


