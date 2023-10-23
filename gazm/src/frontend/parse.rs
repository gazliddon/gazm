use std::path::PathBuf;

use super::{make_tspan, FrontEndError, FrontEndErrorKind, PResult, TSpan, Token, TokenKind};
use crate::item::Node;
use crate::opts::Opts;
use grl_sources::{AsmSource, SourceFile};

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

use unraveler::cut;

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
        let (_sp, node) = cut(parse_source_file)(spam)?;

        Ok(Parsed {
            node,
            includes: Default::default(),
            request: self,
        })
    }
}

pub fn parse_source_file(_input: TSpan) -> PResult<Node> {
    panic!()
}
