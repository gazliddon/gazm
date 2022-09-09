use emu::utils::PathSearcher;

use crate::locate::{ Span, span_to_pos };
use crate::item::{ Item, Node };
use crate::error::{ GResult, ParseError };
use crate::ctx::Opts;
use crate::tokenize::{ tokenize_text,TokenizedText, get_include_files };
use std::path::{ Path,PathBuf };

use emu::utils::sources::AsmSource;

use async_std::prelude::*;

pub struct TokenizeResult {
    pub file: PathBuf,
    pub node : Node,
    pub errors: Vec<ParseError>,
    pub includes : Vec<PathBuf>,
}

use async_recursion::async_recursion;
#[async_recursion(?Send)]
async fn async_tokenize_file<P: AsRef<Path>, PP: AsRef<Path>>(
    depth: usize,
    ctx: &crate::ctx::Context,
    opts: &Opts,
    file: P,
    parent: PP,
) -> GResult<Node> {
    use anyhow::Context;
    use async_std::fs::File;
    use async_std::prelude::*;
    use futures::future::join_all;

    use Item::*;


    let this_file = file.as_ref().to_path_buf();

    let sl = ctx.get_source_file_loader();

    let file_name = sl
        .source_search_paths
        .get_full_path(&file)
        .expect("Couldn't find full path");

    let id = 0;

    let mut f = File::open(&file_name).await.expect("Can't load");

    let mut source = String::new();
    f.read_to_string(&mut source)
        .await
        .expect("can't read_source");

    let input = Span::new_extra(&source, AsmSource::FileId(id));

    let mut tokes = tokenize_text(input, opts)?;

    // Collect all of the include files
    let includes = get_include_files(&tokes);

    let mut fut = vec![];

    for (i, inc_file) in includes {

        let this_file = this_file.clone();

        fut.push(async move {
            let mut p = this_file.clone();
            p.pop();
            p.push(inc_file);
            let ret = async_tokenize_file(depth + 1, ctx, opts, p, this_file).await;
            (i.clone(),ret)
        });
    }

    // Wait for all of the includes to compete
    let ret = join_all(fut).await;

    // Change all of the include nodes to token streams
    for (i,n) in ret {
        tokes.tokens[i] = n.expect("Can't unwrap node");
    }

    // Prepend any errors into this node
    if !tokes.parse_errors.is_empty() {
        let error_node = Node::from_item_span(Errors(tokes.parse_errors), input);
        tokes.tokens.insert(0, error_node);
    }

    let file_pos = span_to_pos(input);
    let item = TokenizedFile(this_file, parent.as_ref().into(), Some(source));
    let node = Node::from_item_pos(item, file_pos).with_children(tokes.tokens);
    Ok(node)
}

#[allow(unused_imports)]
mod test {
    use std::{time::Instant, thread::current};
    use std::path;

    use crate::config::YamlConfig;
    use crate::messages::Verbosity;

    use super::*;
    #[allow(unused_imports)]
    use pretty_assertions::{assert_eq, assert_ne};
    use crate::tokenize::tokenize_file;

    #[test]
    fn test_tokens() {
        use std::env;
        use async_std::task;

        let x = crate::messages::messages();
        x.set_verbosity(&Verbosity::Silent);

        let path = path::PathBuf::from("/Users/garyliddon/development/stargate/gazm.toml");
        println!("Trying to read {}", path.to_string_lossy());

        let config = YamlConfig::new_from_file(&path);
        let mut dir = path.clone();
        dir.pop();

        let mut ctx = crate::ctx::Context::from(config.opts.clone());

        let opts = config.opts.clone();

        ctx.source_file_loader.add_search_path(&dir);
        ctx.source_file_loader.add_search_path(format!("{}/src", dir.to_string_lossy() ));


        let now = Instant::now();
        tokenize_file(0,&mut ctx,&opts,&opts.project_file, "" ).expect("Urgh");
        let elapsed_2 = now.elapsed();

        let now = Instant::now();
        let _ = task::block_on(
            async_tokenize_file(0, &ctx, &opts, &opts.project_file, "")
            );

        let elapsed_1 = now.elapsed();

        println!("Async:  {:.2?}", elapsed_1);
        println!("Normal: {:.2?}", elapsed_2);

        assert!(false);
    }
}
