use crate::async_tokenize;
use crate::ctx::{Context, Opts};
use std::path::{Path, PathBuf};

use crate::ast::Ast;
use crate::item::Node;
use crate::tokenize::tokenize;

use emu::utils::sources::Position;

use thiserror::Error;

use crate::error::UserError;
use crate::error::{GResult, GazmError};
use std::sync::{Arc, Mutex};

pub struct Gazm {
    ctx: Arc<Mutex<Context>>,
}

impl Gazm {
    pub fn new(ctx: Arc<Mutex<Context>>) -> Self {
        Self { ctx }
    }

    pub fn assemble_file<P: AsRef<Path>>(&mut self, file: P) -> GResult<()> {
        use emu::utils::PathSearcher;
        use super::async_tokenize::tokenize_ctx;

        let (is_async, paths) = {
            let mut ctx = self.ctx.lock().unwrap();

            if let Some(dir) = file.as_ref().parent() {
                ctx.source_file_loader.add_search_path(dir);
            }

            (
                ctx.opts.async_build,
                ctx.source_file_loader.get_search_paths().clone(),
            )
        };

        let arc_ctx = self.ctx.clone();

        let node = if is_async {
            tokenize_ctx(arc_ctx, file)?
        } else {
            tokenize(arc_ctx, file)?
        };

        self.assemble_tokens(&node)?;

        let mut ctx = self.ctx.lock().unwrap();

        ctx.tokens.push(node);
        ctx.source_file_loader.set_search_paths(paths);
        ctx.errors.raise_errors()
    }

    fn assemble_tokens(&mut self, tokens: &Node) -> GResult<()> {
        use crate::asmctx::AsmCtx;
        use crate::compile::compile;
        use crate::evaluator::Evaluator;
        use crate::fixerupper::FixerUpper;
        use crate::sizer::size_tree;
        use std::ops::DerefMut;

        let mut ctx_guard = self.ctx.lock().unwrap();
        let mut ctx = ctx_guard.deref_mut();

        let tree = Ast::from_nodes(&mut ctx, tokens)?;

        let id = tree.root().id();

        let mut asm_ctx = AsmCtx {
            fixer_upper: FixerUpper::new(),
            eval: Evaluator::new(&mut ctx.symbols, &mut ctx.source_file_loader),
            direct_page: None,
            source_map: &mut ctx.source_map,
            binary: &mut ctx.binary,
            vars: &ctx.vars,
            errors: &mut ctx.errors,
            opts: &ctx.opts,
            lst_file: &mut ctx.lst_file,
            bin_chunks: &mut ctx.bin_chunks,
            exec_addr: &mut ctx.exec_addr,
        };

        size_tree(&mut asm_ctx, id, &tree)?;

        compile(&mut asm_ctx, &tree)?;
        ctx.ast = Some(tree);

        Ok(())
    }
}
