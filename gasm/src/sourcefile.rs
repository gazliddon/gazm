use super::locate::Position;
use std::path::PathBuf;

////////////////////////////////////////////////////////////////////////////////
pub struct SourceFile {
    pub file : PathBuf,
    source: String,
    lines: Vec<String>,
}

impl SourceFile {
    pub fn new(file : &PathBuf, source: &String) -> Self {
        let lines = source.lines().map(|x| x.to_string()).collect();
        Self {lines, file : file.clone(), source: source.clone()}
    }

    pub fn get_line(&self,p : &Position) -> Result<&str, String> {
        self.lines.get(p.line - 1).map(|x| x.as_str()).ok_or("Out of range".to_string())
    }

    pub fn get_span(&self,p : &Position) -> Result<&str, String> {
        // If the span is zero in length then return the single char at that position
        if p.range.len() == 0 {
            Ok(&self.source[p.range.start..p.range.start+1])
        } else {

            Ok(  &self.source[p.range.clone()]  )
        }
    }
}
use std::fmt::{Debug, DebugMap};

impl Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut x = f.debug_struct("SourceFile");
        x.field("file", &self.file.to_string_lossy());
        x.finish()
    }
}
