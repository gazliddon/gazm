#![deny(unused_imports)]

use crate::{
    assembler::Assembler,
    error::{GResult, GazmErrorKind, ParseError},
    frontend::parse_span,
    info_mess,
    item::{Item, Node},
    // nodeiter::NodeInfo,
    opts::Opts,
    parse::locate::{span_to_pos, Span},
    tokenize::Tokens,
};

use grl_sources::{grl_utils::Stack, Position, SourceFile};

use itertools::Itertools;
use std::path::{Path, PathBuf};
use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct TokenizeRequest {
    pub source_file: SourceFile,
    pub requested_file: PathBuf,
    pub parent: Option<PathBuf>,
    pub opts: Opts,
    pub include_stack: IncludeStack,
}

impl TokenizeRequest {
    pub fn for_single_source_file(source_file: SourceFile, opts: &Opts) -> Self {
        Self {
            requested_file: source_file.file.clone(),
            source_file,
            parent: None,
            opts: opts.clone(),
            include_stack: Default::default(),
        }
    }
}

impl TokenizeRequest {
    pub fn get_file_name_string(&self) -> String {
        self.source_file.file.to_string_lossy().to_string()
    }
    pub fn get_file_name(&self) -> &PathBuf {
        &self.source_file.file
    }
}

#[derive(Debug, Clone)]
pub struct TokenizeResult {
    pub node: Node,
    pub errors: Vec<ParseError>,
    // pub includes: Vec<(Position, PathBuf)>,
    pub request: TokenizeRequest,
}

impl TokenizeResult {
    pub fn get_includes(&self) -> Vec<(Position,PathBuf)> {
        // iter through this node to find includes and put them on the includes stack
        self.node.iter().filter_map(|n| {
            n.node
                .item
                .unwrap_include()
                .map(|path| (n.node.ctx, path.clone()))
        }).collect()
    }
}

impl TryInto<TokenizeResult> for TokenizeRequest {
    type Error = GazmErrorKind;

    fn try_into(self) -> Result<TokenizeResult, Self::Error> {

        let (node,errors) = if self.opts.new_frontend {
            self.new_tokenize()
        } else {
            self.old_tokenize()
        }?;

        Ok( TokenizeResult { node, errors, request: self } )
    }
}

impl TokenizeRequest {

    pub fn new_tokenize(&self) -> GResult<(Node,Vec<ParseError>)> {
        info_mess!("Tokenizing with new front end");
        use crate::frontend::{make_tspan, to_tokens_no_comment};
        let tokens = to_tokens_no_comment(&self.source_file);
        let span = make_tspan(&tokens, &self.source_file);

        // TODO need to collect errors properly - this parser should be an ALL parser
        let (_rest, node) =
            parse_span(span).map_err(|_| GazmErrorKind::Misc("whoops".to_string()))?;
        let item = Item::TokenizedFile(self.source_file.file.clone(), self.parent.clone(), true);
        let node = Node::from_item_kids_tspan(item, &node.children, span);
        let errors = vec![];
        Ok((node,errors))
    }

    pub fn old_tokenize(&self) -> GResult<(Node,Vec<ParseError>)> {
        info_mess!("Tokenizing with old front end");
        let i = self.source_file.get_entire_source();
        let id = self.source_file.file_id;
        let input = Span::new_extra(i, id);
        let tokens = Tokens::from_text(&self.opts, input)?;
        let item = Item::TokenizedFile(self.source_file.file.clone(), self.parent.clone(), false);
        let node = Node::new_with_children(item, &tokens.tokens, span_to_pos(input));
        Ok((node,tokens.parse_errors))
    }
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Default, Debug)]
#[allow(dead_code)]
pub struct IncludeStack {
    include_stack: Stack<PathBuf>,
}

#[derive(Error, Debug, Clone)]
#[allow(dead_code)]
pub enum IncludeErrorKind {
    #[error("Circular include")]
    CircularInclude,
    #[error("At the top of the stack!")]
    CantPop,
}

#[allow(dead_code)]
impl IncludeStack {
    pub fn new() -> Self {
        Default::default()
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
    Tokens(Box<TokenizeResult>),
    Request(Box<TokenizeRequest>),
}

impl Assembler {
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
            .map(|(pos, path)| Ok((*pos, self.get_full_path(path)?)))
            .collect();
        res
    }

    fn get_tokens<P: AsRef<Path>>(
        &mut self,
        requested_file: P,
        parent: Option<PathBuf>,
    ) -> GResult<GetTokensResult> {
        let expanded_file = self.get_full_path(&requested_file)?;

        if let Some(tokes) = self.get_tokens_from_full_path(&expanded_file) {
            Ok(GetTokensResult::Tokens(tokes.clone().into()))
        } else {
            let sf = self.read_source(&requested_file)?;

            let toke_req = TokenizeRequest {
                source_file: sf.clone(),
                requested_file: requested_file.as_ref().to_path_buf(),
                parent,
                opts: self.opts.clone(),
                include_stack: Default::default(),
            };

            Ok(GetTokensResult::Request(toke_req.into()))
        }
    }
}

pub fn tokenize_no_async(ctx: &mut Assembler) -> GResult<()> {
    tokenize(ctx, |to_tokenize| {
        to_tokenize.into_iter().map(|req| req.try_into()).collect()
    })
}

pub fn tokenize_async(ctx: &mut Assembler) -> GResult<()> {
    tokenize(ctx, |to_tokenize| {
        use rayon::prelude::*;
        to_tokenize
            .into_par_iter()
            .map(|req| req.try_into())
            .collect()
    })
}

/// f = handler for tokenize request
pub fn tokenize<F>(ctx: &mut Assembler, f: F) -> GResult<()>
where
    F: Fn(Vec<TokenizeRequest>) -> Vec<GResult<TokenizeResult>>,
{
    let mut files_to_process = vec![(ctx.get_project_file(), None)];

    while !files_to_process.is_empty() {
        let size = files_to_process.len();

        let (to_tokenize, mut incs_to_process) = files_to_process.iter().try_fold(
            (Vec::with_capacity(size), Vec::with_capacity(size)),
            |(mut to_tok, mut incs), (req_file, parent)| {
                use GetTokensResult::*;

                // TODO: Replace parent with incstack
                let tokes = ctx.get_tokens(req_file, parent.clone())?;

                match tokes {
                    Tokens(tokes) => {
                        let req = &tokes.request;
                        let file_name = req.get_file_name();
                        info_mess!("TOKES: Got {:?}", file_name);
                        let includes = tokes.get_includes();
                        let full_paths = ctx.get_full_paths_with_parent(&includes, parent)?;
                        incs.reserve(full_paths.len());
                        incs.extend(full_paths)
                    }
                    // If I don't have tokens then add it to a q of requestes
                    Request(req) => {
                        let file_name = req.get_file_name();
                        info_mess!("TOKES:Requesting {:?}", file_name);
                        to_tok.push(*req)
                    }
                };
                Ok::<_, GazmErrorKind>((to_tok, incs))
            },
        )?;

        let tokenized = f(to_tokenize);

        for tokes in tokenized.into_iter() {
            match tokes {
                Ok(res) => {
                    let req = &res.request;
                    info_mess!("Tokenized! {}", req.source_file.file.to_string_lossy());
                    ctx.add_parse_errors(&res.errors)?;
                    incs_to_process.push((req.requested_file.clone(), req.parent.clone()));
                    ctx.get_token_store_mut().add_tokens(res);
                }
                Err(_) => ctx.asm_out.errors.add_result(tokes)?,
            }
        }

        files_to_process = incs_to_process;
    }

    Ok(())
}

#[allow(unused_imports)]
#[allow(dead_code)]
#[cfg(test)]
mod test {
    use grl_sources::grl_utils::PathSearcher;
    use std::path;
    use std::{thread::current, time::Instant};

    use crate::assembler::Assembler;
    use crate::cli::TomlConfig;
    use crate::messages::Verbosity;

    use super::*;
    #[allow(unused_imports)]
    use pretty_assertions::{assert_eq, assert_ne};

    fn mk_ctx(config: &TomlConfig) -> Assembler {
        let mut dir = config.file.clone();
        dir.pop();
        let mut ctx = Assembler::try_from(config.opts.clone()).expect("Can't make context");
        ctx.get_source_file_loader_mut().add_search_path(&dir);
        ctx.get_source_file_loader_mut()
            .add_search_path(format!("{}/src", dir.to_string_lossy()));
        ctx
    }
}
