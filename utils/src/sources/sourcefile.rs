use super::Position;
use std::fmt::Debug;
///! In memrory representation of a source file
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct SourceFile {
    pub file: PathBuf,
    pub source: String,
    pub line_offsets: Vec<std::ops::Range<usize>>,
    pub num_of_lines: usize,
}

fn get_range(whole_buffer: &str, part: &str) -> std::ops::Range<usize> {
    let start = part.as_ptr() as usize - whole_buffer.as_ptr() as usize;
    let end = start + part.len();
    start..end
}

fn mk_offsets(source: &str) -> Vec<std::ops::Range<usize>> {
    source.lines().map(|x| get_range(source, x)).collect()
}

#[derive(Clone)]
pub struct TextPos {
    pub line: usize,
    pub character: usize,
}

/// Contains information for an edit to the in memrory text file
/// start..end is half open, end = the character after the last char to edit

pub struct TextEdit<'a> {
    start: TextPos,
    end: TextPos,
    text: &'a str,
}

impl<'a> TextEdit<'a> {
    pub fn new(start: TextPos, end: TextPos, text: &'a str) -> Self {
        Self { start, end, text }
    }
}

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum EditErrorKind {
    #[error("Index out of range")]
    IndexOutOfRange
}

pub type EditResult<T> = Result<T,EditErrorKind>;

impl SourceFile {
    pub fn new<P: AsRef<Path>>(file: P, source: &str) -> Self {
        let mut ret = Self {
            file: file.as_ref().to_path_buf(),
            source: source.to_string(),
            line_offsets: Default::default(),
            num_of_lines: Default::default(),
        };

        ret.build_line_table();
        ret
    }

    pub fn edit(&mut self, _edit: &TextEdit) -> EditResult<()> {
        let r = self.get_range(_edit)?;
        let first = &self.source[..r.start];
        let last = &self.source[r.end..];
        let new_source = first.to_owned() + &_edit.text + last;
        self.source = new_source;
        self.build_line_table();
        Ok(())
    }

    fn get_range(&self, edit: &TextEdit) -> EditResult<std::ops::Range<usize>> {
        let start_index = self.pos_to_index(edit.start.line, edit.start.character)?;
        let end_index = self.pos_to_index(edit.end.line, edit.end.character)?;
        assert!(start_index <= end_index);
        Ok( start_index..end_index )
    }

    fn build_line_table(&mut self) {
        self.line_offsets = mk_offsets(&self.source);
        self.num_of_lines = self.line_offsets.len();
    }

    pub fn pos_to_index(&self, line: usize, character: usize) -> EditResult<usize> {
        assert!(line < self.line_offsets.len());
        let ret = self.line_offsets[line].start + character;
        assert!(ret < self.source.len());
        Ok(ret)
    }

    pub fn get_num_of_lines(&self) -> usize {
        self.num_of_lines
    }

    /// Get Line n from source file
    /// LINE starts at zero, must be adjusted for position
    pub fn get_line(&self, line: usize) -> Option<&str> {
        self.line_offsets
            .get(line)
            .cloned()
            .map(|o| &self.source[o])
    }

    pub fn get_line_from_position(&self, p: &Position) -> &str {
        self.get_line(p.line - 1).unwrap()
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
