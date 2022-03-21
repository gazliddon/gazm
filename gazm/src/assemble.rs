use emu::cpu::RegEnum;

use crate::compile::Compiler;

use emu::isa::Instruction;
use serde::Serializer;
use utils::sources::SymbolQuery;
use utils::sources::{
    ItemType, Position, SourceDatabase, SourceInfo, SourceMapping, SymbolError, SymbolWriter,
};

use crate::ctx::Context;
use crate::ast::{ Ast, AstTree, AstNodeRef, AstNodeId };
use crate::binary::BinRef;
use crate::binary::{AccessType, Binary};
use crate::error::UserError;
use crate::eval::{EvalErrorEnum, eval};
use crate::item::AddrModeParseType;
use crate::item::IndexParseType;
use crate::messages::info;
use crate::messages::messages;
use crate::{item, messages};
use item::Item;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::vec;


use crate::gasm::{GResult, Gasm};

// use serde::{Deserialize, Serialize};
// use serde_json::json;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Assembled {
    #[serde(skip)]
    pub mem: Vec<u8>,
    pub database: SourceDatabase,
}

use std::collections::HashMap;
use crate::ctx::Opts;

pub struct Assembler<'a> {
    pub the_tree: AstTree,
    fixup: HashMap<(usize, AstNodeId), Item>,
    pub ctx: &'a mut Context,
    opts: Opts,
}

use crate::gasm::GasmError;

impl<'a> Assembler<'a> {

    pub fn new(ctx : &'a mut Context, opts: &Opts, ast: crate::ast::AstTree) -> GResult<Self> {
        // Needs symbols
        // Sources

        if let Some(file) = &opts.as6809_sym {
            crate::as6809::add_reference_syms(file, &mut ctx.symbols).map_err(|e| 
                GasmError::Misc( e.to_string()))?;
        }

        let ret = Self {
            the_tree: ast,
            ctx,
            fixup: Default::default(),
            opts: opts.clone(),
        };

        Ok(ret)
    }

    pub fn get_children(&self, id: AstNodeId) -> Vec<AstNodeId> {
        let node = self.get_node(id).unwrap();
        node.children().map(|n| n.id()).collect()
    }

    pub fn eval_node(&self, node: AstNodeRef) -> GResult<i64> {
        eval(&self.ctx.symbols, node).map_err(|err| {
            let info = self.get_source_info(&node.value().pos).unwrap();
            UserError::from_ast_error(err.into(), &info).into()
        })
    }

    pub fn user_error<S: Into<String>>(&self, err: S, id: AstNodeId, is_failure: bool) -> GasmError {
        let node = self.get_node(id).unwrap();
        let info = self.get_source_info(&node.value().pos).unwrap();
        UserError::from_text(err, &info, is_failure).into()
    }

    pub fn user_error_id<S: Into<String>>(
        &self,
        err: S,
        id: AstNodeId,
        is_failure: bool,
    ) -> UserError {
        let node = self.get_node(id).unwrap();
        let info = self.get_source_info(&node.value().pos).unwrap();
        UserError::from_text(err, &info, is_failure)
    }


    pub fn eval_first_arg(&self, id: AstNodeId) -> GResult<(i64, AstNodeId)> {
        let node = self.get_node(id).unwrap();
        let c = node
            .first_child()
            .ok_or_else(|| self.user_error("Missing argument", id, true))?;
        let v = self.eval_node(c)?;
        Ok((v, c.id()))
    }


    pub fn eval_childern(&self, id: AstNodeId) -> GResult<Vec<i64>> {
        let node = self.get_node(id).unwrap();
        let mut ret = vec![];

        for node in node.children() {
            let v = self.eval_node(node)?;
            ret.push(v)
        }

        Ok(ret)
    }
    pub fn eval_two_args(&self, id: AstNodeId) -> GResult<(i64, i64)> {
        let args = self.eval_n_args(id, 2)?;
        Ok((args[0], args[1]))
    }

    pub fn try_eval_n_args(&self, id: AstNodeId, n: usize) -> Vec<Option<i64>> {
        let node = self.get_node(id).unwrap();
        let mut ret = vec![];

        for (i, node) in node.children().enumerate() {
            if i == n {
                break;
            }
            let to_push = if let Ok(v) = self.eval_node(node) {
                Some(v)
            } else {
                None
            };
            ret.push(to_push)
        }

        ret
    }

    pub fn eval_n_args(&self, id: AstNodeId, n: usize) -> GResult<Vec<i64>> {
        let node = self.get_node(id).unwrap();
        let mut ret = vec![];

        for (i, node) in node.children().enumerate() {
            if i == n {
                break;
            }
            let v = self.eval_node(node)?;
            ret.push(v)
        }

        Ok(ret)
    }

    pub fn fixup_item(&mut self, pc: usize, node_id: AstNodeId, value: item::Item) {
        let k = (pc, node_id);
        self.fixup.insert(k, value);
    }

    pub fn compile(&'a mut self) -> GResult<()> {
        panic!()
        // let mut compiler = Compiler::new(self, self.opts.clone())?;
        // let _ = compiler.compile()?;
        // Ok(())
    }

    pub fn size(&'a mut self) -> GResult<()> {
        use crate::sizer::Sizer;
        // let mut sizer = Sizer::new(self);
        // sizer.size()
        Ok(())
    }

    pub fn eval_with_pc(&mut self, n: AstNodeId, pc: u64) -> GResult<i64> {
        self.ctx
            .symbols
            .add_symbol_with_value("*", pc as i64)
            .unwrap();
        let n = self.get_node(n).unwrap();
        let ret = self.eval_node(n)?;
        self.ctx.symbols.remove_symbol_name("*");
        Ok(ret)
    }

    pub fn set_scope(&mut self, scope: &str) {
        self.ctx.symbols.set_root();
        if scope != "root" {
            self.ctx.symbols.set_scope(scope);
        }
    }

    pub fn get_fixed_up_item(&self, pc: usize, id: AstNodeId) -> Item {
        let k = (pc, id);

        if let Some(ret) = self.fixup.get(&k) {
            ret.clone()
        } else {
            let this_node = self.get_node(id).unwrap();
            this_node.value().item.clone()
        }
    }

    pub fn eval_macro_args(&mut self, scope: &String, instance_id: AstNodeId, macro_id: AstNodeId) {
        use item::Item::*;

        self.ctx.symbols.set_scope(scope);

        let macro_node = self.get_node(macro_id).unwrap();

        let mut assignments = vec![];

        if let MacroDef(.., params) = &macro_node.value().item {
            let args = self.get_node(instance_id).unwrap().children();
            args.zip(params).for_each(|(arg, param)| {
                if !self.ctx.symbols.symbol_exists_from_name(param) {
                    let evaled = self.eval_node(arg);
                    if let Ok(value) = evaled {
                        assignments.push((param.clone(), value));
                    }
                }
            });
        }

        for (param, value) in assignments {
            self.ctx
                .symbols
                .add_symbol_with_value(&param, value)
                .unwrap();
        }

        self.ctx.symbols.pop_scope();
    }

    pub fn get_node(&self, id : AstNodeId) -> Option<AstNodeRef> {
        self.the_tree.get(id)
    }

    pub fn get_source_info_id(&self, id: AstNodeId) -> GResult<SourceInfo> {
        let pos = &self.get_node(id).unwrap().value().pos;
        self.ctx.sources().get_source_info(pos).map_err(|e| e.into())
    }

    pub fn get_source_info(&self, pos: &Position) -> Result<SourceInfo, String> {
        self.ctx.sources().get_source_info(pos)
    }
    pub fn get_root(&self) -> AstNodeRef {
        self.the_tree.root()
    }


   pub fn get_binary_extents(
        &self,
        file_name: PathBuf,
        id: AstNodeId,
    ) -> GResult<std::ops::Range<usize>> {
        let data_len = self
            .ctx
            .get_size(file_name)
            .map_err(|e| self.user_error(e.to_string(), id, true))?;

        let node = self.get_node(id).unwrap();

        let mut r = 0..data_len;

        let mut c = node.children();

        let offset_size = c
            .next()
            .and_then(|offset| c.next().map(|size| (offset, size)));

        if let Some((offset, size)) = offset_size {
            let offset = self.eval_node(offset)?;
            let size = self.eval_node(size)?;
            let offset = offset as usize;
            let size = size as usize;
            let last = (offset + size) - 1;

            if !(r.contains(&offset) && r.contains(&last)) {
                let msg =
                    format!("Trying to grab {offset:04X} {size:04X} from file size {data_len:X}");
                return Err(self.user_error(msg, id, true).into());
            };

            r.start = offset;
            r.end = offset + size;
        }

        Ok(r)
    }

    pub fn get_binary_chunk(
        &mut self,
        file_name: PathBuf,
        id: AstNodeId,
        range: std::ops::Range<usize>,
    ) -> GResult<Vec<u8>> {
        let (_, bin) = self
            .ctx
            .read_binary_chunk(file_name, range)
            .map_err(|e| self.user_error(e.to_string(), id, true))?;

        Ok(bin)
    }

    pub fn get_binary(&mut self, file_name: &Path, id: AstNodeId) -> GResult<Vec<u8>>{
        let range = self.get_binary_extents(file_name.to_path_buf(), id)?;
        self.get_binary_chunk(file_name.to_path_buf(), id, range)
    }

}



