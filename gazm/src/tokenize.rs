#![forbid(unused_imports)]
use std::path::PathBuf;


use crate::{
    error::ParseError,
    item::{Item, Node},
    opts::Opts,
};

use grl_sources::Position;


pub fn mk_pc_equate(node: &Node) -> Node {
    use Item::{AssignmentFromPc, Label, LocalAssignmentFromPc, LocalLabel};
    let pos = node.ctx;

    match &node.item {
        Label(label_def) => Node::new(AssignmentFromPc(label_def.clone()), pos),
        LocalLabel(label_def) => Node::new(LocalAssignmentFromPc(label_def.clone()), pos),
        _ => panic!("shouldn't happen"),
    }
}

#[derive(Default)]
pub struct Tokens {
    pub tokens: Vec<Node>,
    pub opts: Opts,
    pub parse_errors: Vec<ParseError>,
    pub includes: Vec<(Position, PathBuf)>,
    docs: DocTracker,
}

#[derive(Default)]
struct DocTracker {
    doc_lines: Vec<String>,
}

impl DocTracker {
    pub fn has_docs(&self) -> bool {
        !self.doc_lines.is_empty()
    }

    pub fn add_doc_line(&mut self, doc: &str) {
        self.doc_lines.push(doc.to_string())
    }
    pub fn flush_docs(&mut self) -> Option<String> {
        if self.has_docs() {
            let ret = self.doc_lines.join("\n");
            *self = Default::default();
            Some(ret)
        } else {
            None
        }
    }
}




