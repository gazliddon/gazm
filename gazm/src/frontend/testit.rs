// #![deny(unused_imports)]


use std::fs;
use crate::opts::Opts;
use crate::ast::{Ast, iter_ids_recursive };

use super::{ create_source_file, TokenizeRequest, TokenizeResult, FeResult, Item };

pub fn test(opts: &Opts) {
    let text = fs::read_to_string(&opts.project_file).unwrap();
    let mut sf = create_source_file(&text);
    sf.file = opts.project_file.clone();

    let req = TokenizeRequest::for_single_source_file(sf, opts);
    let res: FeResult<TokenizeResult> = req.try_into();

    match res {
        Ok(tres) => {
            let mut tree = Ast::from_node(&tres.node);
            // Strip any ycomments
            let it = iter_ids_recursive(tree.tree.root());
            tree.detach_nodes_filter(it, |n| matches!(n.value().item, Item::Comment(..)));
            println!("Parsed fine!");
            println!("{}", tree)
        }

        Err(e) => println!(
            "{e}\nFailed : line: {} col: {}",
            e.position.line() + 1,
            e.position.col() + 1
        ),
    }
}
