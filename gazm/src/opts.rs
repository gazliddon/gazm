#![forbid(unused_imports)]
use std::{collections::HashMap, path::{ PathBuf,Path }};

use crate::{lsp::LspConfig, messages::Verbosity, vars::{ Vars, VarsErrorKind }, error::GResult};
use serde::Deserialize;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BuildType {
    Build,
    Lsp,
    Check,
    Format,
    Test,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BinReference {
    pub file: PathBuf,
    pub addr: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckSum {
    pub addr: usize,
    pub size: usize,
    pub sha1: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct Opts {
    pub verbose: Verbosity,

    pub project_file: PathBuf,

    pub assemble_dir: Option<PathBuf>,
    pub source_mapping: Option<PathBuf>,
    pub syms_file: Option<PathBuf>,
    pub as6809_sym: Option<PathBuf>,
    pub deps_file: Option<PathBuf>,
    pub ast_file: Option<PathBuf>,

    pub ignore_relative_offset_errors: bool,
    pub mem_size: usize,
    pub max_errors: usize,
    pub no_async: bool,
    pub bin_references: Vec<BinReference>,
    pub cpu: crate::cli::CpuKind,

    pub verbose_errors: bool,

    #[serde(skip)]
    pub do_includes: bool,

    #[serde(skip)]
    pub checksums: HashMap<String, CheckSum>,

    #[serde(skip)]
    pub vars: Vars,

    #[serde(skip)]
    pub build_type: BuildType,

    #[serde(skip)]
    pub lsp_config: LspConfig,
}


impl Opts {
    pub fn update_vars(&mut self) {
        self.vars.set_var("PROJECT_FILE", &self.project_file.to_string_lossy());
        self.vars.set_var("MEM_SIZE", &format!("{}", self.mem_size));
    }

    pub fn update_paths(&mut self) -> GResult<()>{
        Ok(())
    }

    pub fn expand_path<P>(&self, p: P) -> Result<PathBuf, VarsErrorKind> 
        where 
        P : AsRef<Path>
    {
        self.vars.expand_vars_in_path(p)
    }
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            cpu: Default::default(),
            assemble_dir: Default::default(),
            source_mapping: Default::default(),
            ignore_relative_offset_errors: false,
            as6809_sym: Default::default(),
            deps_file: Default::default(),
            project_file: Default::default(),
            ast_file: Default::default(),
            vars: Default::default(),
            checksums: Default::default(),
            bin_references: Default::default(),
            lsp_config: Default::default(),
            no_async: false,
            syms_file: Default::default(),
            verbose_errors: false,

            do_includes: true,
            build_type: BuildType::Build,
            verbose: Verbosity::Silent,
            mem_size: 64 * 1024,
            max_errors: 10,

        }
    }
}
