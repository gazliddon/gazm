use crate::ctx::{Context, Opts};
use std::path::{Path, PathBuf};

use crate::assemble::Assembler;
use crate::ast::Ast;
use crate::item::Node;
use crate::tokenize::{tokenize, tokenize_file_from_str};

use utils::sources::Position;

use thiserror::Error;

use crate::error::UserError;

pub struct Gasm {
    ctx: Context,
    opts: Opts,
}

pub type GResult<T> = Result<T, GasmError>;

#[derive(Error, Debug, Clone)]
pub enum GasmError {
    #[error(transparent)]
    UserError(#[from] UserError),
    #[error("Misc: {0}")]
    Misc(String),
    #[error("Too Many Errors")]
    TooManyErrors,
}

impl From<String> for GasmError {
    fn from(x: String) -> Self {
        GasmError::Misc(x)
    }
}
impl From<anyhow::Error> for GasmError {
    fn from(x: anyhow::Error) -> Self {
        GasmError::Misc(x.to_string())
    }
}

impl Gasm {
    pub fn new(ctx: Context, opts: Opts) -> Self {
        Self { ctx, opts }
    }

    pub fn assemble_file(&mut self, x: &Path) -> GResult<()> {
        use utils::PathSearcher;
        let paths = self.ctx.source_file_loader.get_search_paths().clone();

        if let Some(dir) = x.parent() {
            self.ctx.source_file_loader.add_search_path(dir);
        }

        let tokens = tokenize(&mut self.ctx, &self.opts, x)?;
        let ret = self.assemble_tokens(tokens);

        self.ctx.source_file_loader.set_search_paths(paths);
        ret
    }

    pub fn assemble_text(&mut self, _x: &str) -> GResult<()> {
        self.ctx.errors.raise_errors()
    }

    fn assemble_tokens(&mut self, tokens: Node) -> GResult<()> {
        let tree = Ast::from_nodes(&mut self.ctx, tokens)?;

        use crate::assemble::Assembler;

        let mut a = Assembler::new(&mut self.ctx, &self.opts, tree)?;
        a.assemble()?;

        self.ctx.errors.raise_errors()?;

        Ok(())
    }
}
