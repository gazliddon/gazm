use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Vars {
    vars: HashMap<String, String>,
}

impl From<Vec<(String, String)>> for Vars {
    fn from(input: Vec<(String, String)>) -> Self {
        let mut ret: Self = Default::default();
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

    pub fn set_var<V: Into<String>>(&mut self, var: V, val: V) {
        self.vars.insert(var.into(), val.into());
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
}