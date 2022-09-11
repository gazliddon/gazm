use emu::utils::PathSearcher;

use crate::ctx::Context;
use crate::ctx::Opts;
use crate::error::{GResult, ParseError};
use crate::item::{Item, Node};
use crate::locate::{span_to_pos, Span};
use crate::tokenize::{tokenize_text, TokenizedText};
use async_std::prelude::*;
use emu::utils::sources::{AsmSource, SourceFileLoader};
use std::path::{Path, PathBuf};

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
    pub includes: Vec<(usize, PathBuf)>,
    pub path: Vec<usize>,
}

fn get_include_files(tokes: &TokenizedText) -> Vec<(usize, PathBuf)> {
    // Collect all of the include files
    tokes
        .tokens
        .iter()
        .enumerate()
        .filter_map(|(i, n)| n.item().get_include().map(|inc| (i, inc)))
        .collect()
}

fn tokenize_file<P: AsRef<Path>, PP: AsRef<Path>>(
    ctx: Arc<Mutex<Context>>,
    requested_file: P,
    parent: PP,
    path: Vec<usize>,
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
    let item = TokenizedFile(requested_file.clone(), parent.as_ref().into());
    let node = Node::from_item_pos(item, file_pos).with_children(tokens.tokens);

    let ret = TokenizeResult {
        requested_file,
        loaded_file: file,
        file_id,
        includes,
        node,
        errors: tokens.parse_errors,
        path,
    };

    Ok(ret)
}


pub fn tokenize<P: AsRef<Path>>(ctx: &Arc<Mutex<Context>>, main_file : P) -> GResult<Node> {
    use async_std::task;
    use async_std::task::block_on;
    use futures::future::join_all;

    let main_file = main_file.as_ref().to_path_buf();

    let mut files_to_tokenize: Vec<(Vec<usize>, PathBuf)> = vec![(vec![], main_file.clone())];
    let mut tokenized_files: HashMap<PathBuf, TokenizeResult> = Default::default();

    while !files_to_tokenize.is_empty() {
        let mut files_being_tokenized = vec![];

        for (path, inc) in files_to_tokenize.iter().cloned() {
            let ctx = ctx.clone();
            let path = path.clone();
            let tokenizing = task::spawn(async move {
                tokenize_file(ctx, inc, "", path).expect("Error should be handled!")
            });

            files_being_tokenized.push(tokenizing)
        }

        let join_results = block_on(join_all(files_being_tokenized));

        for result in join_results.iter() {
            tokenized_files.insert(result.requested_file.clone(), result.clone());
        }

        files_to_tokenize = join_results
            .into_iter()
            .map(|rez| {
                let ret = rez
                    .includes
                    .into_iter()
                    .map(|(_i, f)| {
                        let mut path = rez.path.clone();
                        path.push(_i);
                        (path, f)
                    })
                    .collect::<Vec<(Vec<usize>, PathBuf)>>();
                ret
            })
            .flatten()
            .collect();
    }

    let mut base_node = tokenized_files.get(&main_file).unwrap().node.clone();


    Ok(base_node)
}

fn modify_node(node: &mut Node, path: &[usize], val: Node) {

    if !path.is_empty() {
        let mut ret = node;

        for p in path.iter() {
            ret = &mut ret.children[*p]
        }

        println!("Was {:?}", ret.item().get_include());

        *ret = val;
        println!("is {:?}", ret.item());
    }
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
        let project_file = ctx.opts.project_file.clone();

        let ctx = Arc::new(Mutex::new(ctx));

        let _ = tokenize_ctx(ctx, &project_file).unwrap();

        let elapsed = now.elapsed();

        println!("{:0.5?}", elapsed);

        panic!("Done")
    }
}
