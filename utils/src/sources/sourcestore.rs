use super::{Position,AsmSource};
use super::symbols::SymbolTable;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

pub trait LocationTrait {
    fn get_line_number(&self) -> usize;
    fn get_file(&self) -> &String;
}

#[derive(Clone, Debug)]
pub struct SourceLine<L : LocationTrait> {
    pub loc: L,
    pub addr: Option<u16>,
    pub line: Option<String>,
}

pub trait SourceDataBase<L: LocationTrait> {
    fn addr_to_source_line(&self, addr: u16) -> Option<&SourceLine<L>>;
    fn get(&self, file: &str) -> Option<&Vec<SourceLine<L>>>;
    fn addr_to_loc(&self, _addr: u16) -> Option<&L>;

    fn loc_to_source_line(&self, loc: &L) -> Option<&SourceLine<L>> {
        let lines = self.get(&loc.get_file())?;
        lines.get(loc.get_line_number())
    }
}


////////////////////////////////////////////////////////////////////////////////
pub struct SourceFile {
    pub file: PathBuf,
    source: String,
    lines: Vec<String>,
}

impl SourceFile {
    pub fn new(file: &Path, source: &str) -> Self {
        let lines = source.lines().map(|x| x.to_string()).collect();
        Self {
            lines,
            file: file.to_path_buf(),
            source: source.to_string(),
        }
    }

    pub fn get_line(&self, p: &Position) -> Result<&str, String> {
        self.lines
            .get(p.line - 1)
            .map(|x| x.as_str())
            .ok_or_else(|| "Out of range".to_string())
    }

    pub fn get_span(&self, p: &Position) -> Result<&str, String> {
        // If the span is zero in length then return the single char at that position
        if p.range.is_empty() {
            Ok(&self.source[p.range.start..p.range.start + 1])
        } else {
            Ok(&self.source[p.range.clone()])
        }
    }
}
use std::fmt::Debug;

impl Debug for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut x = f.debug_struct("SourceFile");
        x.field("file", &self.file.to_string_lossy());
        x.finish()
    }
}

///////////////////////////////////////////////////////////////////////////////

// Add a source file to the hash if this is a source node
// return true if it did

#[derive(Debug, Clone)]
pub struct SourceInfo<'a> {
    pub fragment: &'a str,
    pub line_str: &'a str,
    pub line: usize,
    pub col: usize,
    pub source_file: &'a SourceFile,
    pub file: PathBuf,
}

#[derive(Debug)]
pub struct Sources {
    pub id_to_source_file: HashMap<u64, SourceFile>,
}

impl Sources {
    pub fn get_source_info<'a>(
        &'a self,
        pos: &Position,
    ) -> Result<SourceInfo<'a>, String> {

        if let AsmSource::FileId(file_id) = pos.src {
            let source_file = self.id_to_source_file.get(&file_id).ok_or(format!(
                "Can't find file id {:?} {:?}",
                file_id, self.id_to_source_file
            ))?;
            let fragment = source_file.get_span(pos)?;
            let line_str = source_file.get_line(pos)?;

            let ret = SourceInfo {
                line_str,
                col: pos.col,
                line: pos.line,
                fragment,
                source_file,
                file: source_file.file.clone(),
            };

            Ok(ret)
        } else {
            Err("No file id!".to_string())
        }
    }
}

use serde::{Deserialize, Serialize};
// use serde_json::json;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Mapping {
    pub file_id: u64,
    pub line: usize,
    pub range: std::ops::Range<usize>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SourceMapping {
    addr_to_mapping: HashMap<u64, Mapping>,
}

impl SourceMapping {
    pub fn new() -> Self {
        Self {
            addr_to_mapping: HashMap::new(),
        }
    }

    pub fn add_mapping(&mut self, addr: i64, pos: &Position) {
        let addr = addr as u64;

        if let AsmSource::FileId(file_id) = pos.src {
            let entry = Mapping {
                file_id,
                line: pos.line,
                range: pos.range.clone(),
            };

            self.addr_to_mapping.insert(addr, entry);
        } else {
            panic!("No file id!")
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDatabase {
    id_to_source_file : HashMap<u64,PathBuf>,
    mappings: SourceMapping,
    symbols : SymbolTable,
}

impl SourceDatabase {
    pub fn new(mappings: &SourceMapping, sources : &Sources, symbols : &SymbolTable) -> Self {
        let mut id_to_source_file = HashMap::new();

        for (k, v) in &sources.id_to_source_file {
            id_to_source_file.insert(*k, v.file.clone());
        }

        Self {
            mappings : mappings.clone(),
            id_to_source_file,
            symbols : symbols.clone()
        }
    }
}
