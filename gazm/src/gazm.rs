use crate::ast::Ast;
use crate::async_tokenize;
use crate::ctx::{AsmOut, Context, Opts};
use crate::error::UserError;
use crate::error::{GResult, GazmErrorType};
use crate::item::Node;
use crate::tokenize::{self, TokenizedText};
use crate::locate::Span;
use emu::utils::sources::{EditResult, Position, SourceFile, TextEditTrait};
use rayon::iter::plumbing::Consumer;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use log::error;

pub struct Assembler {
    ctx: Arc<Mutex<Context>>,
}

impl Into<Context> for Assembler {
    fn into(self) -> Context {
        let lock = Arc::try_unwrap(self.ctx).expect("Still multiple owners");
        let ctx = lock.into_inner().expect("can't lock mutex");
        ctx
    }
}

impl Assembler {
    /// Create an Assembler
    pub fn new(opts: Opts) -> Self {
        let ctx = Context::from(opts.clone());
        let ctx = Arc::new(Mutex::new(ctx));
        Self { ctx }
    }


    /// Assemble for the first time
    pub fn assemble(&self) -> GResult<()> {
        self.with_inner(|ctx| ctx.reset_all());
        self.reassemble()
    }

    /// Reassemble the project keeping the same caches
    /// but clearing the assembly output
    pub fn reassemble(&self) -> GResult<()> {
        let file = self.with_inner(|ctx: &mut Context| -> GResult<PathBuf> {
            ctx.reset_output();
            Ok(ctx.opts.project_file.clone())
        })?;

        assemble_file(&self.ctx, file)

    }

    /// Operate on the inner Context
    pub fn with_inner<R>(&self, f: impl FnOnce(&mut Context) -> R) -> R {
        let ctx = &mut self.ctx.lock().expect("Could not lock mutex");
        f(ctx)
    }

    pub fn write_outputs(&self) -> GResult<()> {
        self.with_inner(|ctx| {
            ctx.write_ouputs()
        })
    }

    pub fn replace_file_contents<P: AsRef<Path>>(&self, file: P, new_text: &str) -> GResult<()> {
        self.with_inner(|ctx| -> GResult<()> {
            let res = ctx.edit_source_file(&file, |editable| editable.replace_file(new_text))?;
            Ok(res)
        })
    }

    /// Edit a file
    /// and invalidate the token cache
    pub fn edit_file<P: AsRef<Path>, X>(
        &self,
        file: P,
        f: impl FnOnce(&mut dyn TextEditTrait) -> EditResult<X>,
    ) -> GResult<X> {
        let x = self.with_inner(|ctx| -> GResult<X> {
            let res = ctx.edit_source_file(&file, |editable| f(editable))?;

            Ok(res)
        })?;

        Ok(x)
    }
}

pub fn with_state<R, S>(data: &Arc<Mutex<S>>, f: impl FnOnce(&mut S) -> R) -> R {
    let state = &mut data.lock().expect("Could not lock mutex");
    f(state)
}

pub fn create_ctx(opts: Opts) -> Arc<Mutex<Context>> {
    let ctx = Context::from(opts);
    let ctx = Arc::new(Mutex::new(ctx));
    ctx
}

pub fn reassemble_ctx(arc_ctx: &Arc<Mutex<Context>>) -> GResult<()> {
    let file = with_state(arc_ctx, |ctx| -> GResult<PathBuf> {
        ctx.reset_output();
        Ok(ctx.opts.project_file.clone())
    })?;

    assemble_file(arc_ctx, file)
}

fn assemble_file<P: AsRef<Path>>(arc_ctx: &Arc<Mutex<Context>>, file: P) -> GResult<()> {
    use emu::utils::PathSearcher;

    let ( paths, is_asnc ) = with_state(&arc_ctx, |ctx| {
        if let Some(dir) = file.as_ref().parent() {
            ctx.get_source_file_loader_mut().add_search_path(dir);
        }
        let paths = ctx.get_source_file_loader_mut().get_search_paths().clone();
        ( paths, ctx.opts.build_async )
    });

    let node = if is_asnc {
        async_tokenize::tokenize(&arc_ctx, file)

    } else {
        tokenize::tokenize(&arc_ctx,file, true)
    }?;

    assemble_tokens(arc_ctx, &node)?;

    with_state(&arc_ctx, |ctx| {
        ctx.asm_out.tokens.push(node);
        ctx.get_source_file_loader_mut().set_search_paths(paths);
        ctx.asm_out.errors.raise_errors()
    })
}

pub fn assemble_tokens(arc_ctx: &Arc<Mutex<Context>>, tokens: &Node) -> GResult<()> {
    use crate::asmctx::AsmCtx;
    use crate::compile::compile;
    use crate::evaluator::Evaluator;
    use crate::fixerupper::FixerUpper;
    use crate::lookup::LabelUsageAndDefintions;
    use crate::sizer::size_tree;

    with_state(&arc_ctx, |ctx| {
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
    })
}
