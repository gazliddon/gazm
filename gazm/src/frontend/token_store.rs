#![forbid(unused_imports)]
use super::TokenizeResult;

use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
};

use thin_vec::ThinVec;

#[derive(Default, Clone, Debug)]
pub struct TokenStore {
    pub tokens: HashMap<PathBuf, TokenizeResult>,
}

/// Cache containing tokenized versions of source files
impl TokenStore {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Get any cached tokens for this file
    pub fn get_tokens<P: AsRef<Path>>(&self, file: P) -> Option<&TokenizeResult> {
        self.tokens.get(&file.as_ref().to_path_buf())
    }

    /// Add tokens for this file
    pub fn add_tokens(&mut self, tokes: TokenizeResult) {
        let file = tokes.request.get_file_name().clone();
        self.tokens.insert(file, tokes);
    }

    /// Are there tokens for this file?
    pub fn has_tokens<P: AsRef<Path>>(&self, file: P) -> bool {
        self.get_tokens(file).is_some()
    }

    /// Scrub this file's cache entry
    pub fn invalidate_tokens<P: AsRef<Path>>(&mut self, file: P) {
        if self.has_tokens(&file) {
            let file = file.as_ref().to_path_buf();
            self.tokens.remove(&file);
        }
    }

    /// Get a list of files we're looking after tokens for
    /// as strings
    pub fn get_files(&self) -> ThinVec<Cow<str>> {
        self.tokens.keys().map(|k| k.to_string_lossy()).collect()
    }
}
