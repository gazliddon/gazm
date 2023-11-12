#![deny(unused_imports)]
use super::*;
use crate::ast::{iter_ids_recursive, Ast};
use crate::async_tokenize::{TokenizeRequest, TokenizeResult};
use crate::error::{GResult, GazmErrorKind};
use crate::item::Item;
use crate::opts::Opts;
use std::fs;

pub fn test(opts: &Opts) {
    let text = fs::read_to_string(&opts.project_file).unwrap();
    let mut sf = create_source_file(&text);
    sf.file = opts.project_file.clone();

    let req = TokenizeRequest::for_single_source_file(sf, opts);
    let res: GResult<TokenizeResult> = req.try_into();

    match res {
        Ok(tres) => {
            let mut tree = Ast::from_node(&tres.node);
            // Strip any ycomments
            let it = iter_ids_recursive(tree.tree.root());
            tree.detach_nodes_filter(it, |n| matches!(n.value().item, Item::Comment(..)));
            println!("Parsed fine!");
            println!("{}", tree)
        }

        Err(GazmErrorKind::ParseError(e)) => println!(
            "Failed : line: {} col: {}",
            e.pos.line() + 1,
            e.pos.col() + 1
        ),

        Err(e) => {
            println!("Dunno:\n{e}")
        }
    }
}
