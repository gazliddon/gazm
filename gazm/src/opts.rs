#![forbid(unused_imports)]
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    error::GResult,
    lsp::LspConfig,
    messages::Verbosity,
    vars::{Vars, VarsErrorKind},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BuildType {
    Build,
    Lsp,
    Check,
    Format,
    Test,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

/// Per-target metadata output (contract §5).  Absent writes nothing;
/// `metadata = true` writes the derived `<target>.meta` bundle; a string
/// names the bundle file explicitly, resolved to the same place as the
/// derived name (the build directory).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Metadata {
    Derived(bool),
    File(String),
}

impl Metadata {
    /// The bundle file name this mode requests, if any.
    pub fn bundle_name(&self, target: &str) -> Option<String> {
        match self {
            Metadata::Derived(true) => Some(format!("{target}.meta")),
            Metadata::Derived(false) => None,
            Metadata::File(name) => Some(name.clone()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct Opts {
    pub verbose: Verbosity,

    pub project_file: PathBuf,

    pub assemble_dir: Option<PathBuf>,
    /// Write the metadata bundle — a v5 `TargetInfo` header plus the
    /// source-map and symbols envelopes — when set: `metadata = true`
    /// writes the derived `<target>.meta`, a string names the bundle
    /// explicitly (resolved to the build directory). Absent writes
    /// nothing.
    pub metadata: Option<Metadata>,
    pub deps_file: Option<PathBuf>,
    pub ast_file: Option<PathBuf>,

    pub ignore_relative_offset_errors: bool,
    /// 68000 backend: reproduce MadMac's code-gen restrictions (8-bit
    /// branches error instead of auto-extending, `.w` absolutes must fit
    /// signed 16 bits, zero-displacement branches are illegal). Clear to
    /// get the future saner behaviour (auto-extend branches, unsigned
    /// ranges). Only the cpu68000 backend reads this.
    pub madmac_compat: bool,
    pub mem_size: usize,
    pub max_errors: usize,
    pub no_async: bool,
    pub bin_references: Vec<BinReference>,
    pub cpu: crate::cpukind::CpuKind,

    pub verbose_errors: bool,

    pub error_mismatches: bool,

    #[serde(skip)]
    pub checksums: HashMap<String, CheckSum>,

    #[serde(skip)]
    pub vars: Vars,

    #[serde(skip)]
    pub build_type: BuildType,

    #[serde(skip)]
    pub format_project: bool,

    #[serde(skip)]
    pub lsp_config: LspConfig,

    #[serde(skip)]
    pub target_name: Option<String>,
}

impl Opts {
    pub fn update_vars(&mut self) {
        self.vars
            .set_var("PROJECT_FILE", &self.project_file.to_string_lossy());
        self.vars.set_var("MEM_SIZE", &format!("{}", self.mem_size));
    }

    pub fn update_paths(&mut self) -> GResult<()> {
        Ok(())
    }

    pub fn expand_path<P>(&self, p: P) -> Result<PathBuf, VarsErrorKind>
    where
        P: AsRef<Path>,
    {
        self.vars.expand_vars_in_path(p)
    }
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            cpu: Default::default(),
            assemble_dir: Default::default(),
            ignore_relative_offset_errors: false,
            madmac_compat: false,
            metadata: None,
            deps_file: Default::default(),
            project_file: Default::default(),
            ast_file: Default::default(),
            vars: Default::default(),
            checksums: Default::default(),
            bin_references: Default::default(),
            lsp_config: Default::default(),
            target_name: Default::default(),
            no_async: false,
            verbose_errors: false,
            error_mismatches: false,
            build_type: BuildType::Build,
            format_project: false,
            verbose: Verbosity::Silent,
            mem_size: 64 * 1024,
            max_errors: 10,
        }
    }
}
