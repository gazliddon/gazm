#![forbid(unused_imports)]
use std::collections::HashMap;
use std::path::{ Path,PathBuf };

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Vars {
    vars: HashMap<String, String>,
}

impl From<Vec<(String, String)>> for Vars {
    fn from(input: Vec<(String, String)>) -> Self {
        let mut ret: Self = Vars::default();
        for (k, v) in input {
            ret.set_var(k.to_string(), v.to_string());
        }
        ret
    }
}

impl Vars {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_var<V: Into<String>>(&mut self, var: V, value: V) {
        self.vars.insert(var.into(), value.into());
    }

    pub fn get_var(&self, v: &str) -> Option<&String> {
        self.vars.get(v)
    }

    pub fn expand_vars<P: Into<String>>(&self, val: P) -> String {
        let mut ret = val.into();
        for (k, to) in &self.vars {
            let from = format!("$({k})");
            ret = ret.replace(&from, to);
        }
        ret
    }
    pub fn expand_vars_in_path<P: AsRef<Path>>(&self, p: P) -> PathBuf {
        let r = self.expand_vars(p.as_ref().to_string_lossy());
        PathBuf::from(r)
    }
}
