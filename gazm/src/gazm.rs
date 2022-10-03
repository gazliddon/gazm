use crate::async_tokenize;
use crate::ctx::{Context, Opts};
use std::path::{Path, PathBuf};
use crate::ast::Ast;
use crate::item::Node;
use crate::tokenize;
use emu::utils::sources::Position;
use rayon::iter::plumbing::Consumer;
use thiserror::Error;
use crate::error::UserError;
use crate::error::{GResult, GazmErrorType};
use std::sync::{Arc, Mutex};

pub fn with_state<R, S>(data: &Arc<Mutex<S>>, f: impl FnOnce(&mut S) -> R) -> R {
    let state = &mut data.lock().expect("Could not lock mutex");
    f(state)
}

pub fn create_ctx(opts: Opts) -> Arc<Mutex<Context>> { 
    let ctx = Context::from(opts);
    let ctx = Arc::new(Mutex::new(ctx));
    ctx
}

fn assemble_arc(opts: Opts) -> GResult<Arc<Mutex<Context>>> {
    let file = opts.project_file.clone();
    let ctx = create_ctx(opts);
    assemble_file(&ctx, file)?;
    Ok(ctx)
}

pub fn assemble_from_opts(opts: Opts) -> GResult<Context> {
    let ctx_arc = assemble_arc(opts)?;
    let lock = Arc::try_unwrap(ctx_arc).expect("Still multiple owners");
    let ctx = lock.into_inner().expect("can't lock mutex");
    Ok(ctx)
}

fn assemble_file<P: AsRef<Path>>(arc_ctx: &Arc<Mutex<Context>>, file: P) -> GResult<()> {
    use emu::utils::PathSearcher;

    let (is_async, paths) = with_state(&arc_ctx, |ctx| {
        if let Some(dir) = file.as_ref().parent() {
            ctx.source_file_loader.add_search_path(dir);
        }
        let paths = ctx.source_file_loader.get_search_paths().clone();
        (ctx.opts.build_async, paths)
    });

    let node = if is_async {
        async_tokenize::tokenize(&arc_ctx, file)?
    } else {
        tokenize::tokenize(&arc_ctx, file)?
    };

    assemble_tokens(arc_ctx, &node)?;

    with_state(&arc_ctx, |ctx| {
        ctx.tokens.push(node);
        ctx.source_file_loader.set_search_paths(paths);
        ctx.errors.raise_errors()
    })
}

pub fn assemble_tokens(arc_ctx: &Arc<Mutex<Context>>, tokens: &Node) -> GResult<()> {
    use crate::asmctx::AsmCtx;
    use crate::compile::compile;
    use crate::evaluator::Evaluator;
    use crate::fixerupper::FixerUpper;
    use crate::sizer::size_tree;

    with_state(&arc_ctx, |ctx| {
        let tree = Ast::from_nodes(ctx, tokens)?;

        let id = tree.root().id();

        let mut asm_ctx = AsmCtx {
            fixer_upper: FixerUpper::new(),
            eval: Evaluator::new(&mut ctx.symbols, &mut ctx.source_file_loader),
            direct_page: None,
            source_map: &mut ctx.source_map,
            binary: &mut ctx.binary,
            errors: &mut ctx.errors,
            opts: &ctx.opts,
            lst_file: &mut ctx.lst_file,
            exec_addr: &mut ctx.exec_addr,
            bin_to_write_chunks: &mut ctx.bin_to_write_chunks,
        };

        size_tree(&mut asm_ctx, id, &tree)?;

        compile(&mut asm_ctx, &tree)?;
        ctx.ast = Some(tree);

        Ok(())
    })
}

