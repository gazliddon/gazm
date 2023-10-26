#![deny(unused_imports)]
use super::*;
use std::fs;
use std::path::Path;

pub fn test<P: AsRef<Path>>(file: P) {
    use TokenKind::*;
    let not_comment = |k: &TokenKind| k != &DocComment && k != &Comment;

    let text = fs::read_to_string(&file).unwrap();
    let sf = create_source_file(&text);
    let tokens = to_tokens_filter(&sf, not_comment);
    let span = make_tspan(&tokens, &sf);
    let _r = parse_span(span);

    println!("Parsed!");
}
