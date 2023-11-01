#![deny(unused_imports)]
 
use crate::{item::Node, opts::Opts };

use grl_sources::SourceFile;

use std::path::PathBuf;
use super::{
    Token,FrontEndError,make_tspan,parse_span
};

#[derive(Debug, Clone)]
pub struct ParseTask {
    opts: Opts,
    source_file: SourceFile,
}

#[derive(Debug, Clone)]
pub struct Parsed {
    pub node: Node,
    pub includes: Vec<PathBuf>,
    pub request: ParseTask,
}

impl ParseTask {
    pub fn from_text(opts: &Opts, text: &str) -> Self {
        let source_file = SourceFile::new("NO FILE", text, 0);
        Self::from_source(opts, &source_file)
    }

    pub fn from_source(opts: &Opts, source_file: &SourceFile) -> Self {
        Self {
            opts: opts.clone(),
            source_file: source_file.clone(),
        }
    }

    fn tokenize(&self) -> Vec<Token> {
        super::to_tokens_filter(&self.source_file, |k| k.is_comment())
    }
}

impl TryInto<Parsed> for ParseTask {
    type Error = FrontEndError;

    fn try_into(self) -> Result<Parsed, Self::Error> {
        let tokens = self.tokenize();
        let spam = make_tspan(&tokens, &self.source_file);
        let (_, node) = parse_span(spam)?;



        Ok(Parsed {
            node,
            includes: Default::default(),
            request: self,
        })
    }
}

