use crate::ctx::{Context, Opts};
use std::path::{Path, PathBuf};

use crate::ast::Ast;
use crate::item::Node;
use crate::tokenize::tokenize ;

use utils::sources::Position;

use thiserror::Error;

use crate::error::UserError;
use crate::error::{GazmError, GResult };

pub struct Gazm<'a> {
    ctx: &'a mut Context,
    opts: Opts,
}

impl<'a> Gazm<'a> {
    pub fn new(ctx: &'a mut Context, opts: Opts) -> Self {
        Self { ctx, opts }
    }

    pub fn assemble_file<P: AsRef<Path>>(&mut self, x: P) -> GResult<()> {
        use utils::PathSearcher;
        let paths = self.ctx.source_file_loader.get_search_paths().clone();

        if let Some(dir) = x.as_ref().parent() {
            self.ctx.source_file_loader.add_search_path(dir);
        }

        let tokens = tokenize(&mut self.ctx, &self.opts, x)?;
        self.assemble_tokens(tokens)?;

        self.ctx.source_file_loader.set_search_paths(paths);
        self.ctx.errors.raise_errors()
    }

    fn assemble_tokens(&mut self, tokens: Node) -> GResult<()> {
        use crate::asmctx::AsmCtx;
        use crate::fixerupper::FixerUpper;
        use crate::evaluator::Evaluator;
        let tree = Ast::from_nodes(&mut self.ctx, tokens)?;

        use crate::sizer::size_tree;
        use crate::compile::compile;

        let id = tree.root().id();

        let mut asm_ctx = AsmCtx {
            fixer_upper: FixerUpper::new(),
            eval: Evaluator::new(&mut self.ctx.symbols, &mut self.ctx.source_file_loader),
            direct_page: None, 
            source_map: &mut self.ctx.source_map,
            binary: &mut self.ctx.binary,
            vars: &self.ctx.vars,
            errors: &mut self.ctx.errors,
            opts: &self.opts,
        };

        size_tree( &mut asm_ctx,id, &tree)?;
        compile(&mut asm_ctx, &tree)?;
        Ok(())
    }
}
