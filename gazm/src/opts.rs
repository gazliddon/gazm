use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    ast::AstTree,
    astformat,
    async_tokenize::TokenizeResult,
    binary::{self, AccessType, Binary},
    error::{ErrorCollector, GResult, GazmErrorKind, ParseError, UserError},
    lookup::LabelUsageAndDefintions,
    lsp::LspConfig,
    messages::Verbosity,
    status_err, status_mess,
    token_store::TokenStore,
    vars::Vars,
};
use sources::{
    fileloader::{FileIo, SourceFileLoader},
    AsmSource, BinToWrite, EditErrorKind, EditResult, Position, SourceDatabase, SourceFile,
    SourceFiles, SourceMapping, TextEditTrait,
};

use grl_utils::{hash::get_hash, PathSearcher};

use anyhow::{Context as AnyContext, Result};
use itertools::Itertools;
use serde::Deserialize;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BuildType {
    Build,
    Lsp,
    Check,
    Format,
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
    pub assemble_dir: Option<PathBuf>,
    pub verbose: Verbosity,
    pub source_mapping: Option<String>,
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
    pub no_async: bool,
    pub bin_references: Vec<BinReference>,

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

impl Default for Opts {
    fn default() -> Self {
        Self {
            assemble_dir: Default::default(),
            verbose: Verbosity::Silent,
            source_mapping: Default::default(),
            trailing_comments: false,
            star_comments: false,
            ignore_relative_offset_errors: false,
            as6809_lst: Default::default(),
            as6809_sym: Default::default(),
            deps_file: Default::default(),
            mem_size: 64 * 1024,
            project_file: Default::default(),
            lst_file: Default::default(),
            encode_blank_lines: false,
            ast_file: Default::default(),
            max_errors: 10,
            vars: Default::default(),
            checksums: Default::default(),
            bin_references: Default::default(),
            build_type: BuildType::Build,
            lsp_config: Default::default(),
            do_includes: true,
            no_async: false,
            syms_file: None,
        }
    }
}
