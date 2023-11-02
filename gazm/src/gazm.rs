// #![forbid(unused_imports)]
use crate::{
    asmctx::AsmCtx, ast::AstCtx, async_tokenize, compile::compile, ctx::Assembler, error::GResult,
    fixerupper::FixerUpper, item::Node, lookup::LabelUsageAndDefintions, opts::Opts,
    sizer::size_tree,
};

use grl_sources::{
    grl_utils::{PathSearcher, Stack},
    EditResult, TextEditTrait,
};

use std::{
    path::Path,
    sync::{Arc, Mutex},
};


impl Assembler {
    /// Create an Assembler
    pub fn new(opts: Opts) -> Self {
        let ctx = Assembler::try_from(opts).expect("Can't create context");
        ctx
    }

    /// Assemble for the first time
    pub fn assemble(&mut self) -> GResult<()> {
        self.reset_all();
        self.assemble_project()
    }

    /// Reassemble the project keeping the same caches
    /// but clearing the assembly output
    pub fn reassemble(&mut self) -> GResult<()> {
        self.reset_output();
        self.assemble_project()
    }

    pub fn write_outputs(&mut self) -> GResult<()> {
        self.write_ouputs()
    }


    fn assemble_project(&mut self) -> GResult<()> {
        let file = self.opts.project_file.to_owned();
        let paths = self
            .get_source_file_loader_mut()
            .get_search_paths()
            .to_vec();

        if let Some(dir) = file.parent() {
            self.get_source_file_loader_mut().add_search_path(dir);
        }

        let tokes = {
            if self.opts.no_async {
                async_tokenize::tokenize_no_async(self)?;
            } else {
                async_tokenize::tokenize_async(self)?;
            }

            let file = self.get_project_file();
            self.get_tokens_from_full_path(&file).unwrap().clone()
        };

        self.assemble_tokens(&tokes.node)?;

        self.get_source_file_loader_mut().set_search_paths(&paths);
        self.asm_out.errors.raise_errors()
    }

    fn assemble_tokens(&mut self, tokens: &Node) -> GResult<()> {
        let tree = AstCtx::from_nodes(self, tokens)?;
        let docs = tree.docs;
        let tree = tree.ast_tree;

        let mut asm_ctx = AsmCtx {
            fixer_upper: FixerUpper::new(),
            ctx: self,
        };

        size_tree(&mut asm_ctx, &tree)?;
        compile(&mut asm_ctx, &tree)?;

        let lookup = LabelUsageAndDefintions::new(&tree, &self.asm_out.symbols, docs);
        self.asm_out.ast = Some(tree);
        self.asm_out.lookup = Some(lookup);
        Ok(())
    }
}

pub fn with_state<R, S>(data: &Arc<Mutex<S>>, f: impl FnOnce(&mut S) -> R) -> R {
    let state = &mut data.lock().expect("Could not lock mutex");
    f(state)
}

