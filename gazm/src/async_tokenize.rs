use emu::utils::PathSearcher;

use crate::asmctx::AsmCtx;
use crate::binary::AccessType;
use crate::ctx::Context;
use crate::ctx::Opts;
use crate::error::GazmError;
use crate::error::{GResult, ParseError};
use crate::item::{Item, Node};
use crate::locate::{span_to_pos, Span};
use crate::tokenize::{tokenize_text, TokenizedText};
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
}

use emu::utils::sources::Position;

fn tokenize_file<P: AsRef<Path>>(
    ctx: &TokenizeContext,
    requested_file: P,
    parent: Option<PathBuf>,
) -> GResult<TokenizeResult> {
    use Item::*;

    let requested_file = requested_file.as_ref().to_path_buf();

    let (file, source, file_id, opts) = ctx.with(|ctx| {
        ctx.read_source(&requested_file)
            .map(|(file, source, file_id)| (file, source, file_id, ctx.opts.clone()))
    })?;

    let input = Span::new_extra(&source, AsmSource::FileId(file_id));
    let tokens = tokenize_text(input, opts)?;

    // Collect all of the include files
    // let includes = get_include_files(&tokens);

    let file_pos = span_to_pos(input);
    let item = TokenizedFile(requested_file.clone(), parent.clone());
    let node = Node::from_item_pos(item, file_pos).with_children(tokens.tokens);

    let ret = TokenizeResult {
        requested_file,
        loaded_file: file,
        file_id,
        node,
        errors: tokens.parse_errors,
        parent,
    };

    Ok(ret)
}

#[derive(Default, Clone, Debug)]
pub struct TokenStore {
    tokens: HashMap<PathBuf, Node>,
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
        let ret = self.with(|ctx| -> GResult<PathBuf> {
            let inc_file = ctx.get_full_path(&file);
            inc_file
        });
        ret
    }

    /// Is this file in the path?
    /// Is this file already tokenized?
    pub fn get_file_info<P: AsRef<Path>>(&self, requested_file: P) -> GResult<(bool, PathBuf)> {
        let ret = self.with(|ctx| -> GResult<(bool, PathBuf)> {
            let actual_file = ctx.get_full_path(&requested_file)?;
            let has_this_file = ctx.token_store.has_tokens(&requested_file);
            Ok((has_this_file, actual_file))
        })?;

        Ok(ret)
    }
}

impl TokenStore {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn get_tokens<P: AsRef<Path>>(&self, file: P) -> Option<&Node> {
        self.tokens.get(&file.as_ref().to_path_buf())
    }

    pub fn add_tokens<P: AsRef<Path>>(&mut self, file: P, node: Node) {
        self.tokens.insert(file.as_ref().to_path_buf(), node);
    }

    pub fn has_tokens<P: AsRef<Path>>(&self, file: P) -> bool {
        self.get_tokens(file).is_some()
    }
}

pub fn tokenize<P: AsRef<Path>>(ctx: &Arc<Mutex<Context>>, requested_file: P) -> GResult<Node> {
    let include_stack = Stack::new();
    let token_ctx = TokenizeContext::new(&ctx );
    let file_name = tokenize_main(&token_ctx, &requested_file, None, include_stack)?;

    let ret = token_ctx.with(|ctx| -> GResult<Node> {
        ctx.errors.raise_errors()?;
        let toks = ctx.token_store.get_tokens(&file_name).unwrap().clone();
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

        Err(GazmError::ParseError(ParseError::new_from_pos(
            "Circular include".to_string(),
            &pos,
            true,
        )))
    } else {
        Ok(())
    }
}

fn tokenize_main<P: AsRef<Path>>(
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
        let mut tokenized = tokenize_file(ctx, &actual_file, parent)?;

        ctx.with(|ctx| -> GResult<()> {
            for e in &tokenized.errors {
                ctx.add_parse_error(e.clone())?;
            }
            Ok(())
        })?;

        let includes: Vec<Node> = tokenized
            .node
            .iter()
            .cloned()
            .filter(|n| n.item().get_include().is_some())
            .collect();

        // Did we find any includes?
        // If so spawn a task to tokenize each include file
        // in the scope of this file
        // Tokenize includes!
        
        rayon::scope(|s| -> GResult<()> {
            for (file, pos) in includes
                .into_iter()
                .map(|n| (n.item().get_include().unwrap(), n.ctx.clone()))
            {
                let full_path = ctx.get_full_path(&file)?;
                check_circular(&actual_file, &full_path, &include_stack, &pos)?;

                let mut stack = include_stack.clone();
                stack.push(full_path.clone());
                let actual_file = actual_file.clone();

                s.spawn(move |_| {
                    let res = tokenize_main(ctx, &full_path, Some(actual_file.clone()), stack);
                    ctx.with(|ctx| {
                        let _ = ctx.errors.add_result(res);
                    })
                });
            }
            Ok(())
        })?;

        // Now go through the tokens of this file
        // and replace any includes with the correct tokens from the token store
        // and then add the update tokens to the token store

        ctx.with(|ctx| -> GResult<()> {
            ctx.errors.raise_errors()?;


            let inc_nodes : Vec<_> = tokenized.node.children.iter_mut().filter_map(|c| {
                c.item().get_include().map(|f| {
                    let ret = (c, ctx.get_full_path(f).unwrap());
                    ret
                })
            }).collect();

            let ts = &ctx.token_store;

            for (c, file) in inc_nodes {
                *c = Box::new(ts.get_tokens(&file).unwrap().clone());
            }

            ctx.token_store.add_tokens(actual_file.clone(), tokenized.node);

            Ok(())
        })?;
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
        let config = YamlConfig::new_from_file(&path);
        config
    }

    fn mk_ctx(config: &YamlConfig) -> crate::ctx::Context {
        let mut dir = config.file.clone();
        dir.pop();
        let mut ctx = crate::ctx::Context::from(config.opts.clone());
        ctx.source_file_loader.add_search_path(&dir);
        ctx.source_file_loader
            .add_search_path(format!("{}/src", dir.to_string_lossy()));
        ctx
    }

    #[test]
    fn test_tokens() {
        use async_std::task;
        use std::env;

        let x = crate::messages::messages();
        x.set_verbosity(&Verbosity::Silent);

        let now = Instant::now();

        let config = get_config("/Users/garyliddon/development/stargate/gazm.toml");

        let ctx = mk_ctx(&config);
        let project_file = ctx.opts.project_file.clone();

        let ctx = Arc::new(Mutex::new(ctx));

        let _ = tokenize(&ctx, &project_file).unwrap();
        // let _ = crate::tokenize::tokenize(&ctx, &project_file).unwrap();

        let elapsed = now.elapsed();

        println!("{:0.5?}", elapsed);

        panic!("Done")
    }
}
