
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
}

impl Default for Binary {
    fn default() -> Self {
        Self {
            write_address: 0,
            written: false,
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

    pub fn write_byte(&mut self, _val : u8) {
        self.dirty();
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

    let indent = get_indent(depth);

    let assemble_msg = format!("Assembling.. {:?}", file_name.to_string_lossy());
    let succes_msg = format!("Assembly complete");

    println!("{}{}", indent,assemble_msg.green().bold());

    for n in &base_node.children {
        let i = n.item();

        match i {
            Org => bin.set_write_addr(get_value_n(0,n)),
            Assignment => (),
            Fdb => bin.bump_write_address(n.children.len()),
            OpCode(_,ins) => bin.bump_write_address(ins.size as usize),
            Fill => bin.fill(get_value_n(0,n),get_value_n(1,n) as u8),
            Comment(_) => (),
            TokenizedFile(file,_) => assemble_bin(&file, _ctx, bin,n, depth+1)?,
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
}

pub fn assemble(ctx : &cli::Context, base_node : Node) -> Result<Binary, UserError> {
    let mut bin = Binary::new();

    assemble_bin(&ctx.file, ctx, &mut bin, &base_node, 0)?;

    Ok(bin)
}

