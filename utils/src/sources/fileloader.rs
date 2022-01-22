use std::path::{Path, PathBuf,};
use std::fs;

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use super::sourcestore::{ SourceFile, Sources };

impl Into<Sources> for SourceFileLoader {
    fn into(self) -> Sources {
        Sources {
            id_to_source_file: self.loaded_files,
        }
    }
}

pub struct SourceFileLoader {
    pub search_paths : Vec<PathBuf>,
    pub loaded_files : HashMap<u64, SourceFile>,
    id : u64,
}

impl Default for SourceFileLoader {
    fn default() -> Self {
        let vec : Vec<&str> = vec![];
        Self::from_search_paths(&vec)
    }
}

impl SourceFileLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_search_paths<P: AsRef<Path>>(paths : &[P]) -> Self {
        let search_paths : Vec<PathBuf> = paths.iter().map(|x| PathBuf::from(x.as_ref())).collect();
        Self {
            search_paths,
            loaded_files : Default::default(),
            id: 0,
        }
    }

    pub fn read_to_string <P: AsRef<Path>>(&mut self, path : P) -> Result<(PathBuf, String, u64 )> {
        let str_path = path.as_ref().to_string_lossy();

        let path = self.get_file_name(path.as_ref()).map_err(|e|
            anyhow!("File {} doesn't exist in any seach paths\nTried:\n{}", str_path, e)
        )?;

        let ret = fs::read_to_string(path.clone())?;
        let id = self.id;
        let source_file = SourceFile::new(&path, &ret);
        self.loaded_files.insert(id, source_file);
        self.id += 1;

        Ok((path, ret, id))
    }

    fn get_file_name<P: AsRef<Path>>(&self, file_name : P) -> Result<PathBuf> {

        let file_name = file_name.as_ref();

        let mut tried : Vec<String> = vec![file_name.to_string_lossy().into()];

        if file_name.exists() {
            Ok(file_name.to_path_buf())
        } else {
            if !file_name.has_root() {
                for i in &self.search_paths {
                    let x = i.join(file_name);
                    if x.exists() {
                        return Ok(x)
                    } else {
                        tried.push(x.to_string_lossy().into());
                    }
                }
            }

            Err(anyhow!(format!("{}", tried.join("\n"))))
        }
    }
}
