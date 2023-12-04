// #![deny(unused_imports)]

use grl_sources::{SourceFile, TextEditTrait};
use std::fs;

use crate::ast::{iter_ids_recursive, Ast};
use crate::error::{ErrorMessage, UserError, UserErrorData};
use crate::opts::Opts;

use super::{
    create_source_file, error::to_user_error, FeResult, FrontEndError, FrontEndErrorKind, Item,
    TokenizeRequest, TokenizeResult,
};

pub fn test_it(opts: &Opts) {
    let text = fs::read_to_string(&opts.project_file).unwrap();
    let mut sf = create_source_file(&text);
    sf.file = opts.project_file.clone();
    let req = TokenizeRequest::for_single_source_file(sf.clone(), opts);
    let tokes: TokenizeResult = req.to_result();

    if tokes.errors.is_empty() {
        println!("Tokenized fine!")
    } else {
        for e in tokes.errors {
            let err = to_user_error(e, &sf);
                err.as_ref().print_pretty()
        }
    }
}
