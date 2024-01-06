#![deny(unused_imports)]

use crate::{error::{to_user_error, ErrorCollectorTrait}, opts::Opts};
use std::fs;

use super::{create_source_file, TokenizeRequest, TokenizeResult};

pub fn test_it(opts: &Opts) {
    let text = fs::read_to_string(&opts.project_file).unwrap();
    let mut sf = create_source_file(&text);
    sf.file = opts.project_file.clone();
    let req = TokenizeRequest::for_single_source_file(sf.clone(), opts);
    let tokes: TokenizeResult = req.to_result();

    println!("{:?}", tokes.errors);

    if tokes.errors.has_errors() {
        for e in tokes.errors.to_vec() {
            let err = to_user_error(e, &sf);
            err.as_ref().print_pretty(true)
        }
    } else {
        println!("Tokenized fine!")
    }
}
