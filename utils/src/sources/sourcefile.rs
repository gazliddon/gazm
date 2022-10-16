use super::Position;
use super::{ TextEditTrait, TextFile };
use std::fmt::Debug;
///! In memrory representation of a source file
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct SourceFile {
    pub file: PathBuf,
    pub source: TextFile,
}


impl SourceFile {
    pub fn new<P: AsRef<Path>>(file: P, source: &str) -> Self {
        Self {
            file: file.as_ref().to_path_buf(),
            source: TextFile::new(source),
        }
    }

    pub fn as_text_edit(&mut self) -> &TextFile {
        &self.source
    }

    /// Get Line n from source file
    /// LINE starts at zero, must be adjusted for position
    pub fn get_line(&self, line: usize) -> Option<&str> {
        self.source.get_line(line).ok()
    }

    pub fn get_line_from_position(&self, p: &Position) -> &str {
        self.get_line(p.line - 1).unwrap()
    }

    pub fn get_span(&self, p: &Position) -> &str {
        // If the span is zero in length then return the single char at that position
        let range = if p.range.is_empty() {
            p.range.start..p.range.start + 1
        } else {
            p.range.clone()
        };
        &self.source.source[range]
    }
}

impl Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut x = f.debug_struct("SourceFile");
        x.field("file", &self.file.to_string_lossy());
        x.finish()
    }
}
