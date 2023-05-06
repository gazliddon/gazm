use crate::ast::Ast;
use crate::async_tokenize;
use crate::ctx::{Context, Opts};
use crate::error::GResult;
use crate::item::Node;

use emu::utils::sources::{EditResult,  TextEditTrait};
use emu::utils::{PathSearcher, Stack};

use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct ScopeTracker {
    stack: Stack<u64>,
}

impl ScopeTracker {
    pub fn new(scope_id : u64) -> Self {
        let mut ret = Self {
            ..Default::default()
        };
        ret.stack.push(scope_id);
        ret
    }

    pub fn scope(&self) -> u64 {
        *self.stack.front().unwrap()
    }
    pub fn set_scope(&mut self, scope_id: u64) {
        let r = self.stack.front_mut().unwrap();
        *r = scope_id;
    }

    pub fn push(&mut self, scope:u64) {
        self.stack.push(scope)
    }

    pub fn pop(&mut self) -> u64 {
        self.stack.pop().unwrap().clone()
    }
}

pub struct Assembler {
    pub ctx: Context,
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

    pub fn replace_file_contents<P: AsRef<Path>>(
        &mut self,
        file: P,
        new_text: &str,
    ) -> GResult<()> {
        Ok(self
            .ctx
            .edit_source_file(&file, |editable| editable.replace_file(new_text))?)
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

fn assemble_project(ctx: &mut Context) -> GResult<()> {
    let file = ctx.opts.project_file.to_owned();
    let paths = ctx.get_source_file_loader_mut().get_search_paths().to_vec();

    if let Some(dir) = file.parent() {
        ctx.get_source_file_loader_mut().add_search_path(dir);
    }

    let tokes = {
        if ctx.opts.no_async {
            async_tokenize::tokenize_no_async(ctx)?;
        } else {
            async_tokenize::tokenize_async(ctx)?;
        }

        let file = ctx.get_project_file();
        ctx.get_tokens_from_full_path(&file).unwrap().clone()
    };

    assemble_tokens(ctx, &tokes.node)?;

    ctx.get_source_file_loader_mut().set_search_paths(&paths);

    ctx.asm_out.errors.raise_errors()
}

pub fn assemble_tokens(ctx: &mut Context, tokens: &Node) -> GResult<()> {
    use crate::asmctx::AsmCtx;
    use crate::compile::compile;
    use crate::fixerupper::FixerUpper;
    use crate::lookup::LabelUsageAndDefintions;
    use crate::sizer::size_tree;

    let tree = Ast::from_nodes(ctx, tokens)?;

    let mut asm_ctx = AsmCtx {
        fixer_upper: FixerUpper::new(),
        ctx,
    };

    size_tree(&mut asm_ctx, &tree)?;
    compile(&mut asm_ctx, &tree)?;

    let lookup = LabelUsageAndDefintions::new(&tree, &ctx.asm_out.symbols);
    ctx.asm_out.ast = Some(tree);
    ctx.asm_out.lookup = Some(lookup);

    Ok(())
}
