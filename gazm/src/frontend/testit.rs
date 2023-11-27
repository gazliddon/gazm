// #![deny(unused_imports)]

use grl_sources::{SourceFile, TextEditTrait};
use std::fs;

use crate::ast::{iter_ids_recursive, Ast};
use crate::error::{UserError, UserErrorData,ErrorMessage};
use crate::opts::Opts;

use super::{
    create_source_file, FeResult, FrontEndError, FrontEndErrorKind, Item, TokenizeRequest,
    TokenizeResult,
};

fn get_line(sf: &SourceFile, line: isize) -> String {
    if line < 0 {
        String::new()
    } else {
        let txt = sf.get_text().get_line(line as usize).unwrap_or("");
        format!("{txt}")
    }
}

fn get_lines(sf: &SourceFile, line: isize) -> String {
    [
        get_line(sf, line - 2),
        get_line(sf, line - 1),
        get_line(sf, line),
        get_line(sf, line + 1),
        get_line(sf, line + 2),
    ]
    .join("\n")
}

fn to_user_error(e: FrontEndError, sf: &SourceFile) -> UserError {
    use ErrorMessage::*;
    let message = if let FrontEndErrorKind::HelpText(ht) = e.kind {
        let short = crate::help::HELP.get_short(ht);
        let full_text = crate::help::HELP.get(ht);
        Markdown(format!("{short}"),format!("{full_text}"))
    } else {
        Plain(format!("{e}"))
    };

    let line = e.position.line();
    let line = get_line(sf, line as isize);

    let ued = UserErrorData {
        message,
        pos: e.position.clone(),
        line,
        file: sf.file.clone(),
        failure: true,
    };

    UserError{data: ued.into()}
}

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
