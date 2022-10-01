use crate::ast::AstTree;
use crate::binary::{self, AccessType, Binary};
use crate::error::{ErrorCollector, GResult, GazmError, ParseError, UserError};
use crate::item::Node;
use crate::macros::Macros;
use crate::messages::Verbosity;
use emu::utils::sources::{
    SourceDatabase, SourceMapping, SourceFiles, SymbolTree, BinToWrite,
};

use emu::utils::sources::fileloader::{FileIo, SourceFileLoader};
use emu::utils::PathSearcher;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::vec;

use emu::utils::sources::BinWriteDesc;

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
use serde::Deserialize;
#[derive(Debug, Clone, Deserialize)]
pub struct CheckSum {
    pub addr: usize,
    pub size: usize,
    pub sha1: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BuildType {
    Build,
    LSP,
    Check,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Settings {}
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct Opts {
    pub verbose: Verbosity,
    pub syms_file: Option<String>,
    pub trailing_comments: bool,
    pub star_comments: bool,
    pub ignore_relative_offset_errors: bool,
    pub as6809_lst: Option<String>,
    pub as6809_sym: Option<String>,
    pub deps_file: Option<String>,
    pub mem_size: usize,
    pub project_file: PathBuf,
    pub lst_file: Option<String>,
    pub encode_blank_lines: bool,
    pub ast_file: Option<PathBuf>,
    pub max_errors: usize,
    #[serde(skip)]
    pub vars: Vec<(String, String)>,
    pub build_async: bool,
    #[serde(skip)]
    pub checksums: HashMap<String, CheckSum>,
    #[serde(skip)]
    pub build_type : BuildType,
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
            mem_size: 65536,
            project_file: "lol".to_owned().into(),
            lst_file: None,
            encode_blank_lines: false,
            ast_file: None,
            max_errors: 10,
            vars: Default::default(),
            build_async: false,
            checksums: Default::default(),
            build_type: BuildType::Build,
        }
    }
}

use emu::utils::sources::nsym;

#[derive(Debug)]
pub struct Context {
    pub symbols: SymbolTree,
    pub source_file_loader: SourceFileLoader,
    pub errors: ErrorCollector,
    pub vars: Vars,
    pub binary: binary::Binary,
    pub source_map: SourceMapping,
    pub lst_file: LstFile,
    pub symbols2: nsym::Symbols,
    pub exec_addr: Option<usize>,
    pub bin_chunks: Vec<BinWriteDesc>,
    pub cwd: PathBuf,
    pub tokens: Vec<Node>,
    pub ast: Option<AstTree>,
    pub opts: Opts,
    pub bin_to_write_chunks: Vec<BinToWrite>,
}

#[derive(Debug, Clone)]
pub struct LstFile {
    pub lines: Vec<String>,
}

impl LstFile {
    pub fn new() -> Self {
        Self { lines: vec![] }
    }

    pub fn add(&mut self, line: &str) {
        self.lines.push(line.to_string())
    }
}

fn to_gazm(e: anyhow::Error) -> GazmError {
    GazmError::Misc(e.to_string())
}

impl Context {
    pub fn get_source_file_loader(&self) -> &SourceFileLoader {
        &self.source_file_loader
    }

    pub fn sources(&self) -> &SourceFiles {
        &self.source_file_loader.sources
    }

    pub fn get_size<P: AsRef<Path>>(&self, path: P) -> Result<usize, GazmError> {
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        let ret = self.source_file_loader.get_size(path).map_err(to_gazm)?;
        Ok(ret)
    }

    pub fn read_source<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(PathBuf, String, u64), GazmError> {
        let path: PathBuf = self
            .vars
            .expand_vars(path.as_ref().to_string_lossy())
            .into();
        let ret = self
            .source_file_loader
            .read_source(&path)
            .map_err(to_gazm)?;
        Ok(ret)
    }

    pub fn get_full_path<P: AsRef<Path>>(&mut self, path: P) -> Result<PathBuf, GazmError> {
        let path: PathBuf = self
            .vars
            .expand_vars(path.as_ref().to_string_lossy())
            .into();

        let ret = self.source_file_loader.get_full_path(&path).map_err(|_| {
            let err = format!("Can't find file {}", path.to_string_lossy());
            GazmError::Misc(err)
        })?;

        Ok(ret)
    }

    pub fn add_parse_error(&mut self, pe: ParseError) -> GResult<()> {
        let ue = UserError::from_parse_error(&pe, self.sources());
        self.errors.add_user_error(ue)
    }

    pub fn add_user_error(&mut self, e: UserError) -> GResult<()> {
        self.errors.add_user_error(e)
    }
}

impl From<&Context> for SourceDatabase {
    fn from(c: &Context) -> Self {
        SourceDatabase::new(
            &c.source_map,
            &c.sources(),
            &c.symbols,
            &c.bin_chunks,
            c.exec_addr,
        )
    }
}

/// Default settings for Context
impl Default for Context {
    fn default() -> Self {
        Self {
            errors: ErrorCollector::new(5),
            binary: binary::Binary::new(65536, binary::AccessType::ReadWrite),
            source_map: Default::default(),
            source_file_loader: Default::default(),
            vars: Default::default(),
            symbols: Default::default(),
            lst_file: LstFile::new(),
            symbols2: nsym::Symbols::new(),
            exec_addr: None,
            bin_chunks: vec![],
            cwd: std::env::current_dir().unwrap(),
            tokens: vec![],
            ast: None,
            opts: Default::default(),
            bin_to_write_chunks: vec![],
        }
    }
}

/// Create a Context from the command line Opts
impl From<Opts> for Context {
    fn from(m: Opts) -> Self {
        let mut ret = Self {
            ..Default::default()
        };

        ret.errors = ErrorCollector::new(m.max_errors);
        ret.binary = Binary::new(m.mem_size, AccessType::ReadWrite);
        ret.opts = m.clone();

        for (k, v) in m.vars {
            ret.vars.set_var(k.to_string(), v.to_string());
        }

        ret
    }
}
