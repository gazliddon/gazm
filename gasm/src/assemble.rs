
use colored::*;

use std::path::PathBuf;
use crate::cli::Context;
use crate::error::UserError;
use crate::item;
use crate::cli;
use item::{ Node, Item };
use romloader::ResultExt;

pub struct Binary {
    write_address : usize,
    written: bool,
    range : Option<(usize, usize)>,
}

impl Default for Binary {
    fn default() -> Self {
        Self {
            write_address: 0,
            written: false,
            range: None,
        }
    }
}

impl Binary {
    fn dirty(&mut self) {
        self.written = true;
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn bump_write_address(&mut self, n : usize) {
        self.write_address += n;
    }

    pub fn get_write_address(&self) -> usize {
        self.write_address
    }

    pub fn set_write_address(&mut self, pc : usize) {
        self.write_address = pc
    }

    pub fn set_write_addr(&mut self, pc : usize) {
        self.write_address = pc;
    }

    pub fn get_range(&self) -> Option<(usize,usize)> {
        self.range
    }

    pub fn write_byte(&mut self, _val : u8) {
        let pc = self.write_address;

        if let Some((mut low, mut high)) = self.range {
            if pc < low {
                low = pc
            }

            if pc > high {
                high = pc
            }

            self.range = Some(( low, high ))

        } else {
            self.range = Some((pc, pc))
        }

        self.write_address += 1;
    }
    pub fn fill(&mut self, count : usize, byte : u8) {
        for _i in 0..count {
            self.write_byte(byte)
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.write_byte(*b)
        }
    }

    pub fn write_word(&mut self, val : u16) {
        self.write_byte(( val&0xff ) as u8);
        self.write_byte(( val>>8 ) as u8);
    }
}

fn get_value(node : &Node) -> usize {
    use Item::*;

    match node.item() {
        Number(n) => *n as usize,
        _ => 0,
    }
} 

fn get_value_n(n : usize, node : &Node ) -> usize {
    get_value(&node.children[n])
}

fn get_indent(depth: usize) -> String {
    let indent = 2 + depth * 2;
    let indent = " ".repeat(indent);
    indent
}

pub fn assemble_bin(file_name : &PathBuf,_ctx : &cli::Context, bin : &mut Binary, base_node : &Node, depth: usize) -> Result<(), UserError>{
    use Item::*;

    if let TokenizedFile(_path, _parent, _source) = base_node.item() {

        let indent = get_indent(depth);

        let assemble_msg = format!("Assembling.. {:?}", file_name.to_string_lossy());
        let succes_msg = format!("Assembly complete");

        println!("{}{}", indent,assemble_msg.green().bold());

        for n in &base_node.children {
            let i = n.item();

            match i {
                Org => {
                    let pc = get_value_n(0,n);
                    bin.set_write_addr(pc)
                }

                Assignment => {
                    // let label = &n.children[0];
                    // let value = &n.children[1];
                }

                Fdb => {
                    for i in n.children.iter() {
                        bin.write_word(get_value(i) as u16)
                    }
                }

                OpCode(_,ins) => {
                    let next = bin.get_write_address()+ins.size as usize;
                    if ins.opcode > 0xff {
                        bin.write_word(ins.opcode);
                    } else {
                        bin.write_byte(ins.opcode as u8);
                    }
                    // TODO WRITE OPERAND
                    bin.set_write_addr(next);
                },

                Fill => {
                    let value = get_value_n(0,n) as u8;
                    let count = get_value_n(1,n);
                    bin.fill(count, value);
                }

                Zmb => bin.fill(get_value_n(0,n),0),
                Zmd => bin.fill(get_value_n(0,n) * 2,0),

                Comment(_) => (),

                TokenizedFile(file,_, _) => assemble_bin(&file, _ctx, bin,n, depth+1)?,
                _ => {
                    let msg = format!("error: unknown item {:?}", i).red().bold();
                    println!("{}{}",get_indent(depth+1), msg);
                }
            }
        }

        if depth == 0 {
            println!("{}{}", indent, succes_msg.green().bold());
        }

        Ok(())
    } else {
        panic!()
    }

}

pub fn assemble(ctx : &cli::Context, base_node : &Node) -> Result<Binary, UserError> {
    let mut bin = Binary::new();

    assemble_bin(&ctx.file, ctx, &mut bin, &base_node, 0)?;

    println!("range {:04x?}", bin.get_range());

    Ok(bin)
}

