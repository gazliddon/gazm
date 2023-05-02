use std::path::{Path, PathBuf};
use crate::async_tokenize::TokenizeResult;

use crate::item::{Item, Node};
use std::collections::HashMap;

#[derive(Default, Clone, Debug)]
pub struct TokenStore {
    pub tokens: HashMap<PathBuf, TokenizeResult>,
}

impl TokenStore {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn get_tokens<P: AsRef<Path>>(&self, file: P) -> Option<&TokenizeResult> {
        self.tokens.get(&file.as_ref().to_path_buf())
    }

    pub fn add_tokens(&mut self, tokes: TokenizeResult) {
        let file = tokes.request.full_file_name.clone();
        self.tokens.insert(file, tokes);
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
