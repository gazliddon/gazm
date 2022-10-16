use crate::ast::AstTree;
use crate::astformat;
use crate::binary::{self, AccessType, Binary};
use crate::error::{ErrorCollector, GResult, GazmErrorType, ParseError, UserError};
use crate::item::Node;
use crate::macros::Macros;
use crate::messages::{status_mess, Verbosity};
use crate::status_err;
use crate::token_store::TokenStore;
use crate::vars::Vars;
use emu::utils;

use utils::sources::{
    fileloader::{FileIo, SourceFileLoader},
    BinToWrite, BinWriteDesc, EditResult, SourceDatabase, SourceFile, SourceFiles, SourceMapping,
    SymbolTree, TextEdit, TextEditTrait, TextPos,
};

use utils::PathSearcher;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::vec;

#[derive(Debug, PartialEq, Clone)]
pub struct WriteBin {
    pub file: PathBuf,
    pub start: usize,
    pub size: usize,
}

use anyhow::{Context as AnyContext, Result};

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
    pub assemble_dir: Option<PathBuf>,
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
    pub build_async: bool,

    #[serde(skip)]
    pub checksums: HashMap<String, CheckSum>,

    #[serde(skip)]
    pub vars: Vars,

    #[serde(skip)]
    pub build_type: BuildType,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            assemble_dir: Default::default(),
            verbose: Verbosity::Silent,
            syms_file: Default::default(),
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
            build_async: false,
            checksums: Default::default(),
            build_type: BuildType::Build,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AsmOut {
    pub direct_page: Option<u8>,
    pub symbols: SymbolTree,
    pub errors: ErrorCollector,
    pub binary: binary::Binary,
    pub source_map: SourceMapping,
    pub lst_file: LstFile,
    pub exec_addr: Option<usize>,
    pub bin_chunks: Vec<BinWriteDesc>,
    pub bin_to_write_chunks: Vec<BinToWrite>,
    pub tokens: Vec<Node>,
    pub ast: Option<AstTree>,
}

#[derive(Debug, Clone)]
pub struct Context {
    token_store: TokenStore,
    source_file_loader: SourceFileLoader,
    pub cwd: PathBuf,
    pub opts: Opts,
    pub asm_out: AsmOut,
}

#[derive(Debug, Clone)]
pub struct LstFile {
    pub lines: Vec<String>,
}

impl Default for LstFile {
    fn default() -> Self {
        Self {
            lines: Default::default(),
        }
    }
}

impl LstFile {
    pub fn new() -> Self {
        Self { lines: vec![] }
    }

    pub fn add(&mut self, line: &str) {
        self.lines.push(line.to_string())
    }
}

fn to_gazm(e: anyhow::Error) -> GazmErrorType {
    GazmErrorType::Misc(e.to_string())
}

impl Context {
    pub fn reset_output(&mut self) {
        self.asm_out = AsmOut::from(&self.opts)
    }

    pub fn reset_all(&mut self) {
        let new_ctx = Context::from(self.opts.clone());
        *self = new_ctx;
    }

    pub fn get_source_file_loader(&self) -> &SourceFileLoader {
        &self.source_file_loader
    }

    pub fn get_source_file_loader_mut(&mut self) -> &mut SourceFileLoader {
        &mut self.source_file_loader
    }

    pub fn with_source_file<P: AsRef<Path>>(&self, file: P, f: impl FnOnce(&SourceFile)) {
        let (_, source) = self.sources().get_source(&file).unwrap();
        f(source)
    }

    pub fn edit_source_file<P: AsRef<Path>>(
        &mut self,
        file: P,
        f: impl FnOnce(&mut dyn TextEditTrait),
    ) {
        let old_hash = self.sources().get_hash(&file).unwrap();
        let source = self.sources_mut().get_source_mut(&file).unwrap();

        f(&mut source.source);

        // Invalidate token cache if needed
        let new_hash = self.sources().get_hash(&file).unwrap();

        if new_hash != old_hash {
            self.get_token_store_mut().invalidate_tokens(&file);
        }
    }

    pub fn get_token_store_mut(&mut self) -> &mut TokenStore {
        &mut self.token_store
    }

    pub fn get_tokens<P: AsRef<Path>>(&self, file: P) -> Option<&Node> {
        self.token_store.get_tokens(&file)
    }

    pub fn has_tokens<P: AsRef<Path>>(&self, file: P) -> bool {
        self.get_tokens(file).is_some()
    }

    pub fn add_tokens<P: AsRef<Path>>(&mut self, file: P, node: Node) {
        self.token_store.add_tokens(file, node)
    }

    pub fn sources(&self) -> &SourceFiles {
        &self.source_file_loader.sources
    }
    pub fn sources_mut(&mut self) -> &mut SourceFiles {
        &mut self.source_file_loader.sources
    }

    pub fn get_vars(&self) -> &Vars {
        &self.opts.vars
    }

    pub fn get_size<P: AsRef<Path>>(&self, path: P) -> Result<usize, GazmErrorType> {
        let path = self.get_vars().expand_vars(path.as_ref().to_string_lossy());
        let ret = self.source_file_loader.get_size(path).map_err(to_gazm)?;
        Ok(ret)
    }

    // TODO : This needs to return a reference to the source file NOT a big old copy of it
    pub fn read_source<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(PathBuf, String, u64), GazmErrorType> {
        let path = self.get_full_path(&path)?;
        // let path_string = path.to_string_lossy();

        // Is it in the cache?
        if let Ok((id, sf)) = self.source_file_loader.sources.get_source(&path) {
            let ret = (sf.file.clone(), sf.source.source.clone(), id);
            Ok(ret)
        } else {
            self.source_file_loader.read_source(&path).map_err(to_gazm)
        }
    }

    pub fn get_full_path<P: AsRef<Path>>(&mut self, path: P) -> Result<PathBuf, GazmErrorType> {
        let path: PathBuf = self
            .get_vars()
            .expand_vars(path.as_ref().to_string_lossy())
            .into();

        let ret = self.source_file_loader.get_full_path(&path).map_err(|_| {
            let err = format!("Can't find file {}", path.to_string_lossy());
            GazmErrorType::Misc(err)
        })?;

        Ok(ret)
    }

    pub fn add_parse_error(&mut self, pe: ParseError) -> GResult<()> {
        let ue = UserError::from_parse_error(&pe, self.sources());
        self.asm_out.errors.add_user_error(ue)
    }

    pub fn add_user_error(&mut self, e: UserError) -> GResult<()> {
        self.asm_out.errors.add_user_error(e)
    }

    pub fn write_bin_chunks(&mut self) -> GResult<()> {
        for bin_to_write in &self.asm_out.bin_to_write_chunks {
            let physical_address = bin_to_write.bin_desc.addr.start;
            let count = bin_to_write.bin_desc.addr.len();
            let p = &bin_to_write.bin_desc.file;

            status_mess!(
                "Writing binary: {} ${physical_address:x} ${count:x}",
                p.to_string_lossy()
            );
            self.source_file_loader.write(p, &bin_to_write.data);
        }
        Ok(())
    }

    /// Write any outputs that need writing
    pub fn write_ouputs(&mut self) -> GResult<()> {
        self.write_bin_chunks()?;
        self.checksum_report();
        self.write_lst_file()?;
        self.write_sym_file()
    }

    pub fn write_lst_file(&mut self) -> GResult<()> {
        use std::fs;

        if let Some(lst_file) = &self.opts.lst_file {
            let text = self.asm_out.lst_file.lines.join("\n");
            fs::write(lst_file, text)
                .with_context(|| format!("Unable to write list file {lst_file}"))?;
            status_mess!("Written lst file {lst_file}");
        }
        Ok(())
    }

    pub fn write_ast_file(&mut self) -> GResult<()> {
        if let Some(ast_file) = &self.opts.ast_file {
            status_mess!("Writing ast: {}", ast_file.to_string_lossy());
            status_err!("Not done!");
            if let Some(ast) = &self.asm_out.ast {
                let x = astformat::as_string(ast.root());
                println!("{x}");
            } else {
                status_err!("No AST file to write");
            }
        }
        Ok(())
    }
    pub fn write_sym_file(&mut self) -> GResult<()> {
        if let Some(sym_file) = &self.opts.syms_file {
            let _syms = self.asm_out.symbols.clone();

            // let sd : SourceDatabase = ctx.into();

            status_mess!("Writing symbols: {}", sym_file);

            // sd.write_json(sym_file).with_context(||format!("Unable to write {sym_file}"))?;

            if let Some(deps) = &self.opts.deps_file {
                status_mess!("Writing deps file : {deps}");

                let as_string = |s: &PathBuf| -> String { s.to_string_lossy().into() };

                let read: Vec<String> = self
                    .get_source_file_loader()
                    .get_files_read()
                    .iter()
                    .map(as_string)
                    .collect();
                let written: Vec<String> = self
                    .get_source_file_loader()
                    .get_files_written()
                    .iter()
                    .map(as_string)
                    .collect();

                let deps_line_2 = format!("{} : {sym_file}", written.join(" \\\n"));
                let deps_line = format!("{deps_line_2}\n{sym_file} : {}", read.join(" \\\n"));

                std::fs::write(deps, deps_line)
                    .with_context(|| format!("Unable to write {deps}"))?;
            }
        }
        Ok(())
    }

    pub fn checksum_report(&self) {

        if !self.opts.checksums.is_empty() {
            use utils::hash::get_hash;
            let mess = crate::messages::messages();

            let mut errors = vec![];

            for (name, csum) in &self.opts.checksums {
                let data = self.asm_out.binary.get_bytes(csum.addr, csum.size);
                let this_hash = get_hash(data);
                let expected_hash = csum.sha1.to_lowercase();

                if this_hash != expected_hash {
                    let hash = format!("{} : {} != {}", name, this_hash, expected_hash);
                    errors.push(hash);
                }
            }

            if errors.is_empty() {
                status_mess!("✅: {} Checksums correct", self.opts.checksums.len())
            } else {
                mess.error("❌ : Mismatched Checksums");
                mess.indent();
                for name in errors {
                    status_err!("{name} : ❌");
                }
                mess.deindent();
            }
        }
    }
}

impl From<&Context> for SourceDatabase {
    fn from(c: &Context) -> Self {
        SourceDatabase::new(
            &c.asm_out.source_map,
            &c.sources(),
            &c.asm_out.symbols,
            &c.asm_out.bin_chunks,
            c.asm_out.exec_addr,
        )
    }
}

/// Default settings for Context
impl Default for Context {
    fn default() -> Self {
        Self {
            source_file_loader: Default::default(),
            cwd: std::env::current_dir().unwrap(),
            opts: Default::default(),
            token_store: TokenStore::new(),
            asm_out: Default::default(),
        }
    }
}
/// Create a Context from the command line Opts
impl From<&Opts> for AsmOut {
    fn from(opts: &Opts) -> Self {
        Self {
            errors: ErrorCollector::new(opts.max_errors),
            binary: Binary::new(opts.mem_size, AccessType::ReadWrite),
            ..Default::default()
        }
    }
}

/// Create a Context from the command line Opts
impl From<Opts> for Context {
    fn from(opts: Opts) -> Self {
        let asm_out = AsmOut::from(&opts);
        Self {
            asm_out,
            opts,
            ..Default::default()
        }
    }
}
