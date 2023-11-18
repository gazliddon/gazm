#![deny(unused_imports)]
use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum VarsErrorKind {
    #[error("Unknown var {0} in {1}")]
    UnableToExpand(String, String)
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Vars {
    vars: HashMap<String, String>,
}

impl From<Vec<(&str, &str)>> for Vars {
    fn from(input: Vec<(&str, &str)>) -> Self {
        let mut ret: Self = Vars::default();
        for (k, v) in input {
            ret.set_var(k.to_string(), v.to_string());
        }
        ret
    }
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

    pub fn expand_vars<P: Into<String>>(&self, val: P) -> Result<String, VarsErrorKind> {
        let mut ret = val.into();
        let original = ret.clone();

        let strip_it = |x: regex::Match| {
            let x = &x.as_str()[2..];
            (&x[0..x.len() - 1]).to_string()
        };

        let regex = Regex::new(r#"\$\(()[^\)]*\)"#).unwrap();

        for to_expand in regex.find_iter(&ret.clone()).map(strip_it).unique() {
            if let Some(to) = self.vars.get(&to_expand) {
                let from = format!("$({to_expand})");
                ret = ret.replace(&from, to);
            } else {
                return Err(VarsErrorKind::UnableToExpand(to_expand.to_string(), original));
            }
        }
        Ok(ret)
    }

    pub fn expand_vars_in_path<P: AsRef<Path>>(&self, p: P) -> Result<PathBuf, VarsErrorKind> {
        let r = self.expand_vars(p.as_ref().to_string_lossy())?;
        Ok(PathBuf::from(r))
    }
}

#[allow(unused_imports)]
mod test {
    use super::*;

    #[test]
    fn test_expand_success() {
        let vars = vec![("OUTDIR", "outdir"), ("BINGBONG", "bingbong")];
        let vars = Vars::from(vars);
        let a = "$(OUTDIR)/hello/$(BINGBONG)/hello/$(OUTDIR)";
        let y = vars.expand_vars(a).expect("Expaning vars");
        assert_eq!(y, "outdir/hello/bingbong/hello/outdir");
    }

    #[test]
    fn text_expand_failure() {
        let vars = vec![("OUTDIR", "outdir"), ("BINGBONG", "bingbong")];
        let vars = Vars::from(vars);
        let to_expand = "$(OUTDIR)/hello/$(BINGBONG)/hello/$(ERR)";
        let res = vars.expand_vars(to_expand);
        let expected = VarsErrorKind::UnableToExpand("ERR".to_string(),to_expand.to_string());
        assert_eq!(res, Err(expected));
    }
}
