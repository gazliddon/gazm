use emu::utils::PathSearcher;

use crate::ctx::Opts;
use crate::error::{GResult, ParseError};
use crate::item::{Item, Node};
use crate::locate::{span_to_pos, Span};
use crate::tokenize::{get_include_files, tokenize_text, TokenizedText};
use std::path::{Path, PathBuf};

use emu::utils::sources::AsmSource;

use async_std::prelude::*;

#[derive(Debug, Clone)]
pub struct TokenizeResult {
    pub file: PathBuf,
    pub node: Node,
    pub errors: Vec<ParseError>,
    pub includes: Vec<(usize, PathBuf)>,
}

fn tokenize_file<P: AsRef<Path>, PP: AsRef<Path>>(
    ctx: &crate::ctx::Context,
    opts: &Opts,
    file: P,
    parent: PP,
) -> GResult<TokenizeResult> {
    use anyhow::Context;
    use std::fs::File;

    use std::thread;
    use Item::*;
    // println!("My id is {:?}", thread::current().id());

    let this_file = file.as_ref().to_path_buf();

    let sl = ctx.get_source_file_loader();

    let file_name = sl
        .source_search_paths
        .get_full_path(&file)
        .expect("Couldn't find full path");

    let id = 0;

    let source = {
        let source = std::fs::read_to_string(&file_name).expect("loade");
        source
    };

    let input = Span::new_extra(&source, AsmSource::FileId(id));

    let tokes = tokenize_text(input, opts)?;

    // Collect all of the include files
    let includes = get_include_files(&tokes);

    let file_pos = span_to_pos(input);
    let item = TokenizedFile(this_file, parent.as_ref().into(), Some(source));
    let node = Node::from_item_pos(item, file_pos).with_children(tokes.tokens);

    let ret = TokenizeResult {
        file: file_name,
        errors: tokes.parse_errors,
        includes,
        node,
    };

    Ok(ret)
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

    fn mk_ctx(config: &YamlConfig) -> (crate::ctx::Context, Opts) {
        let mut dir = config.file.clone();
        dir.pop();
        let mut ctx = crate::ctx::Context::from(config.opts.clone());
        ctx.source_file_loader.add_search_path(&dir);
        ctx.source_file_loader
            .add_search_path(format!("{}/src", dir.to_string_lossy()));
        (ctx, config.opts.clone())
    }

    #[test]
    fn test_tokens() {
        use async_std::task;
        use std::env;
        let x = crate::messages::messages();
        x.set_verbosity(&Verbosity::Silent);

        let now = Instant::now();

        let config = get_config("/Users/garyliddon/development/stargate/gazm.toml");

        let mut files_to_tokenize = vec![(0, config.opts.project_file.clone())];

        let mut tokenized_files : Vec<TokenizeResult> = vec![];

        while !files_to_tokenize.is_empty() {

            let mut files_being_tokenized = vec![];

            for (_id, inc) in files_to_tokenize.iter().cloned() {
                let (ctx, opts) = mk_ctx(&config);
                files_being_tokenized.push(task::spawn(async move {
                    tokenize_file(&ctx, &opts, inc, "").expect("kjkj")
                }))
            }

            let join_results = block_on(join_all(files_being_tokenized));

            tokenized_files.extend(join_results.iter().cloned());
            files_to_tokenize = join_results.into_iter().flat_map(|x| x.includes).collect();
        }

        println!("{:0.5?}", now.elapsed());

        for x in tokenized_files {
            println!("{}", x.file.to_string_lossy())

        }

        panic!("Done")
    }
}
