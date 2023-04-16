use emu::utils::PathSearcher;
use itertools::Itertools;

use crate::asmctx::AsmCtx;
use crate::binary::AccessType;
use crate::ctx::Context;
use crate::ctx::Opts;
use crate::error::GazmErrorType;
use crate::error::{GResult, ParseError};
use crate::info_mess;
use crate::item::{Item, Node};
use crate::locate::{span_to_pos, Span};
use crate::token_store::TokenStore;
use crate::tokenize::{from_text, TokenizedText};
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
    pub includes: Vec<PathBuf>,
}

use emu::utils::sources::Position;
/// Tokenize this file and add its tokens to the token store
/// return the includes this file had in it
pub fn tokenize_file_and_add_tokens<P: AsRef<Path>>(
    ctx: &mut Context,
    actual_file: P,
    parent: Option<PathBuf>,
) -> GResult<Vec<PathBuf>> {
    let tokenized = tokenize_file(ctx, &actual_file, parent)?;

    for e in &tokenized.errors {
        ctx.add_parse_error(e.clone())?;
    }

    let ts = ctx.get_token_store_mut();
    ts.add_tokens(&actual_file, tokenized.node);
    info_mess!("Added tokens for {}", actual_file.as_ref().to_string_lossy());
    Ok(ctx.get_untokenized_files(&tokenized.includes))
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

    let tokens = from_text(ctx, input)?;

    // Collect all of the include files
    // let includes = get_include_files(&tokens);

    let file_pos = span_to_pos(input);
    let item = TokenizedFile(requested_file.clone(), parent.clone());
    let node = Node::new_with_children(item, tokens.tokens, file_pos);

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
    // })
}

struct TokenizeContext {
    ctx: Arc<Mutex<Context>>,
    errors: Vec<ParseError>,
}

impl TokenizeContext {
    pub fn new(ctx: &Arc<Mutex<Context>>) -> Self {
        Self {
            ctx: ctx.clone(),
            errors: vec![],
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&mut Context) -> R) -> R {
        let mut ctx = self.ctx.lock().unwrap();
        f(&mut ctx)
    }

    pub fn get_opts(&self) -> Opts {
        self.with(|ctx| ctx.opts.clone())
    }

    pub fn get_full_path<P: AsRef<Path>>(&self, file: P) -> GResult<PathBuf> {
        self.with(|ctx| -> GResult<PathBuf> {
            ctx.get_full_path(&file)
        })
    }

    /// Is this file in the path?
    /// Is this file already tokenized?
    pub fn get_file_info<P: AsRef<Path>>(&self, requested_file: P) -> GResult<(bool, PathBuf)> {
        let ret = self.with(|ctx| -> GResult<(bool, PathBuf)> {
            let actual_file = ctx.get_full_path(&requested_file)?;
            let has_this_file = ctx.has_tokens(&requested_file);
            Ok((has_this_file, actual_file))
        })?;

        Ok(ret)
    }
}

pub fn tokenize<P: AsRef<Path>>(ctx: &Arc<Mutex<Context>>, requested_file: P) -> GResult<Node> {
    let include_stack = Stack::new();
    let token_ctx = TokenizeContext::new(ctx);
    let file_name = tokenize_async_main_loop(&token_ctx, &requested_file, None, include_stack)?;

    let ret = token_ctx.with(|ctx| -> GResult<Node> {
        ctx.asm_out.errors.raise_errors()?;
        let toks = ctx.get_tokens(&file_name).unwrap().clone();
        Ok(toks)
    })?;

    Ok(ret)
}

fn check_circular<P: AsRef<Path>>(
    from: P,
    full_path: P,
    include_stack: &Stack<PathBuf>,
    pos: &Position,
) -> GResult<()> {
    let full_path = full_path.as_ref().to_path_buf();
    let actual_file = from.as_ref().to_path_buf();

    if include_stack.get_deque().contains(&full_path) {
        println!("Trying to include {}", full_path.to_string_lossy());
        println!("from {}", actual_file.to_string_lossy());
        for (i, x) in include_stack.get_deque().iter().enumerate() {
            println!("{i} - {}", x.to_string_lossy());
        }

        let pe = ParseError::new_from_pos(
            "Circular include".to_string(),
            pos,
            true,
        );

        Err(GazmErrorType::ParseError(pe.into()))
    } else {
        Ok(())
    }
}
////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
struct IncludeStack {
    include_stack: Stack<PathBuf>,
}

pub enum IncludeErrors {
    CircularInclude,
    CantPop,
}

impl IncludeStack {
    pub fn new() -> Self {
        Self {
            include_stack: Default::default(),
        }
    }

    pub fn push<P: AsRef<Path>>(&mut self, p: P) -> Result<(), IncludeErrors> {
        let p = p.as_ref().to_path_buf();
        if self.include_stack.get_deque().contains(&p) {
            Err(IncludeErrors::CircularInclude)
        } else {
            self.include_stack.push(p);
            Ok(())
        }
    }

    pub fn pop<P: AsRef<Path>>(&mut self) -> Result<(), IncludeErrors> {
        if self.include_stack.is_empty() {
            Err(IncludeErrors::CantPop)
        } else {
            self.include_stack.pop();
            Ok(())
        }
    }

    fn is_circular<P: AsRef<Path>>(&self, full_path: P) -> bool {
        let full_path = full_path.as_ref().to_path_buf();
        self.include_stack.get_deque().contains(&full_path)
    }
}

////////////////////////////////////////////////////////////////////////////////

fn is_circular<P: AsRef<Path>>(full_path: P, include_stack: &Stack<PathBuf>) -> bool {
    let full_path = full_path.as_ref().to_path_buf();
    include_stack.get_deque().contains(&full_path)
}

fn tokenize_async_main_loop<P: AsRef<Path>>(
    ctx: &TokenizeContext,
    requested_file: P,
    parent: Option<PathBuf>,
    mut include_stack: Stack<PathBuf>,
) -> GResult<PathBuf> {
    use rayon::prelude::*;
    use std::sync::mpsc::sync_channel;
    use std::thread;
    // Find out if this file is already tokenized and held in the token store
    let (has_this_file, actual_file) = ctx.get_file_info(&requested_file)?;
    include_stack.push(actual_file.clone());

    // It isn't!
    if !has_this_file {
        info_mess!("Reading file {}", actual_file.to_string_lossy());
        let untokenized_includes = ctx.with(|ctx| -> GResult<Vec<PathBuf>> {
            tokenize_file_and_add_tokens(ctx, &actual_file, parent)
        })?;

        // let includes = ctx.with(|ctx| ctx.rationalise_includes(&tokenized.includes));

        rayon::scope(|s| -> GResult<()> {
            for file in untokenized_includes {
                let full_path = ctx.get_full_path(&file)?;

                if is_circular(&full_path, &include_stack) {
                    panic!()
                } else {
                    let mut stack = include_stack.clone();
                    stack.push(full_path.clone());
                    let actual_file = actual_file.clone();

                    s.spawn(move |_| {
                        let res = tokenize_async_main_loop(
                            ctx,
                            &full_path,
                            Some(actual_file.clone()),
                            stack,
                        );
                        ctx.with(|ctx| {
                            let _ = ctx.asm_out.errors.add_result(res);
                        })
                    });
                }
            }
            Ok(())
        })?;
    } else {
        info_mess!("File {} was already tokenized", actual_file.to_string_lossy())
    }

    Ok(actual_file)
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
