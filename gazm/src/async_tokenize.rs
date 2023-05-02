use emu::utils::PathSearcher;
use itertools::Itertools;

use crate::asmctx::AsmCtx;
use crate::ast::AstNodeId;
use crate::binary::AccessType;
use crate::ctx::Context;
use crate::ctx::Opts;
use crate::debug_mess;
use crate::error::GazmErrorKind;
use crate::error::UserError;
use crate::error::{GResult, ParseError};
use crate::gazm::with_state;
use crate::info_mess;
use crate::item::{Item, Node};
use crate::locate::{span_to_pos, Span};
use crate::token_store::TokenStore;
use crate::tokenize::Tokens;

use emu::utils::sources;
use sources::fileloader::{FileIo, SourceFileLoader};
use sources::AsmSource;
use sources::Position;

use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::path::{Path, PathBuf};

use emu::utils::Stack;
use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct TokenizeResult {
    pub file_id: u64,
    pub loaded_file: PathBuf,
    pub requested_file: PathBuf,
    pub node: Node,
    pub errors: Vec<ParseError>,
    pub parent: Option<PathBuf>,
    pub includes: Vec<(Position, PathBuf)>,
}

#[derive(Debug, Clone)]
pub struct TokenizeRequest {
    pub file_id: u64,
    pub expanded_file_name: PathBuf,
    pub requested_file: PathBuf,
    pub parent: Option<PathBuf>,
    pub source: String,
    pub opts: Opts,
}

impl TokenizeRequest {
    pub fn to_result(self, tokens: Tokens) -> TokenizeResult {
        use Item::*;

        let input = Span::new_extra(&self.source, AsmSource::FileId(self.file_id));
        let item = TokenizedFile(self.expanded_file_name.clone(), self.parent.clone());
        let node = Node::new_with_children(item, &tokens.tokens, span_to_pos(input));

        TokenizeResult {
            file_id: self.file_id,
            loaded_file: self.expanded_file_name,
            requested_file: self.requested_file,
            node,
            errors: tokens.parse_errors,
            parent: self.parent,
            includes: tokens.includes,
        }
    }
}

/// Async friendly tokenizer
pub fn tokenize_text(req: &TokenizeRequest) -> GResult<TokenizeResult> {
    let opts = &req.opts;
    let input = Span::new_extra(&req.source, AsmSource::FileId(req.file_id));
    let tokens = Tokens::from_text(opts, input)?;
    Ok(req.clone().to_result(tokens))
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub struct IncludeStack {
    include_stack: Stack<PathBuf>,
}

#[derive(Error, Debug, Clone)]
pub enum IncludeErrorKind {
    #[error("Circular include")]
    CircularInclude,
    #[error("At the top of the stack!")]
    CantPop,
}

impl IncludeStack {
    pub fn new() -> Self {
        Self {
            include_stack: Default::default(),
        }
    }

    pub fn top(&self) -> Option<PathBuf> {
        self.include_stack.top().cloned()
    }

    pub fn push(&mut self, p: &PathBuf) -> Result<(), IncludeErrorKind> {
        if self.include_stack.get_deque().contains(p) {
            Err(IncludeErrorKind::CircularInclude)
        } else {
            self.include_stack.push(p);
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Result<(), IncludeErrorKind> {
        if self.include_stack.is_empty() {
            Err(IncludeErrorKind::CantPop)
        } else {
            self.include_stack.pop();
            Ok(())
        }
    }

    fn is_circular(&self, full_path: &PathBuf) -> bool {
        self.include_stack.get_deque().contains(full_path)
    }
}
////////////////////////////////////////////////////////////////////////////////

pub enum GetTokensResult {
    Tokens(TokenizeResult),
    Request(TokenizeRequest),
}

impl Context {
    fn get_full_paths_with_parent(
        &self,
        paths: &[(Position, PathBuf)],
        parent: &Option<PathBuf>,
    ) -> GResult<Vec<(PathBuf, Option<PathBuf>)>> {
        let full_paths = self
            .get_full_paths(paths)?
            .into_iter()
            .map(|(_, path)| (path, parent.clone()))
            .collect();

        Ok(full_paths)
    }

    fn get_full_paths(&self, paths: &[(Position, PathBuf)]) -> GResult<Vec<(Position, PathBuf)>> {
        let res: GResult<Vec<(Position, PathBuf)>> = paths
            .iter()
            .unique()
            .map(|(pos, path)| Ok((pos.clone(), self.get_full_path(path)?)))
            .collect();
        res
    }
    fn get_tokens<P: AsRef<Path>>(
        &mut self,
        requested_file: P,
        parent: Option<PathBuf>,
    ) -> GResult<GetTokensResult> {
        let requested_file = requested_file.as_ref().to_path_buf();
        let expanded_file = self.get_full_path(&requested_file)?;

        if let Some(tokes) = self.get_tokens_from_full_path(&expanded_file) {
            Ok(GetTokensResult::Tokens(tokes.clone()))
        } else {
            let (expanded_file_name, source, file_id) = self
                .read_source(&requested_file)
                .map(|(file, source, file_id)| (file, source, file_id))?;

            let toke_req = TokenizeRequest {
                file_id,
                expanded_file_name,
                requested_file,
                parent,
                source,
                opts: self.opts.clone(),
            };

            Ok(GetTokensResult::Request(toke_req))
        }
    }
}

pub fn tokenize(ctx: &mut Context) -> GResult<()> {
    let mut files_to_process: Vec<(PathBuf, Option<PathBuf>)> = vec![];

    files_to_process.push((ctx.get_project_file(), None));

    while !files_to_process.is_empty() {
        let mut requests: Vec<TokenizeRequest> = vec![];
        let mut files_to_process_next: Vec<(PathBuf, Option<PathBuf>)> = vec![];

        for (requested_file, parent) in files_to_process.iter() {
            let tokes = ctx.get_tokens(&requested_file, parent.clone())?;

            match tokes {
                // If I have tokens then add any includes to the list of files to tokeniz
                GetTokensResult::Tokens(tokes) => {
                    info_mess!("Got tokes for {}", tokes.loaded_file.to_string_lossy());
                    files_to_process_next.extend(ctx.get_full_paths_with_parent(
                        &tokes.includes,
                        &parent,
                    )?);
                }
                // If I don't have tokens then add it to a q of requestes
                GetTokensResult::Request(req) => {
                    let file = req.expanded_file_name.to_string_lossy();
                    info_mess!("Requesting tokes for {file}");
                    requests.push(req)
                }
            };
        }

        let tokenized: Vec<GResult<TokenizeResult>> = if ctx.opts.no_async {
            requests.iter().map(|req| tokenize_text(req)).collect()
        } else {
            use rayon::prelude::*;
            requests.par_iter().map(|req| tokenize_text(req)).collect()
        };

        for tokes in tokenized.into_iter() {
            match tokes {
                Ok(res) => {
                    info_mess!("Tokenized! {}", res.loaded_file.to_string_lossy());
                    ctx.add_parse_errors(&res.errors)?;
                    files_to_process_next.push((res.requested_file.clone(), res.parent.clone()));
                    ctx.get_token_store_mut().add_tokens(res);
                }
                Err(_) => ctx.asm_out.errors.add_result(tokes)?,
            }
        }
        files_to_process = files_to_process_next;
    }

    Ok(())
}

#[allow(unused_imports)]
mod test {
    use std::path;
    use std::{thread::current, time::Instant};

    use crate::config::YamlConfig;
    use crate::messages::Verbosity;

    use super::*;
    #[allow(unused_imports)]
    use pretty_assertions::{assert_eq, assert_ne};

    fn get_config<P: AsRef<Path>>(path: P) -> YamlConfig {
        println!("Trying to read {}", path.as_ref().to_string_lossy());
        YamlConfig::new_from_file(&path)
    }

    fn mk_ctx(config: &YamlConfig) -> crate::ctx::Context {
        let mut dir = config.file.clone();
        dir.pop();
        let mut ctx = crate::ctx::Context::from(config.opts.clone());
        ctx.get_source_file_loader_mut().add_search_path(&dir);
        ctx.get_source_file_loader_mut()
            .add_search_path(format!("{}/src", dir.to_string_lossy()));
        ctx
    }
}
