#![deny(unused_imports)]
use std::path::Path;
use std::fs;
use super::*;

pub fn test<P: AsRef<Path>>(file: P) {
    use grl_sources::{TextEditTrait, TextFile};
    let text = fs::read_to_string(&file).unwrap();
    let id = 0;
    let source_file = grl_sources::SourceFile::new(&file, &text,id);
    let tokens = to_tokens_kinds(&source_file);

    let tf = TextFile::new(&text);
    let mut prev_line = None;

    for (t, l) in tokens.iter() {
        let text_str = &text.as_str()[l.start..l.end];
        let tp = tf.offset_to_text_pos(l.start).unwrap();
        let line = tf.get_line(tp.line()).unwrap().trim();

        if Some(tp.line()) != prev_line {
            prev_line = Some(tp.line());
            println!("\n{line}");
        }

        print!("\t");

        if t == &TokenKind::Error {
            println!("ERROR: {} {:?}", tp.line() + 1, text_str);
        } else {
            println!("{:?} {:?}", t, text_str)
        }
    }
}


