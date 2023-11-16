#![deny(unused_imports)]
use std::collections::HashMap;
use std::path::{ Path,PathBuf };

use itertools::Itertools;

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

use regex::Regex;

    fn count_expansions(txt: &str) -> usize {
        let regex = Regex::new(r#"\$\(()[^\)]*\)"#).unwrap();
        let x : Vec<_>  = regex.find_iter(txt).collect();
        println!("{x:?}");
        panic!()
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

    pub fn expand_vars<P: Into<String>>(&self, val: P) -> Result<String,String> {
        let mut ret = val.into();
        let original = ret.clone();

        let strip_it = |x: regex::Match| {
            let x = &x.as_str()[2..];
            (&x[0..x.len()-1]).to_string()
        };

        let regex = Regex::new(r#"\$\(()[^\)]*\)"#).unwrap();

        for to_expand in regex.find_iter(&ret.clone()).map(strip_it).unique() {
            if let Some(to)  = self.vars.get(&to_expand) {
            let from = format!("$({to_expand})");
            ret = ret.replace(&from, to);

            } else {
                return Err(format!("Unable to expand var {to_expand} in {original}"))

            }
        }
        Ok(ret)

    }
    pub fn expand_vars_in_path<P: AsRef<Path>>(&self, p: P) -> Result<PathBuf,String> {
        let r = self.expand_vars(p.as_ref().to_string_lossy())?;
        Ok( PathBuf::from(r) )
    }
}

#[allow(unused_imports)]
mod test{
    use super::*;

    #[test]
    fn test_count_caps() {
        let mut vars = Vars::new();

        vars.set_var("OUTDIR", "var 1");
        vars.set_var("BINGBONG", "var 2");

        let a = "$(OUTDIR)/hello/$(BINGBONG)/hello/$(OUTDIR)/$(ERR)";

        let _y = vars.expand_vars(a);
        println!("{_y:?}");

        panic!()
    }

}
