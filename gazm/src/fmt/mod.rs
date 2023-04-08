
use crate::ast::make_tree;
use crate::ctx::Opts;
use crate::gazm::Assembler;
use crate::item::Item;
use crate::item::Node;
use crate::locate::Span;
// use lazy_static::__Deref;

use itertools::Itertools;

pub struct Format {}

pub fn fmt(_opts: &Opts) {
    // let asm = Assembler::new(opts.clone());
    // let (tok_text,source) = asm.tokenize_single_file().unwrap();
    // let ret = render_nodes(tok_text.tokens, source);
    // println!("{}", ret)
}

use std::fmt::Write;
pub fn render_nodes(nodes: Vec<Node>, text: String) -> String {
    let mut ret = String::new();

    let mut last_key = 0;

    for (key, group) in &nodes.iter().group_by(|e| e.ctx.line) {

        if key != last_key + 1 {
            writeln!(&mut ret, "").expect("can't write")
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
                    columns[1] = format!("equ {}", &text[n.children[0].ctx.range.clone()]);
                }

                StructDef(_name) => {
                    panic!()
                }

                LocalAssignmentFromPc(name) | AssignmentFromPc(name) => {
                    columns[0] = name.to_string()
                }

                Org => columns[1] = format!("{}", n),

                WriteBin(file) => { columns[0] ="writebin".to_owned();
                    columns[1] = format!("\"{}\"", file.to_string_lossy())
                },

                Rmb | Fcc(..) | MacroCall(..) | OpCode(..) | Fdb(..) | Fcb(..) | Fill => {
                    let txt = &text[n.ctx.range.clone()].to_string();
                    let txt = txt.replace("\t", " ");
                    columns[1] = format!("{}", txt);
                }

                BlankLine => (),

                _ => println!("Can't format {:?}", n.item()),
            }
            count += 1;
        }

        writeln!(
            &mut ret,
            "{:<8}{:<16}{:<16}{:<16}",
            columns[0], columns[1], columns[2], columns[3]
        )
        .expect("Can't write");
    }
    ret
}
