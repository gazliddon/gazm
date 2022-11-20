use gazm::ctx::{Context, Opts};
use gazm::locate::Span;
use gazm::tokenize::tokenize_text;
use utils::sources::AsmSource;
use clap::{Arg, Command};

use gazm::item::{Item, Node};

use itertools::*;

fn main() {
    let m = parse();

    let mut opts = Opts::default();

    opts.star_comments = m.contains_id("star-comments");
    opts.trailing_comments = m.contains_id("trailing-comments");
    opts.encode_blank_lines = true;

    let file = m.get_one::<String>("file").unwrap();

    let src = std::fs::read_to_string(file).expect("Can't load file");

    let _ctx = Context::default();

    let text = Span::new_extra(&src, AsmSource::FromStr);

    let toks = tokenize_text(text, opts.clone()).expect("Can't tokenize");

    let formatted = render_nodes(toks.tokens, text);

    if m.contains_id("overwrite") {
        std::fs::write(file, formatted).expect("Unable to write file")
    } else {
        println!("{formatted}")
    }
}

pub fn parse() -> clap::ArgMatches {
    Command::new("gazfmt")
        .about("gazm source file reformat")
        .author("gazaxian")
        .version("0.1")
        .arg(
            Arg::new("file")
                // .multiple_values(false)
                .index(1)
                .required(true),
        )
        .arg(
            Arg::new("trailing-comments")
                .long("trailing-comments")
                .help("Text at end of line treated as a comment")
                .short('t'),
        )
        .arg(
            Arg::new("star-comments")
                .long("star-comments")
                .help("Lines that start with '*' parsed as comments")
                .short('q'),
        )
        .arg(
            Arg::new("overwrite")
                .long("overwrite")
                .help("Overwrite the original file")
                .short('o'),
        )
        .get_matches()
}

use std::fmt::Write;

pub fn render_nodes(nodes: Vec<Node>, text: Span) -> String {
    let mut ret = String::new();

    let t: String = text.to_string();

    let mut last_key = 0;

    for (key, group) in &nodes.iter().group_by(|e| e.ctx.line) {
        if key != last_key + 1 {
            writeln!(&mut ret,"").expect("can't write")
        }
        last_key = key;

        let mut columns = vec![String::new(); 4];
        use Item::*;

        let mut count = 0;

        for n in group {
            match n.item() {
                Comment(name) => {
                    if count == 0 {
                        columns[0] = format!("; {}", name)
                    } else {
                        columns[3] = format!("; {}", name)
                    }
                }

                Assignment(name) => {
                    columns[0] = name.to_string();
                    columns[1] = format!("equ {}", 
                        &t[n.children[0].ctx.range.clone()]);
                },

                StructDef(_name) => {
                    panic!()
                }

                LocalAssignmentFromPc(name) | AssignmentFromPc(name) => columns[0] = name.to_string(),
                Org => columns[1] = format!("{}", n),

                Rmb | Fcc(..) | MacroCall(..) | OpCode(..) | Fdb(..) | Fcb(..) => {
                    let txt = &t[n.ctx.range.clone()].to_string();
                    let txt = txt.replace("\t", " ");
                    columns[1] = format!("{}", txt);
                }

                BlankLine => (),

                _ => panic!("Can't format {:?}", n.item()),
            }
            count += 1;
        }

        writeln!(&mut ret,
            "{:<8}{:<16}{:<16}{:<16}",
            columns[0], columns[1], columns[2], columns[3]
        ).expect("Can't write");

    }
    ret
}
