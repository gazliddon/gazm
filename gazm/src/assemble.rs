use emu::cpu::RegEnum;

use crate::asmctx::AsmCtx;
use crate::compile::Compiler;
use crate::evaluator::Evaluator;
use crate::fixerupper::FixerUpper;

use emu::isa::Instruction;
use serde::Serializer;
use utils::sources::{SymbolQuery, SymbolTree};
use utils::sources::{
    ItemType, Position, SourceDatabase, SourceInfo, SourceMapping, SymbolError, SymbolWriter, Sources,
};

use crate::ctx::Context;
use crate::ast::{ Ast, AstTree, AstNodeRef, AstNodeId };
use crate::binary::{self, BinRef};
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
//
//

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
    pub ctx: &'a mut Context,
    opts: Opts,
    binary: Binary,
    source_map : SourceMapping,
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
            opts: opts.clone(),
            binary : Binary::new(opts.memory_image_size, AccessType::ReadWrite),
            source_map: SourceMapping::new(),
        };

        Ok(ret)
    }

    pub fn assemble(&mut self) -> GResult<()>{
        use crate::sizer::Sizer;
        let id = self.the_tree.root().id();

        let mut ctx = AsmCtx {
            fixer_upper: FixerUpper::new(),
            eval: Evaluator::new(&mut self.ctx.symbols, &mut self.ctx.source_file_loader),
            direct_page: None, 
            source_map: &mut self.source_map,
            binary: &mut self.binary,
            vars: &self.ctx.vars,
        };

        let sizer =  Sizer::new(&self.the_tree);

        sizer.size(&mut ctx, id)?;

        let compiler = Compiler::new(self.opts.clone(), &self.the_tree)?;

        compiler.compile(&mut ctx)
    }

}



