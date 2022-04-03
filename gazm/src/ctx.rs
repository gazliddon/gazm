use crate::binary;
use crate::error::{ParseError, UserError, ErrorCollector, GazmError, GResult};
use crate::macros::Macros;
use crate::messages::Verbosity;
use utils::sources::{FileIo, SourceFileLoader, SourceMapping, Sources, SymbolTree};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::gazm::Gazm;

#[derive(Debug, PartialEq, Clone)]
pub struct WriteBin {
    pub file: PathBuf,
    pub start: usize,
    pub size: usize,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Vars {
    vars: HashMap<String, String>,
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

#[derive(Debug, PartialEq, Clone)]
pub struct Settings {}
#[derive(Debug, Clone)]
pub struct Opts {
    pub verbose: Verbosity,
    pub syms_file: Option<String>,
    pub trailing_comments: bool,
    pub star_comments: bool,
    pub ignore_relative_offset_errors: bool,
    pub as6809_lst: Option<String>,
    pub as6809_sym: Option<String>,
    pub deps_file: Option<String>,
    pub memory_image_size: usize,
    pub project_file : PathBuf,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            verbose: Verbosity::Silent,
            syms_file: None,
            trailing_comments: false,
            star_comments: false,
            ignore_relative_offset_errors: false,
            as6809_lst: None,
            as6809_sym: None,
            deps_file: None,
            memory_image_size: 65536,
            project_file: "lol".to_owned().into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    pub symbols: SymbolTree,
    pub source_file_loader: SourceFileLoader,
    pub errors : ErrorCollector,
    pub vars: Vars,
    pub binary : binary::Binary,
    pub source_map : SourceMapping,
}

fn to_gazm(e : anyhow::Error) -> GazmError {
    GazmError::Misc(e.to_string())
}

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

    pub fn get_size<P: AsRef<Path>>(&self, path: P) -> Result<usize, GazmError> {
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        let ret = self.source_file_loader.get_size(path).map_err(to_gazm)?;
        Ok(ret)
    }

    pub fn read_source<P: AsRef<Path>>(&mut self, path: P) -> Result<(PathBuf, String, u64), GazmError> {
        let path: PathBuf = self
            .vars
            .expand_vars(path.as_ref().to_string_lossy())
            .into();
        let ret = self.source_file_loader.read_source(&path).map_err(to_gazm)?;
        Ok(ret)
    }

    pub fn read_binary_chunk<P: AsRef<Path>>(
        &mut self,
        path: P,
        r: std::ops::Range<usize>,
    ) -> Result<(PathBuf, Vec<u8>), GazmError> {
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        let ret = self.source_file_loader.read_binary_chunk(path, r).map_err(to_gazm)?;
        Ok(ret)
    }

    pub fn add_parse_error(&mut self, pe : ParseError ) -> GResult<()> {
        let ue = UserError::from_parse_error(&pe, self.sources());
        self.errors.add_user_error(ue)
    }

    pub fn add_user_error(&mut self, e : UserError) -> GResult<()>{
        self.errors.add_user_error(e)
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            errors: ErrorCollector::new(5),
            binary: binary::Binary::new(65536,binary::AccessType::ReadWrite),
            source_map: Default::default(),
            source_file_loader: Default::default(),
            vars: Default::default(),
            symbols: Default::default(),
        }
    }
}
