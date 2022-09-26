///! In memrory representation of a source file

use std::path::{Path, PathBuf};
use std::fmt::Debug;
use super::Position;

#[derive(Clone)]
pub struct SourceFile {
    pub file: PathBuf,
    pub source: String,
    pub lines: Vec<String>,
}

pub struct SourceFileInfo {
    num_of_lines: usize,
    file: PathBuf,
}

impl SourceFile {
    pub fn new<P: AsRef<Path>>(file: P, source: &str) -> Self {
        let lines = source.lines().map(|x| x.to_string()).collect();
        Self {
            lines,
            file: file.as_ref().to_path_buf(),
            source: source.to_string(),
        }
    }

    pub fn mk_info(&self) -> SourceFileInfo {
        SourceFileInfo {
            file: self.file.clone(),
            num_of_lines: self.lines.len(),
        }
    }

    pub fn get_num_of_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn get_line(&self, p: &Position) -> &str {
        self.lines
            .get(p.line - 1)
            .map(|x| x.as_str())
            .expect("Out of range!")
    }

    pub fn get_span(&self, p: &Position) -> &str {
        // If the span is zero in length then return the single char at that position
        if p.range.is_empty() {
            &self.source[p.range.start..p.range.start + 1]
        } else {
            &self.source[p.range.clone()]
        }
    }
}

impl Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut x = f.debug_struct("SourceFile");
        x.field("file", &self.file.to_string_lossy());
        x.finish()
    }
}

#[derive(Debug, Clone)]
pub struct SourceInfo<'a> {
    pub fragment: &'a str,
    pub line_str: &'a str,
    pub line: usize,
    pub col: usize,
    pub source_file: &'a SourceFile,
    pub file: PathBuf,
    pub pos: Position,
}
