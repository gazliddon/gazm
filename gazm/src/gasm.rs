use crate::ctx::{ Context, Opts };
use std::path::{Path, PathBuf};

use crate::assemble::Assembler;
use crate::ast::Ast;
use crate::tokenize::{tokenize, tokenize_file_from_str};
use crate::item::Node;

use utils::sources::Position;

use thiserror::Error;

use crate::error::UserError;

pub struct Gasm {
    ctx: Context,
    opts: Opts,
}

pub type GResult<T> = Result<T,GasmError>;

#[derive(Error,Debug, Clone)]
pub enum GasmError {
    #[error("User error")]
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

    pub fn assemble_file(&mut self, _x: &Path) -> GResult<()> {
        let tokens = tokenize(&mut self.ctx, &self.opts)?;
        self.assemble_tokens(tokens)
    }

    pub fn assemble_text(&mut self, _x: &str) -> GResult<()> {
        self.ctx.errors.raise_errors()
    }

    fn assemble_tokens(&mut self, _tokens : Node) -> GResult<()> {
        // let tree = Ast::from_nodes(&mut self.ctx, tokens)?;

        use crate::sizer::Sizer;
        // let mut sizer = Sizer::new(self);
        // sizer.size()

        Ok(())

    }

    pub fn get_symbols(&self) {}
}
