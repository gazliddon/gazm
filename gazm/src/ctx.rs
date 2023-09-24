use crate::{
    ast::AstTree,
    astformat,
    async_tokenize::TokenizeResult,
    binary::{self, AccessType, BinRef, Binary},
    error::{ErrorCollector, GResult, GazmErrorKind, ParseError, UserError},
    gazmsymbols::{Serializable, SymbolTree},
    lookup::LabelUsageAndDefintions,
    lsp::LspConfig,
    messages::Verbosity,
    opts::{BinReference, Opts},
    status_err, status_mess,
    token_store::TokenStore,
    vars::Vars,
};

use grl_sources::{
    fileloader::{FileIo, SourceFileLoader},
    AsmSource, BinToWrite, EditErrorKind, EditResult, Position, SourceDatabase, SourceFile,
    SourceFiles, SourceMapping, TextEditTrait,
};

use grl_utils::{hash::get_hash, PathSearcher};
use anyhow::{Context as AnyContext, Result};
use itertools::Itertools;
use std::path::{Path, PathBuf};

fn join_paths<P: AsRef<Path>, I: Iterator<Item = P>>(i: I, sep: &str) -> String {
    let z: Vec<String> = i.map(|s| s.as_ref().to_string_lossy().into()).collect();
    z.join(sep)
}

#[derive(Debug, PartialEq, Clone)]
pub struct WriteBin {
    pub file: PathBuf,
    pub start: usize,
    pub size: usize,
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
    pub bin_to_write_chunks: Vec<BinToWrite>,
    pub ast: Option<AstTree>,
    pub lookup: Option<LabelUsageAndDefintions>,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub token_store: TokenStore,
    source_file_loader: SourceFileLoader,
    pub cwd: PathBuf,
    pub opts: Opts,
    pub asm_out: AsmOut,
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

fn to_gazm(e: anyhow::Error) -> GazmErrorKind {
    GazmErrorKind::Misc(e.to_string())
}

impl Context {
    pub fn get_untokenized_files(&self, files: &[(Position, PathBuf)]) -> Vec<(Position, PathBuf)> {
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
        self.asm_out = AsmOut::try_from(&self.opts).expect("Can't reset ouput")
    }

    pub fn reset_all(&mut self) {
        let new_ctx = Context::try_from(self.opts.clone()).expect("can't reset all");
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

    pub fn edit_source_file<P: AsRef<Path>, X>(
        &mut self,
        file: P,
        f: impl FnOnce(&mut dyn TextEditTrait) -> EditResult<X>,
    ) -> EditResult<X> {
        if let Ok(source) = self.sources_mut().get_source_mut(&file) {
            let old_hash = source.source.get_hash().clone();

            let res = f(&mut source.source)?;

            // Invalidate token cache if needed
            let new_hash = source.source.get_hash().clone();

            if new_hash != old_hash {
                self.get_token_store_mut().invalidate_tokens(&file);
            }
            Ok(res)
        } else {
            Err(EditErrorKind::NoSourceFile(
                file.as_ref().to_string_lossy().into(),
            ))
        }
    }

    pub fn get_token_store_mut(&mut self) -> &mut TokenStore {
        &mut self.token_store
    }

    pub fn get_tokens_from_full_path<P: AsRef<Path>>(&self, file: P) -> Option<&TokenizeResult> {
        self.token_store.get_tokens(&file)
    }
    pub fn expand_path_and_get_tokens<P: AsRef<Path>>(&self, file: P) -> GResult<&TokenizeResult> {
        let file = self.get_full_path(file)?;
        self.token_store
            .get_tokens(&file)
            .ok_or(GazmErrorKind::Misc("Can't find tokens".to_string()))
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

    pub fn get_size<P: AsRef<Path>>(&self, path: P) -> Result<usize, GazmErrorKind> {
        let path = self.get_vars().expand_vars(path.as_ref().to_string_lossy());
        let ret = self.source_file_loader.get_size(path).map_err(to_gazm)?;
        Ok(ret)
    }

    pub fn read_source<P: AsRef<Path>>(&mut self, path: P) -> Result<&SourceFile, GazmErrorKind> {
        let path = self.get_full_path(&path)?;
        // let path_string = path.to_string_lossy();
        // Is it in the cache?
        let id = if let Ok((id, _)) = self.source_file_loader.sources.get_source(&path) {
            id
        } else {
            let sf = self
                .source_file_loader
                .read_source(&path)
                .map_err(to_gazm)?;
            sf.file_id
        };

        let ret = self
            .source_file_loader
            .sources
            .get_source_file_from_id(id)?;

        Ok(ret)
    }

    pub fn get_full_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, GazmErrorKind> {
        let path: PathBuf = self
            .get_vars()
            .expand_vars(path.as_ref().to_string_lossy())
            .into();

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
        self.write_source_mapping()?;
        self.write_sym_file()?;
        self.write_deps_file()?;
        Ok(())
    }

    fn write_file<P: AsRef<Path>>(&mut self, p: P, txt: &str) -> GResult<String> {
        let full_file_name = self.get_vars().expand_vars(p.as_ref().to_string_lossy());
        std::fs::write(&full_file_name, txt)
            .with_context(|| format!("Unable to write {full_file_name}"))?;
        Ok(full_file_name)
    }

    pub fn write_lst_file(&mut self) -> GResult<()> {
        if let Some(lst_file) = &self.opts.lst_file {
            let lst_file = self.get_vars().expand_vars(lst_file);

            use std::fs;

            let text = self.asm_out.lst_file.lines.join("\n");

            fs::write(&lst_file, text)
                .with_context(|| format!("Unable to write list file {lst_file}"))?;
            status_mess!("Written lst file {lst_file}");
        }

        Ok(())
    }

    pub fn write_ast_file(&mut self) -> GResult<()> {
        if let Some(ast_file) = &self.opts.ast_file {
            let ast_file = self.get_vars().expand_vars(ast_file.to_string_lossy());
            status_mess!("Writing ast: {}", ast_file);

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

    pub fn write_deps_file(&mut self) -> GResult<()> {
        if let Some(deps) = &self.opts.deps_file {
            if let Some(sym_file) = &self.opts.source_mapping {
                let sym_file = self.get_vars().expand_vars(sym_file);
                let sf = self.get_source_file_loader();
                let read = join_paths(sf.get_files_read().iter(), " \\\n");
                let written = join_paths(sf.get_files_written().iter(), " \\\n");
                let deps_line_2 = format!("{written} : {sym_file}");
                let deps_line = format!("{deps_line_2}\n{sym_file} : {read}");

                status_mess!("Writing deps file: {deps}");

                std::fs::write(deps, deps_line)
                    .with_context(|| format!("Unable to write {deps}"))?;
            }
        }

        Ok(())
    }

    pub fn write_sym_file(&mut self) -> GResult<()> {
        if let Some(syms_file) = &self.opts.syms_file {
            let serialized: Serializable = self.get_symbols().into();
            let json_text = serde_json::to_string_pretty(&serialized).unwrap();
            let file_name = self.write_file(syms_file.clone(), &json_text)?;
            status_mess!("Writen symbols file: {}", file_name);
        }

        Ok(())
    }

    pub fn write_source_mapping(&mut self) -> GResult<()> {
        if let Some(sym_file) = &self.opts.source_mapping {
            let sym_file = self.get_vars().expand_vars(sym_file);
            let sd: SourceDatabase = (&*self).into();
            let file_name = sd
                .write_json(&sym_file)
                .with_context(|| format!("Unable to write {sym_file}"))?;

            status_mess!("Written source mappings {file_name}");
        }

        Ok(())
    }

    pub fn checksum_report(&self) {
        if !self.opts.checksums.is_empty() {
            let mess = crate::messages::messages();

            let mut errors = vec![];

            for (name, csum) in &self.opts.checksums {
                let data = self
                    .asm_out
                    .binary
                    .get_bytes(csum.addr, csum.size)
                    .expect("Binary error");
                let this_hash = get_hash(data);
                let expected_hash = csum.sha1.to_lowercase();

                if this_hash != expected_hash {
                    let hash = format!("{name} : {this_hash} != {expected_hash}");
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

impl From<&Context> for SourceDatabase {
    fn from(c: &Context) -> Self {
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
use std::fs::File;
use std::io::Read;

fn get_file_as_byte_vec<P: AsRef<Path>>(filename: P) -> Result<Vec<u8>, std::io::Error> {
    let mut f = File::open(&filename)?;
    let metadata = std::fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    let _read = f.read(&mut buffer)?;
    Ok(buffer)
}

/// Create a Context from the command line Opts
impl TryFrom<&Opts> for AsmOut {
    type Error = String;
    fn try_from(opts: &Opts) -> Result<AsmOut, String> {
        let mut binary = Binary::new(opts.mem_size, AccessType::ReadWrite);

        for BinReference { file, addr } in &opts.bin_references {
            let x = get_file_as_byte_vec(file).map_err(|e| e.to_string())?;
            let bin_ref = BinRef {
                file: file.clone(),
                dest: *addr,
                start: 0,
                size: x.len(),
            };
            binary.bin_reference(&bin_ref, &x)
        }

        let ret = Self {
            errors: ErrorCollector::new(opts.max_errors),
            binary,
            ..Default::default()
        };

        Ok(ret)
    }
}

/// Create a Context from the command line Opts
impl TryFrom<Opts> for Context {
    type Error = String;
    fn try_from(opts: Opts) -> Result<Self, String> {
        let asm_out = AsmOut::try_from(&opts)?;

        let ret = Self {
            asm_out,
            opts,
            ..Default::default()
        };
        Ok(ret)
    }
}
