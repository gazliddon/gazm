use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::messages::Verbosity;
use romloader::sources::{FileIo, SourceFileLoader, Sources, SymbolTable, SymbolTree};

#[derive(Debug, PartialEq, Clone)]
pub struct WriteBin {
    pub file: PathBuf,
    pub start: usize,
    pub size: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Vars {
    vars: HashMap<String, String>,
}

impl Vars {
    pub fn new() -> Self {
        Self {
            vars: Default::default(),
        }
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

#[derive(Debug, PartialEq, Clone)]
pub struct Settings {}

#[derive(Debug, Clone)]
pub struct Context {
    pub verbose: Verbosity,
    pub files: Vec<PathBuf>,
    pub syms_file: Option<String>,
    pub trailing_comments: bool,
    pub star_comments: bool,
    pub max_errors: usize,
    pub ignore_relative_offset_errors: bool,
    pub as6809_lst: Option<String>,
    pub as6809_sym: Option<String>,
    pub deps_file: Option<String>,
    pub memory_image_size: usize,
    pub vars: Vars,
    pub symbols: SymbolTree,
    pub source_file_loader: SourceFileLoader,
}
use anyhow::{anyhow, Result};

impl Context {
    pub fn get_source_file_loader(&self) -> &SourceFileLoader {
        &self.source_file_loader
    }

    pub fn sources(&self) -> &Sources {
        &self.source_file_loader.sources
    }

    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&mut self, path: P, data: C) -> PathBuf {
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        self.source_file_loader.write(path, data)
    }

    pub fn get_size<P: AsRef<Path>>(&self, path: P) -> Result<usize> { 
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        self.source_file_loader.get_size(path)
    }

    pub fn read_source<P: AsRef<Path>>(&mut self, path : P) -> Result<(PathBuf, String, u64)> {
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        self.source_file_loader.read_source(&path.into())
    }

    pub fn read_binary_chunk<P: AsRef<Path>>(
        &mut self,
        path: P,
        r: std::ops::Range<usize>,
    ) -> Result<(PathBuf, Vec<u8>)> {
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        self.source_file_loader.read_binary_chunk(path, r)
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            files: Default::default(),
            verbose: Verbosity::SILENT,
            syms_file: None,
            trailing_comments: false,
            star_comments: false,
            max_errors: 5,
            ignore_relative_offset_errors: false,
            as6809_lst: None,
            as6809_sym: None,
            memory_image_size: 0x10000,
            vars: Vars::new(),
            symbols: SymbolTree::new(),
            source_file_loader: SourceFileLoader::new(),
            deps_file: None,
        }
    }
}
