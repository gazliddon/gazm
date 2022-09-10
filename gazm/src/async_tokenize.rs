use emu::utils::PathSearcher;

use crate::ctx::Opts;
use crate::error::{GResult, ParseError};
use crate::item::{Item, Node};
use crate::locate::{span_to_pos, Span};
use crate::tokenize::{tokenize_text, TokenizedText};
use std::path::{Path, PathBuf};
use crate::ctx::Context;
use emu::utils::sources::{AsmSource, SourceFileLoader};
use async_std::prelude::*;

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
////////////////////////////////////////////////////////////////////////////////
///
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct TokenizeResult {
    pub file_id: u64,
    pub file: PathBuf,
    pub node: Node,
    pub errors: Vec<ParseError>,
    pub includes: Vec<PathBuf>,
}

fn get_include_files(tokes: &TokenizedText) -> Vec<PathBuf> {
    // Collect all of the include files
    tokes
        .tokens
        .iter()
        .filter_map(|n| {
            n.item().get_include()
        })
        .collect()
}

fn tokenize_file<P: AsRef<Path>, PP: AsRef<Path>>(
    ctx: Arc<Mutex<Context>>,
    requested_file: P,
    parent: PP,
) -> GResult<TokenizeResult> {
    use anyhow::Context;
    use std::fs::File;

    use std::thread;
    use Item::*;

    let requested_file = requested_file.as_ref().to_path_buf();

    let (file, source, file_id, opts) = {
        let mut ctx = ctx.lock().unwrap();
        ctx.read_source(&requested_file)
            .map(|(file_name, source, id)| (file_name, source, id, ctx.opts.clone()))
    }?;

    let input = Span::new_extra(&source, AsmSource::FileId(file_id));
    let tokens = tokenize_text(input, &opts)?;

    // Collect all of the include files
    let includes = get_include_files(&tokens);

    let file_pos = span_to_pos(input);
    let item = TokenizedFile(requested_file, parent.as_ref().into());
    let node = Node::from_item_pos(item, file_pos).with_children(tokens.tokens);

    let ret = TokenizeResult {
        file,
        file_id,
        includes,
        node,
        errors: tokens.parse_errors,
    };

    Ok(ret)
}

pub fn tokenize_ctx(ctx: Arc<Mutex<Context>>) -> GResult<()> {
    // TODO
    // Get parent correct
    
    use async_std::task;
    use async_std::task::block_on;
    use futures::future::join_all;

    let file = ctx.lock().unwrap().opts.project_file.clone();

    let mut files_to_tokenize = vec![file];

    let mut tokenized_files: Vec<TokenizeResult> = vec![];

    while !files_to_tokenize.is_empty() {
        let mut files_being_tokenized = vec![];

        for inc in files_to_tokenize.iter().cloned() {
            let ctx = ctx.clone();
            let tokenizing = task::spawn(async move {
                tokenize_file(ctx, inc, "").expect("Error should be handled!")
            });

            files_being_tokenized.push(tokenizing)
        }

        let join_results = block_on(join_all(files_being_tokenized));
        tokenized_files.extend(join_results.iter().cloned());
        files_to_tokenize = join_results.into_iter().flat_map(|x| x.includes).collect();
    }

    Ok(())
}

use std::collections::HashMap;

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

        let ctx = Arc::new(Mutex::new(ctx));

        tokenize_ctx(ctx).unwrap();

        println!("{:0.5?}", now.elapsed());

        panic!("Done")
    }
}
