#![deny(unused_imports)]

use grl_sources::{ SourceFile,Position };

use crate::cli::CpuKind; 

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParserState {
    pub cpu : CpuKind,
}

impl Default for ParserState {
    fn default() -> Self {
        Self { cpu: CpuKind::Cpu6809 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParseText<'a> {
    pub pos : Position,
    pub source_file : &'a SourceFile,
    pub state: ParserState,
}


impl<'a> ParseText<'a> {
    pub fn new(source_file: &'a SourceFile, range: std::ops::Range<usize>, ) -> Self {
        let pos = source_file.get_position(range);
        Self {
            pos,
            source_file,
            state: ParserState::default(),
        }
    }
}

impl<'a> ParseText<'a> {
    pub fn get_text(&self) -> &str {
        self.source_file.get_span(&self.pos)
    }

    pub fn as_range(&self) -> std::ops::Range<usize> {
        self.pos.range()
    }
}
