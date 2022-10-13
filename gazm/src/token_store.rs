use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};

use crate::item::{Item, Node};
use std::collections::HashMap;

#[derive(Default, Clone, Debug)]
pub struct TokenStore {
    tokens: HashMap<PathBuf, Node>,
}

#[derive(Clone, Debug)]
pub struct TokenEntry {
    node: Node,
    hash: Digest,
}

impl TokenEntry {
    pub fn new(source: &str, node: Node) -> Self {
        let mut hasher = Sha1::new();
        hasher.update(source.as_bytes());
        let hash = hasher.digest();
        Self { node, hash }
    }
}

impl TokenStore {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn get_tokens<P: AsRef<Path>>(&self, file: P) -> Option<&Node> {
        self.tokens.get(&file.as_ref().to_path_buf())
    }

    pub fn add_tokens<P: AsRef<Path>>(&mut self, file: P, node: Node) {
        self.tokens.insert(file.as_ref().to_path_buf(), node);
    }

    pub fn add_tokens_hash<P: AsRef<Path>>(&mut self, _file: P, node: Node, src: &str) {
        let _te = TokenEntry::new(src, node);
        // self.tokens.insert(file.as_ref().to_path_buf(), node);
    }

    pub fn has_tokens<P: AsRef<Path>>(&self, file: P) -> bool {
        self.get_tokens(file).is_some()
    }

    pub fn invalidate_tokens<P: AsRef<Path>>(&mut self, file: P) {
        if self.has_tokens(&file) {
            let file = file.as_ref().to_path_buf();
            self.tokens.remove(&file);
        }
    }
}
