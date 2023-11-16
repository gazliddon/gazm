#![forbid(unused_imports)]
use std::path::{Path, PathBuf};

use crate::{
    ast::{Ast, AstCtx, AstNodeId},
    error::{ErrorCollector, GResult, GazmErrorKind, ParseError, UserError},
    frontend::{tokenize_async, tokenize_no_async, TokenStore, TokenizeResult},
    frontend::{Item, Node},
    gazmsymbols::SymbolTree,
    lookup::LabelUsageAndDefintions,
    messages::status,
    opts::{BinReference, Opts},
    status_err,
    vars::Vars,
};

use grl_sources::{
    fileloader::SourceFileLoader,
    grl_utils::{fileutils, PathSearcher},
    AsmSource, BinToWrite, FileIo, Position, SourceDatabase, SourceFile, SourceFiles,
    SourceMapping,
};

use super::{
    binary::{AccessType, BinRef, Binary},
    fixerupper::FixerUpper,
};

#[derive(Debug)]
pub struct Assembler {
    pub token_store: TokenStore,
    pub source_file_loader: SourceFileLoader,
    pub cwd: PathBuf,
    pub opts: Opts,
    pub asm_out: AsmOut,
    pub fixer_upper: FixerUpper,
}

#[derive(Debug, Clone, Default)]
pub struct AsmOut {
    pub pc_symbol_id: Option<SymbolScopeId>,
    pub direct_page: Option<u8>,
    pub symbols: SymbolTree,
    pub errors: ErrorCollector,
    pub binary: Binary,
    pub source_map: SourceMapping,
    pub lst_file: LstFile,
    pub exec_addr: Option<usize>,
    pub bin_to_write_chunks: Vec<BinToWrite>,
    pub ast: Option<Ast>,
    pub lookup: Option<LabelUsageAndDefintions>,
}

#[derive(Default, Debug, Clone)]
pub struct LstFile {
    pub lines: Vec<String>,
}

impl LstFile {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add(&mut self, line: &str) {
        self.lines.push(line.to_string())
    }
}

impl Assembler {
    pub fn get_untokenized_files(&self, files: &[(Position, PathBuf)]) -> Vec<(Position, PathBuf)> {
        use itertools::Itertools;
        files
            .iter()
            .cloned()
            .filter_map(
                |(pos, path)| match self.get_tokens_from_full_path(&path).is_some() {
                    true => None,
                    false => Some((pos, path)),
                },
            )
            .unique()
            .collect()
    }

    pub fn get_project_file(&self) -> PathBuf {
        self.get_full_path(&self.opts.project_file).unwrap()
    }

    pub fn reset_output(&mut self) {
        self.asm_out = AsmOut::try_from(self.opts.clone()).expect("Can't reset ouput")
    }

    pub fn reset_all(&mut self) {
        let new_ctx = Assembler::try_from(self.opts.clone()).expect("can't reset all");
        *self = new_ctx;
    }

    pub fn get_source_file_loader(&self) -> &SourceFileLoader {
        &self.source_file_loader
    }

    pub fn get_source_file_loader_mut(&mut self) -> &mut SourceFileLoader {
        &mut self.source_file_loader
    }

    pub fn get_token_store_mut(&mut self) -> &mut TokenStore {
        &mut self.token_store
    }

    pub fn get_tokens_from_full_path<P: AsRef<Path>>(&self, file: P) -> Option<&TokenizeResult> {
        self.token_store.get_tokens(&file)
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

    pub fn read_source<P: AsRef<Path>>(&mut self, path: P) -> Result<&SourceFile, GazmErrorKind> {
        let path = self.get_full_path(&path)?;
        // let path_string = path.to_string_lossy();
        // Is it in the cache?
        if let Ok((_, _)) = self.source_file_loader.sources.get_source(&path) {
        } else {
            self.source_file_loader.read_source(&path)?;
        };

        let sf = self
            .source_file_loader
            .sources
            .get_source(&path)
            .map(|(_, b)| b)?;

        Ok(sf)
    }

    pub fn expand_path<P: AsRef<Path>>(&self, path: P) -> GResult<PathBuf> {
        self
            .get_vars()
            .expand_vars_in_path(path).map_err(|e| GazmErrorKind::Misc(e.to_owned()))
    }

    pub fn get_full_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, GazmErrorKind> {
        let path = self.expand_path(path)?;
        let ret = self.source_file_loader.get_full_path(&path).map_err(|_| {
            let err = format!("Can't find file {}", path.to_string_lossy());
            GazmErrorKind::Misc(err)
        })?;

        Ok(ret)
    }

    pub fn add_parse_errors(&mut self, pe: &[ParseError]) -> GResult<()> {
        for e in pe {
            let ue = UserError::from_parse_error(e, self.sources());
            self.asm_out.errors.add_user_error(ue)?
        }
        Ok(())
    }

    pub fn asm_source_to_path(&self, a: &AsmSource) -> Option<PathBuf> {
        match a {
            AsmSource::FromStr => None,
            AsmSource::FileId(id) => self
                .source_file_loader
                .sources
                .get_source_file_from_id(*id)
                .ok()
                .map(|sf| sf.file.clone()),
        }
    }

    pub fn path_to_id<P: AsRef<Path>>(&self, p: P) -> Option<u64> {
        self.sources().get_source(p).ok().map(|r| r.0)
    }
}

impl From<&Assembler> for SourceDatabase {
    fn from(c: &Assembler) -> Self {
        let bins: Vec<_> = c
            .asm_out
            .bin_to_write_chunks
            .iter()
            .map(|c| c.bin_desc.clone())
            .collect();
        SourceDatabase::new(
            &c.asm_out.source_map,
            c.sources(),
            &c.asm_out.symbols,
            &bins,
            c.asm_out.exec_addr,
        )
    }
}

/// Default settings for Context
impl Default for Assembler {
    fn default() -> Self {
        Self {
            source_file_loader: Default::default(),
            cwd: std::env::current_dir().unwrap(),
            opts: Default::default(),
            token_store: TokenStore::new(),
            asm_out: Default::default(),
            fixer_upper: Default::default(),
        }
    }
}

impl BinRef {
    pub fn from_bin_reference(
        BinReference { file, addr }: BinReference,
        r: std::ops::Range<usize>,
    ) -> Self {
        BinRef::new(file, r, addr)
    }
}

/// Create a Context from the command line Opts
impl TryFrom<Opts> for AsmOut {
    type Error = String;

    fn try_from(opts: Opts) -> Result<AsmOut, String> {
        let mut binary = Binary::new(opts.mem_size, AccessType::ReadWrite);

        for br in &opts.bin_references {
            let x = crate::utils::get_file_as_byte_vec(&br.file);

            match x {
                Ok(x) => {
                    let bin_ref = BinRef::from_bin_reference(br.clone(), 0..x.len());
                    binary.add_bin_reference(&bin_ref, &x)
                }

                Err(_) => {
                    status_err!("Cannot load binary ref file {}", br.file.to_string_lossy())
                }
            }
        }

        let mut ret = Self {
            errors: ErrorCollector::new(opts.max_errors),
            binary,
            ..Default::default()
        };

        ret.add_default_symbols(&opts);
        Ok(ret)
    }
}

impl AsmOut {
    /// Add in default symbols from build
    pub fn add_default_symbols(&mut self, opts: &Opts) {
        let mut write = self.symbols.get_root_writer();
        write.create_or_set_scope("gazm");
        write
            .create_and_set_symbol("mem_size", opts.mem_size as i64)
            .expect("Create a symbole for memory size");
    }
}

/// Create a Context from the command line Opts
impl TryFrom<Opts> for Assembler {
    type Error = String;
    fn try_from(opts: Opts) -> Result<Self, String> {
        let asm_out = AsmOut::try_from(opts.clone())?;

        let ret = Self {
            asm_out,
            opts,
            ..Default::default()
        };
        Ok(ret)
    }
}
impl Assembler {
    /// Create an Assembler
    pub fn new(opts: Opts) -> Self {
        Assembler::try_from(opts).expect("Can't create context")
    }

    /// Assemble for the first time
    pub fn assemble(&mut self) -> GResult<()> {
        self.reset_all();
        self.assemble_project()
    }

    /// Reassemble the project keeping the same caches
    /// but clearing the assembly output
    pub fn reassemble(&mut self) -> GResult<()> {
        self.reset_output();
        self.assemble_project()
    }

    fn assemble_project(&mut self) -> GResult<()> {
        let file = self.get_project_file();

        let original_paths = self
            .get_source_file_loader_mut()
            .get_search_paths()
            .to_vec();

        // TODO Do we need to add the parent dir of the projectfile to the search paths?
        if let Some(dir) = file.parent() {
            self.get_source_file_loader_mut().add_search_path(dir);
        }

        let tokes = {
            if self.opts.no_async {
                status("Lexing no async", |_| tokenize_no_async(self))?
            } else {
                status("Lexing async", |_| tokenize_async(self))?
            }

            let file = self.get_project_file();
            self.get_tokens_from_full_path(&file).unwrap().clone()
        };

        self.assemble_tokens(&tokes.node)?;

        self.get_source_file_loader_mut()
            .set_search_paths(&original_paths);

        self.asm_out.errors.raise_errors()
    }

    pub fn set_pc_symbol(&mut self, val: usize) -> Result<(), SymbolError> {
        let id = self.get_pc_symbol_id();
        self.get_symbols_mut().set_value_for_id(id, val as i64)
    }

    pub fn get_pc_symbol_id(&mut self) -> SymbolScopeId {
        if let Some(id) = self.asm_out.pc_symbol_id {
            id
        } else {
            let mut writer = self.get_symbols_mut().get_root_writer();

            let pc_symbol_id = writer
                .set_or_create_and_set_symbol("*", 0)
                .expect("Can't make PC symbol");

            self.asm_out.pc_symbol_id = Some(pc_symbol_id);
            pc_symbol_id
        }
    }

    fn assemble_tokens(&mut self, tokens: &Node) -> GResult<()> {
        let AstCtx { docs, ast_tree, .. } = AstCtx::from_nodes(self, tokens)?;

        status("Compiling", |_| {
            super::sizer::size(self, &ast_tree)?;
            super::compile::compile(self, &ast_tree)?;
            Ok::<(),GazmErrorKind>(())
        })?;

        let lookup = LabelUsageAndDefintions::new(&ast_tree, &self.asm_out.symbols, docs);
        self.asm_out.ast = Some(ast_tree);
        self.asm_out.lookup = Some(lookup);
        Ok(())
    }
}

// Symbol
use crate::gazmsymbols::{SymbolError, SymbolScopeId};

impl Assembler {
    pub fn set_symbol_value(
        &mut self,
        symbol_id: SymbolScopeId,
        val: usize,
    ) -> Result<(), SymbolError> {
        self.get_symbols_mut()
            .set_symbol_for_id(symbol_id, val as i64)
    }
}

// File fuunction
impl Assembler {
    pub fn get_expanded_path<P: AsRef<Path>>(&self, path: P) -> GResult<PathBuf> {
        self.get_vars().expand_vars_in_path(&path).map_err(|e| GazmErrorKind::Misc(e.to_string()))
    }

    pub fn get_abs_path<P: AsRef<Path>>(&mut self, path: P) -> GResult<PathBuf> {
        let path = self.get_expanded_path(path)?;
        Ok(fileutils::abs_path_from_cwd(path))
    }

    pub fn get_file_size<P: AsRef<Path>>(&self, path: P) -> GResult<usize> {
        let path = self.get_expanded_path(&path)?;
        let ret = self.get_source_file_loader().get_size(path)?;
        Ok(ret)
    }

    pub fn read_binary<P: AsRef<Path>>(&mut self, path: P) -> GResult<(PathBuf, Vec<u8>)> {
        let path = self.get_expanded_path(path)?;
        let ret = self.get_source_file_loader_mut().read_binary(path)?;
        Ok(ret)
    }

    pub fn read_binary_chunk<P: AsRef<Path>>(
        &mut self,
        path: P,
        r: std::ops::Range<usize>,
    ) -> GResult<(PathBuf, Vec<u8>)> {
        let path = self.get_expanded_path(&path)?;
        let ret = self
            .get_source_file_loader_mut()
            .read_binary_chunk(path, r)?;
        Ok(ret)
    }
}

impl Assembler {
    pub fn set_dp(&mut self, dp: i64) {
        if dp < 0 {
            self.asm_out.direct_page = None;
        } else {
            self.asm_out.direct_page = Some(dp as u64 as u8);
        }
    }
}

impl Assembler {
    pub fn binary(&self) -> &Binary {
        &self.asm_out.binary
    }

    pub fn binary_mut(&mut self) -> &mut Binary {
        &mut self.asm_out.binary
    }

    pub fn add_bin_to_write<P: AsRef<Path>>(
        &mut self,
        path: P,
        range: std::ops::Range<usize>,
    ) -> GResult<PathBuf> {
        let physical_address = range.start;
        let count = range.len();

        let data = self
            .asm_out
            .binary
            .get_bytes(physical_address, count)?
            .to_vec();

        let path = self.get_abs_path(path)?;
        // Save a record of the file Written
        // this goes into the written sym file eventually
        let bin_to_write = BinToWrite::new(data, &path, range);
        self.asm_out.bin_to_write_chunks.push(bin_to_write);

        // return the path written to, may have been expanded
        Ok(path)
    }
}

// Fixup
impl Assembler {
    pub fn get_fixup_or_default(&self, id: AstNodeId, i: &Item, scope_id: u64) -> Item {
        self.fixer_upper.get_fixup_or_default(scope_id, id, i)
    }

    pub fn add_fixup<I: Into<Item>>(
        &mut self,
        id: AstNodeId,
        v: I,
        scope_id: u64,
    ) -> (u64, AstNodeId) {
        self.fixer_upper.add_fixup(scope_id, id, v.into());
        (scope_id, id)
    }
}

impl Assembler {
    pub fn add_source_mapping(&mut self, pos: &Position, pc: usize) {
        let (_, phys_range) = self.binary().range_to_write_address(pc);

        let si = self.get_source_info(pos);

        if let Ok(si) = si {
            let mem_text = if phys_range.is_empty() {
                String::new()
            } else {
                format!(
                    "{:02X?}",
                    self.asm_out.binary.get_bytes_range(phys_range.clone())
                )
            };

            let m_pc = format!("{:05X} {:04X} {} ", phys_range.start, pc, mem_text);
            let m = format!("{:50}{}", m_pc, si.line_str);

            if !mem_text.is_empty() {
                self.asm_out.lst_file.add(&m);
            }
        }
    }
}
