use crate::opts::Opts;
use crate::error::GResult;
use crate::item::Node;
use crate::item::{Item, LabelDefinition};
use crate::item6809::MC6809::{self,OpCode};

use itertools::Itertools;

pub fn fmt(_opts: &Opts) -> GResult<String> {
    // let asm = Assembler::new(opts.clone());
    // let tokenized = asm.tokenize_file(&opts.project_file)?;

    //     self.ctx.get_source_file_loader_mut()
    //         .read_source(&opts.project_file)?;

    // println!("{}", render_nodes(&tokenized.node.children, text));

    Ok("hello".to_string())
}

use std::fmt::Write;

fn get_operand(
    node: &Node,
    text: &str,
) ->  String {

    if node.children.is_empty() {
        "".to_owned()
    } else {
        let c = &node.children[0];
        text[c.ctx.range.clone()].to_owned()
    }
}
#[allow(dead_code)]
pub fn render_nodes(nodes: &[Node], text: String) -> String {
    let mut ret = String::new();

    let mut last_key = 0;

    for (key, group) in &nodes.iter().group_by(|e| e.ctx.line) {
        use Item::*;
        if key != last_key + 1 {
            writeln!(&mut ret).expect("can't write")
        }

        last_key = key;

        let mut columns = vec![String::new(); 4];

        let join_kids = |n: &Node| -> String {
            let args: Vec<_> = n
                .children
                .iter()
                .map(|c| &text[c.ctx.range.clone()])
                .collect();

            args.join(",")
        };

        for (count, n) in group.enumerate() {
            match &n.item {
                Label(LabelDefinition::Text(label)) => columns[0] = label.clone(),

                Comment(name) => {
                    if count == 0 {
                        columns[0] = format!("; {name}")
                    } else {
                        columns[3] = format!("; {name}")
                    }
                }

                MacroCall(name) => {
                    columns[1] = name.clone();
                    columns[2] = format!("({})", join_kids(n));
                }

                Assignment(name) => {
                    columns[0] = name.as_string();
                    columns[1] = format!("equ {}", &text[n.children[0].ctx.range.clone()]);
                }

                StructDef(_name) => {
                    panic!()
                }

                LocalAssignmentFromPc(LabelDefinition::Text(name))
                | AssignmentFromPc(LabelDefinition::Text(name)) => columns[0] = name.to_string(),

                Org => columns[1] = format!("{n}"),

                WriteBin(file) => {
                    columns[0] = "writebin".to_owned();
                    columns[1] = format!("\"{}\"", file.to_string_lossy())
                }

                Cpu( OpCode(txt,_ins, _addr_mode)  )=> {
                    let arg = get_operand(n, &text);
                    // let original_txt = text[n.ctx.range.clone()].to_string().replace('\t', " ");
                    columns[1] = txt.to_owned().to_lowercase();
                    columns[2] = arg;
                }

                Fill => {
                    // let original_txt = text[n.ctx.range.clone()].to_string().replace('\t', " ");
                    // columns[1] = original_txt
                }

                Fcc(text) => {
                    columns[1] = "fcc".to_owned();
                    columns[2] = text.clone();
                }

                Rmb => {
                    columns[1] = "rmb".to_owned();
                    columns[2] = join_kids(n);
                }

                Fdb(..) => {
                    columns[1] = "fdb".to_owned();
                    columns[2] = join_kids(n);
                }
                Fcb(..) => {
                    columns[1] = "fcb".to_owned();
                    columns[2] = join_kids(n);
                }

                Include(path) => {
                    columns[1] = "include".to_owned();
                    columns[2] = format!("\"{}\"", path.to_string_lossy())
                }

                Scope(scope) => {
                    columns[1] = "scope".to_owned();
                    columns[2] = scope.to_owned();
                }

                BlankLine => (),

                _ => println!("Can't format {:?}", &n.item),
            }
        }

        writeln!(
            &mut ret,
            "{:<16}{:<8}{:<16}{:<16}",
            columns[0], columns[1], columns[2], columns[3]
        )
        .expect("Can't write");
    }
    ret
}
