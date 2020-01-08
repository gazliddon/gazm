use super::chunk::{ Location };

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader};
use super::error;
use std::collections::HashMap;


////////////////////////////////////////////////////////////////////////////////

pub struct SourceFile {
    file : String,
    lines : Vec<String>
}

impl SourceFile {
    pub fn new( file : &str ) -> error::Result<Self> {
        // info!("Trying to load {}", file);
        let f = File::open(file)?;
        let f = BufReader::new(f);

        let lines : Result<Vec<_>, _> = f.lines().collect();

        let ret = Self {
            file : file.to_string(),
            lines : lines?
        };

        Ok(ret)
    }

    pub fn line(&self, line : usize) -> Option<&String> {
        self.lines.get(line -1)
    }
}
use std::cell::RefCell;

pub struct SourceStore {
    files: RefCell<HashMap<String,SourceFile>>,
    source_dir : String
}

impl SourceStore {

    pub fn new(source_dir : &str) -> Self {
        Self {
            files: RefCell::new(HashMap::new()),
            source_dir : source_dir.to_string()
        }
    }

    fn make_key(&self, file : &str) -> String {
        format!("{}/{}", self.source_dir, file)
    }

    pub fn get<F>(&self, file : &str, func : F) where
        F : FnOnce(&SourceFile) {
            let key = self.make_key(file);

            let mut files  = self.files.borrow_mut();

            if !files.contains_key(&key) {
                if let Ok(source_file) = SourceFile::new(&key) {
                    files.insert(key.clone(), source_file);
                } else {
                    return;
                }
            }

            func(files.get(&key).unwrap())
        }

    pub fn get_line(&self, loc : &Location) -> Option<String> {
        let mut res =  None;

        self.get(
            loc.file.to_str().unwrap(),
            |sf| {
                res = sf.line(loc.line_number).cloned();
            }
        );

        res
    }
}
