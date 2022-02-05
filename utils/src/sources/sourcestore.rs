
use super::symbols::SymbolTable;
use super::{AsmSource, Position};
use std::collections::HashMap;
use std::path::PathBuf;

pub trait LocationTrait: Clone {
    fn get_line_number(&self) -> usize;
    fn get_file(&self) -> &PathBuf;
}

#[derive(Clone, Debug)]
pub struct SourceLine<'a> {
    pub text: String,
    pub file: PathBuf,
    pub file_id : u64,
    pub line_number: usize,
    pub mapping: Option<&'a Mapping>,
}

impl<'a> LocationTrait for SourceLine<'a> {
    fn get_line_number(&self) -> usize {
        self.line_number
    }

    fn get_file(&self) -> &PathBuf {
        &self.file
    }
}

pub struct SourceFileAccess<'a> {
    source_database: &'a SourceDatabase,
    pub file_id: u64,
    num_of_lines: usize,
}

impl<'a> SourceFileAccess<'a> {
    pub fn new(source_database: &'a SourceDatabase, file_id: u64, num_of_lines: usize) -> Self {
        Self {
            source_database,
            file_id,
            num_of_lines,
        }
    }

    pub fn num_of_lines(&self) -> usize {
        self.num_of_lines
    }

    pub fn get_line(&self, line: usize) -> Option<SourceLine<'a>> {
        self.source_database.get_source_line(self.file_id, line)
    }
}

////////////////////////////////////////////////////////////////////////////////
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
    pub fn new(file: &PathBuf, source: &str) -> Self {
        let lines = source.lines().map(|x| x.to_string()).collect();
        Self {
            lines,
            file: file.to_path_buf(),
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
use std::str::FromStr;

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
    pub fn new() -> Self {
        Self {
            id_to_source_file: HashMap::new(),
        }
    }

    pub fn add_source_text(&mut self, text : &str ) -> u64 {
        let max = self.id_to_source_file.keys().max();

        let id = if let Some(max) = max {
            max + 1

        } else {
            0
        };

        let name = format!("macro_epxansion_{}",id);
        let source_file = SourceFile::new(&PathBuf::from_str(&name).unwrap(), text);
        self.id_to_source_file.insert(id, source_file);
        id
    }

    pub fn get_source_info<'a>(&'a self, pos: &Position) -> Result<SourceInfo<'a>, String> {
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

#[derive(PartialEq,Debug, Serialize, Deserialize, Clone)]
pub enum ItemType {
    OpCode,
    Command,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mapping {
    pub file_id: u64,
    pub line: usize,
    pub mem_range: std::ops::Range<usize>,
    pub item_type : ItemType,
}

use crate::Stack;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceMapping {
    pub addr_to_mapping: Vec<Mapping>,
    #[serde(skip)]
    macro_stack: Stack<Position>,
}

impl Default for SourceMapping {
    fn default() -> Self {
        Self {
            addr_to_mapping: Default::default(),
            macro_stack : Default::default(),
        }
    }
}

impl SourceMapping {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_macro(&mut self, pos : &Position) {
        self.macro_stack.push(pos.clone())
    }

    pub fn stop_macro(&mut self) {
        self.macro_stack.pop();
    }

    fn get_macro_pos(&self) -> Option<&Position> {
        self.macro_stack.front()
    }

    fn is_expanding_macro(&self) -> bool {
        self.macro_stack.is_empty() == false
    }

    pub fn add_mapping(&mut self, mem_range: std::ops::Range<usize>, pos: &Position, item_type: ItemType) {

        let pos = self.get_macro_pos().unwrap_or(pos);

        if let AsmSource::FileId(file_id) = pos.src {
            let entry = Mapping {
                file_id,
                line: pos.line,
                mem_range: mem_range.clone(),
                item_type
            };

            self.addr_to_mapping.push(entry);
        } else {
            panic!("No file id!")
        }
    }
}
use std::cell::RefCell;

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceDatabase {
    id_to_source_file: HashMap<u64, PathBuf>,
    mappings: SourceMapping,
    symbols: SymbolTable,
    #[serde(skip)]
    range_to_mapping: HashMap<std::ops::Range<usize>, Mapping>,
    #[serde(skip)]
    addr_to_mapping: HashMap<usize, Mapping>,
    #[serde(skip)]
    source_files: RefCell<HashMap<u64, SourceFile>>,
    #[serde(skip)]
    source_file_to_id: HashMap<PathBuf, u64>,
    #[serde(skip)]
    loc_to_mapping: HashMap<(u64, usize), Mapping>,
}

pub struct SourceLineInfo<'a> {
    file: &'a String,
    line: usize,
    mem_range: std::ops::Range<u64>,
}

impl SourceDatabase {
    pub fn new(mappings: &SourceMapping, sources: &Sources, symbols: &SymbolTable) -> Self {
        let mut id_to_source_file = HashMap::new();

        for (k, v) in &sources.id_to_source_file {
            id_to_source_file.insert(*k, v.file.clone());
        }

        let mut ret = Self {
            source_files: Default::default(),
            source_file_to_id: Default::default(),
            mappings: mappings.clone(),
            id_to_source_file,
            symbols: symbols.clone(),
            range_to_mapping: Default::default(),
            addr_to_mapping: Default::default(),
            loc_to_mapping: Default::default(),
        };

        ret.post_deserialize();

        ret
    }

    pub fn from_json(sym_file: &str) -> Self {
        let symstr = std::fs::read_to_string(sym_file).unwrap();
        let mut sd: SourceDatabase = serde_json::from_str(&symstr).unwrap();
        sd.post_deserialize();
        sd
    }

    fn post_deserialize(&mut self) {
        for v in &self.mappings.addr_to_mapping {
            self.range_to_mapping.insert(v.mem_range.clone(), v.clone());
            self.addr_to_mapping.insert(v.mem_range.start, v.clone());
            let loc = (v.file_id, v.line);
            self.loc_to_mapping.insert(loc, v.clone());
        }

        for (k, v) in &self.id_to_source_file {
            self.source_file_to_id.insert(v.clone(), *k);
        }
    }

    fn load_source_file<'a>(&'a self, file_id: u64) -> Result<(), ()> {
        let file_name = self.id_to_source_file.get(&file_id).ok_or(())?;
        let x = self.source_files.borrow().contains_key(&file_id);

        if !x {
            let s = std::fs::read_to_string(file_name).expect("Should have read source file");
            let mut x = self.source_files.borrow_mut();
            x.insert(file_id, SourceFile::new(file_name, &s));
            x.get(&file_id);
        }

        Ok(())
    }

    pub fn get_source_file_from_file_name<'a>(
        &'a self,
        file_name: &PathBuf,
    ) -> Option<SourceFileAccess<'a>> {
        self.source_file_to_id
            .get(file_name)
            .and_then(|file_id| self.get_source_file(*file_id))
    }

    pub fn get_source_file<'a>(&'a self, file_id: u64) -> Option<SourceFileAccess<'a>> {
        self.func_source_file(file_id, |sf| {
            let num_of_lines = sf.get_num_of_lines();
            Some(SourceFileAccess::new(self, file_id, num_of_lines))
        })
    }

    fn get_source_line(&self, file_id: u64, line: usize) -> Option<SourceLine> {
        self.func_source_file(file_id, |sf| {
            sf.lines.get(line - 1).map(|text| SourceLine {
                file_id,
                file: sf.file.clone(),
                line_number: line,
                text: text.to_string(),
                mapping: self.loc_to_mapping.get(&(file_id, line)),
            })
        })
    }

    pub fn func_source_file<F, R>(&self, file_id: u64, func: F) -> Option<R>
    where
        F: Fn(&SourceFile) -> Option<R>,
    {
        self.load_source_file(file_id).unwrap();
        self.source_files
            .borrow()
            .get(&file_id)
            .and_then(|sf| func(sf))
    }

    pub fn get_source_line_from_file(
        &self,
        file_name: &PathBuf,
        line: usize,
    ) -> Option<SourceLine> {
        self.get_source_file_from_file_name(file_name)
            .and_then(|sf| sf.get_line(line))
    }

    pub fn get_source_info_from_address(&self, addr: usize) -> Option<SourceLine> {
        self.addr_to_mapping
            .get(&addr)
            .and_then(|m| self.get_source_line(m.file_id, m.line))
    }
}
