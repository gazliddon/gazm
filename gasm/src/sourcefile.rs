use crate::locate::Position;
use std::hash::Hash;
use std::path::{Path, PathBuf};

use crate::ast::{AstNodeId, AstTree, ItemWithPos};
use std::collections::HashMap;

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
use std::fmt::{Debug, DebugMap};

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
fn get_tokenize_file(t: &AstTree, node_id: AstNodeId) -> Option<SourceFile> {
    t.get(node_id)
        .unwrap()
        .value()
        .item
        .get_my_tokenized_file()
        .map(|(f, _, s)| SourceFile::new(f, s))
}
fn set_file_ids(
    t: &mut AstTree,
    node_id: AstNodeId,
    file_node_id: AstNodeId,
    mapper: &mut HashMap<AstNodeId, SourceFile>,
) {
    let mut file_node_id = file_node_id;

    if let Some(source) = get_tokenize_file(t, node_id) {
        file_node_id = node_id;
        mapper.insert(node_id, source);
    }

    let mut node = t.get_mut(node_id).unwrap();
    node.value().file_id = Some(file_node_id);

    let children: Vec<_> = t.get(node_id).unwrap().children().map(|n| n.id()).collect();

    for c in children {
        set_file_ids(t, c, file_node_id, mapper)
    }
}

fn add_file_references(ast: &mut AstTree) -> HashMap<AstNodeId, SourceFile> {
    let root_id = ast.root().id();
    let mut hm = HashMap::new();
    set_file_ids(ast, root_id, root_id, &mut hm);
    hm
}

#[derive(Debug, Clone)]
pub struct NodeSourceInfo<'a> {
    pub fragment: &'a str,
    pub line_str: &'a str,
    pub line: usize,
    pub col: usize,
    pub source_file: &'a SourceFile,
    pub file: PathBuf,
}

#[derive(Debug)]
pub struct Sources {
    pub id_to_source_file: HashMap<AstNodeId, SourceFile>,
}

impl Sources {
    pub fn new(ast: &mut AstTree) -> Self {
        Self {
            id_to_source_file: add_file_references(ast),
        }
    }

    pub fn get_source_info_from_value<'a>(
        &'a self,
        v: &ItemWithPos,
    ) -> Result<NodeSourceInfo<'a>, String> {
        let pos = &v.pos;
        let file_id = v.file_id.ok_or_else(|| "No file id!".to_string())?;

        let source_file = self.id_to_source_file.get(&file_id).ok_or(format!(
            "Can't find file id {:?} {:?}",
            file_id, self.id_to_source_file
        ))?;
        let fragment = source_file.get_span(pos)?;
        let line_str = source_file.get_line(pos)?;

        let ret = NodeSourceInfo {
            line_str,
            col: pos.col,
            line: pos.line,
            fragment,
            source_file,
            file: source_file.file.clone(),
        };

        Ok(ret)
    }
}

use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct Mapping {
    pub file_id: u64,
    pub line: usize,
    pub range: std::ops::Range<usize>,

}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SourceMapping {
    addr_to_mapping: HashMap<u64,Mapping>,
    #[serde(skip)]
    pub node_id_to_file_id : HashMap<AstNodeId, u64>,
}

impl SourceMapping {
    pub fn new() -> Self {
        Self {
            addr_to_mapping: HashMap::new(),
            node_id_to_file_id: HashMap::new(),
        }
    }

    pub fn add_mapping(&mut self, addr: i64, id: Option<AstNodeId>, pos: &Position) {
        let addr = addr as u64;
        if let Some(id) = id {

            let next_id = self.node_id_to_file_id.len() as u64;

            let file_id = *self.node_id_to_file_id.entry(id).or_insert(next_id);

            let entry = Mapping {
                file_id,
                line: pos.line,
                range: pos.range.clone(),
            };

            self.addr_to_mapping.insert(addr, entry);
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SourceDatabase {
    source_files : HashMap<u64, PathBuf>,
    mappings : SourceMapping,
}

impl SourceDatabase {
    pub fn new(mappings: SourceMapping, sources : &Sources) -> Self {
        let mut source_files = HashMap::new();


        for (k,v) in &sources.id_to_source_file {
            let file_id = mappings.node_id_to_file_id.get(k).unwrap();
            source_files.insert(*file_id,v.file.clone());
        }

        Self {
            mappings,
            source_files,
        }
    }
}

