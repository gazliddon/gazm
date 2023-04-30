use crate::ast::Ast;
use crate::async_tokenize::{self, TokenizeResult};
use crate::ctx::{AsmOut, Context, Opts};
use crate::error::UserError;
use crate::error::{GResult, GazmErrorKind};
use crate::item::Node;
use crate::locate::Span;
use crate::tokenize;
use emu::utils::sources::{EditResult, Position, SourceFile, TextEditTrait};
use emu::utils::PathSearcher;
use rayon::iter::plumbing::Consumer;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use log::error;

pub struct Assembler {
    pub ctx: Context
}

impl From<Assembler> for Context {
    fn from(value: Assembler) -> Self {
        value.ctx
    }
}

impl Assembler {
    /// Create an Assembler
    pub fn new(opts: Opts) -> Self {
        let ctx = Context::from(opts);
        Self { ctx }
    }

    pub fn tokenize_file<P: AsRef<Path>>(&mut self, file: P) -> GResult<TokenizeResult> {
            let file = self.ctx.get_source_file_loader().get_full_path(file)?;
        tokenize::tokenize(&mut self.ctx, file)
    }

    /// Assemble for the first time
    pub fn assemble(&mut self) -> GResult<()> {
            self.ctx.reset_all();
            assemble_project(&mut self.ctx)
    }

    /// Reassemble the project keeping the same caches
    /// but clearing the assembly output
    pub fn reassemble(&mut self) -> GResult<()> {
            self.ctx.reset_output();
            assemble_project(&mut self.ctx)
    }


    pub fn write_outputs(&mut self) -> GResult<()> {
        self.ctx.write_ouputs()
    }

    pub fn replace_file_contents<P: AsRef<Path>>(&mut self, file: P, new_text: &str) -> GResult<()> {
            Ok(self.ctx.edit_source_file(&file, |editable| editable.replace_file(new_text))?)
    }

    /// Edit a file
    /// and invalidate the token cache
    pub fn edit_file<P: AsRef<Path>, X>(
        &mut self,
        file: P,
        f: impl FnOnce(&mut dyn TextEditTrait) -> EditResult<X>,
    ) -> GResult<X> {
            let r = self.ctx.edit_source_file(&file, |editable| f(editable))?;
        Ok(r)

    }
}

pub fn with_state<R, S>(data: &Arc<Mutex<S>>, f: impl FnOnce(&mut S) -> R) -> R {
    let state = &mut data.lock().expect("Could not lock mutex");
    f(state)
}

pub fn create_ctx(opts: Opts) -> Arc<Mutex<Context>> {
    let ctx = Context::from(opts);
    Arc::new(Mutex::new(ctx))
}

fn assemble_project(ctx: &mut Context) -> GResult<()> {
    use emu::utils::PathSearcher;

    let file = ctx.opts.project_file.to_owned();
    let paths = ctx.get_source_file_loader_mut().get_search_paths().to_vec();

    if let Some(dir) = file.parent() {
        ctx.get_source_file_loader_mut().add_search_path(dir);
    }

    let tokes = if ctx.opts.build_async {
        async_tokenize::tokenize(ctx, file)
    } else {
        tokenize::tokenize(ctx, file)
    }?;

    assemble_tokens(ctx, &tokes.node)?;
    ctx.get_source_file_loader_mut().set_search_paths(&paths);
    ctx.asm_out.errors.raise_errors()
}

pub fn assemble_tokens(ctx: &mut Context, tokens: &Node) -> GResult<()> {
    use crate::asmctx::AsmCtx;
    use crate::compile::compile;
    use crate::evaluator::Evaluator;
    use crate::fixerupper::FixerUpper;
    use crate::lookup::LabelUsageAndDefintions;
    use crate::sizer::size_tree;

    let tree = Ast::from_nodes(ctx, tokens)?;
    let lookup = LabelUsageAndDefintions::new(&tree);
    ctx.asm_out.lookup = Some(lookup);

    let id = tree.root().id();

    let mut asm_ctx = AsmCtx {
        fixer_upper: FixerUpper::new(),
        ctx,
    };

    size_tree(&mut asm_ctx, id, &tree)?;
    compile(&mut asm_ctx, &tree)?;
    ctx.asm_out.ast = Some(tree);
    Ok(())
}
