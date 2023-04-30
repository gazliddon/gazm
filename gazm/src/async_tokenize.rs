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
use crate::info_mess;
use crate::item::{Item, Node};
use crate::locate::{span_to_pos, Span};
use crate::token_store::TokenStore;
use crate::tokenize::Tokens;
use async_std::prelude::*;

use emu::utils::sources;
use sources::fileloader::{FileIo, SourceFileLoader};
use sources::AsmSource;

use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::path::{Path, PathBuf};

use emu::utils::Stack;

////////////////////////////////////////////////////////////////////////////////
struct IdResource<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    id: u64,
    id_to_resource: HashMap<u64, V>,
    key_to_id: HashMap<K, u64>,
}

impl<K, V> Default for IdResource<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    fn default() -> Self {
        Self {
            id: 0,
            id_to_resource: Default::default(),
            key_to_id: Default::default(),
        }
    }
}

impl<K, V> IdResource<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    pub fn new() -> Self {
        Self {
            id: 0,
            ..Default::default()
        }
    }

    /// Add a resource
    /// returns None if resource already existed
    pub fn add_resource(&mut self, k: K, v: V) -> Option<u64> {
        if self.key_to_id.get(&k).is_some() {
            None
        } else {
            self.id += 1;
            let id = self.id;

            self.id_to_resource.insert(id, v);
            self.key_to_id.insert(k, id);
            Some(id)
        }
    }

    /// Get a resource
    pub fn get_resource(&self, id: u64) -> Option<&V> {
        self.id_to_resource.get(&id)
    }

    pub fn get_id(&self, k: &K) -> Option<u64> {
        self.key_to_id.get(k).cloned()
    }

    pub fn get_resource_from_key(&self, k: &K) -> Option<&V> {
        self.get_id(k).and_then(|id| self.get_resource(id))
    }
}
use std::rc::Rc;
////////////////////////////////////////////////////////////////////////////////
///
use std::sync::{Arc, Mutex};

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

use emu::utils::sources::Position;
/// Tokenize this file and add its tokens to the token store
/// return the includes this file had in it
pub fn tokenize_file_and_add_tokens<P: AsRef<Path>>(
    ctx: &mut Context,
    actual_file: P,
    parent: Option<PathBuf>,
) -> GResult<TokenizeResult> {
    let tokenized = tokenize_file(ctx, &actual_file, parent)?;

    let ts = ctx.get_token_store_mut();
    ts.add_tokens(&actual_file, tokenized.clone());
    info_mess!(
        "Added tokens for {}",
        actual_file.as_ref().to_string_lossy()
    );

    Ok(tokenized)
}

pub fn tokenize_file<P: AsRef<Path>>(
    ctx: &mut Context,
    requested_file: P,
    parent: Option<PathBuf>,
) -> GResult<TokenizeResult> {
    // ctx.with(|ctx| {
    use Item::*;

    let requested_file = requested_file.as_ref().to_path_buf();

    let (file, source, file_id) = ctx
        .read_source(&requested_file)
        .map(|(file, source, file_id)| (file, source, file_id))?;
    let input = Span::new_extra(&source, AsmSource::FileId(file_id));

    let tokens = Tokens::from_text(ctx, input)?;

    for e in &tokens.parse_errors {
        ctx.add_parse_error(e.clone())?;
    }

    if let Some((pos, _path)) = tokens
        .includes
        .iter()
        .find(|(_, path)| *path == requested_file)
    {
        Err(GazmErrorKind::ParseError(
            ParseError::new_from_pos("Self included", pos, true).into(),
        ))
    } else {
        // Collect all of the include files
        // let includes = get_include_files(&tokens);

        let item = TokenizedFile(requested_file.clone(), parent.clone());
        let node = Node::new_with_children(item, &tokens.tokens, span_to_pos(input));

        let ret = TokenizeResult {
            requested_file,
            loaded_file: file,
            file_id,
            node,
            errors: tokens.parse_errors,
            parent,
            includes: tokens.includes,
        };

        Ok(ret)
    }
}

struct TokenizeContext {
    ctx: Arc<Mutex<Context>>,
}

impl Into<Context> for TokenizeContext {
    fn into(self) -> Context {
        let mutex = Arc::try_unwrap(self.ctx).unwrap();
        mutex.into_inner().unwrap()
    }
}

impl TokenizeContext {
    pub fn new(ctx: &Context) -> Self {
        let ctx = Arc::new(Mutex::new(ctx.clone()));
        Self {
            ctx,
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&mut Context) -> R) -> R {
        let mut ctx = self.ctx.lock().unwrap();
        f(&mut ctx)
    }


    // pub fn get_full_path<P: AsRef<Path>>(&self, file: P) -> GResult<PathBuf> {
    //     self.with(|ctx| -> GResult<PathBuf> { ctx.get_full_path(&file) })
    // }

    // pub fn has_tokens_for_file<P: AsRef<Path>>(&self, requested_file: P) -> bool {
    //     self.with(|ctx| -> bool {
    //         // let actual_file = ctx.get_full_path(&requested_file)?;
    //         ctx.get_tokens(&requested_file).is_some()
    //     })
    // }
}
pub fn tokenize<P: AsRef<Path>>(
    ctx: &mut Context,
    requested_file: P,
) -> GResult<TokenizeResult> {

    let include_stack = IncludeStack::new();

    let token_ctx = TokenizeContext::new(ctx);

    tokenize_async_main_loop(&token_ctx, &requested_file, None, include_stack)?;

    let ret = token_ctx.with(|ctx| -> GResult<TokenizeResult> {
        ctx.asm_out.errors.raise_errors()?;
        let toks = ctx.get_tokens(&requested_file).unwrap().clone();
        Ok(toks)
    })?;

    *ctx = token_ctx.into();

    Ok(ret)
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
struct IncludeStack {
    include_stack: Stack<PathBuf>,
}

use thiserror::Error;
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

fn tokenize_async_main_loop<P: AsRef<Path>>(
    ctx: &TokenizeContext,
    file_to_tokenize: P,
    parent: Option<PathBuf>,
    mut include_stack: IncludeStack,
) -> GResult<()> {
    use rayon::prelude::*;

    let file_to_tokenize = file_to_tokenize.as_ref().to_path_buf();

    info_mess!(
        "Checking if I have tokens for {}",
        file_to_tokenize.to_string_lossy()
    );

    let includes = ctx.with(|ctx| -> GResult<_> {
        let includes = if let Some(tokes) = ctx.get_tokens(&file_to_tokenize) {
            info_mess!("Had tokens for {}", file_to_tokenize.to_string_lossy());
            tokes.includes.clone()
        } else {
            info_mess!("Tokenizing {}", file_to_tokenize.to_string_lossy());
            tokenize_file_and_add_tokens(ctx, &file_to_tokenize, parent.clone())?
                .includes
                .clone()
        };

        Ok(includes)
    })?;
 
    info_mess!("Reading file {}", file_to_tokenize.to_string_lossy());

    include_stack.push(&file_to_tokenize).unwrap();

    rayon::scope(|s| -> GResult<()> {
        for (pos, full_path) in includes.iter() {
            if include_stack.is_circular(&full_path) {
                // TODO: Debug this
                let pe = ParseError::new_from_pos("Circular include", pos, true);
                return Err(GazmErrorKind::ParseError(pe.into()));
            } else {
                let include_stack = include_stack.clone();
                let included_from = file_to_tokenize.clone();

                s.spawn(move |_| {
                    let res = tokenize_async_main_loop(
                        ctx,
                        &full_path,
                        Some(included_from),
                        include_stack,
                    );

                    ctx.with(|ctx| {
                        let _ = ctx.asm_out.errors.add_result(res);
                    })
                });
            }
        }
        Ok(())
    })?;

    Ok(())
}

#[allow(unused_imports)]
mod test {
    use std::path;
    use std::{thread::current, time::Instant};

    use crate::config::YamlConfig;
    use crate::messages::Verbosity;

    use super::*;
    use async_std::task::block_on;
    use futures::future::join_all;
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

    // fn test_tokens( b: &mut Bencher) {
    //     use async_std::task;
    //     use std::env;

    //     let x = crate::messages::messages();
    //     x.set_verbosity(&Verbosity::Silent);

    //     let now = Instant::now();

    //     let config = get_config("/Users/garyliddon/development/stargate/gazm.toml");

    //     let ctx = mk_ctx(&config);
    //     let project_file = ctx.opts.project_file.clone();

    //     let ctx = Arc::new(Mutex::new(ctx));

    //     let _ = tokenize(&ctx, &project_file).unwrap();
    //     // let _ = crate::tokenize::tokenize(&ctx, &project_file).unwrap();

    //     let elapsed = now.elapsed();
    //     // println!("{:0.5?}", elapsed);
    // }
}
