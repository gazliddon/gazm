use crate::ctx::Opts;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct YamlConfig {
    pub opts : Opts,
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
        let config_file = format!("./gazm.toml");
        Self::new_from_file(config_file)
    }

    pub fn new_from_file(file : String) -> Self {
        use toml::Value;
        let config_file = file;
        let f = std::fs::read_to_string(config_file).expect("can't read");
        let mut val : Self = toml::from_str(&f).unwrap();


        let vars : Vec<_> = val.vars.clone().into_iter().map(|z| z).collect();
        val.opts.vars = vars;

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
        assert!(false)
    }
}


