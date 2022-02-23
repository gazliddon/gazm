use std::fs;
use std::path::{Path, PathBuf};

use super::sourcestore::{SourceFile, Sources};
use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct SourceFileLoader {
    pub search_paths: Vec<PathBuf>,
    pub sources: Sources,
    id: u64,
}

impl Default for SourceFileLoader {
    fn default() -> Self {
        let vec: Vec<&str> = vec![];
        Self::from_search_paths(&vec)
    }
}

impl SourceFileLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_search_paths<P: AsRef<Path>>(paths: &[P]) -> Self {
        let search_paths: Vec<PathBuf> = paths.iter().map(|x| PathBuf::from(x.as_ref())).collect();
        Self {
            search_paths,
            sources: Sources::new(),
            id: 0,
        }
    }

    pub fn read_to_string<P: AsRef<Path>>(&mut self, path: P) -> Result<(PathBuf, String, u64)> {

        let path = self.get_file_name(path.as_ref())?;
        let ret = fs::read_to_string(path.clone())?;
        let id = self.id;
        let source_file = SourceFile::new(&path, &ret);
        self.sources.id_to_source_file.insert(id, source_file);
        self.id += 1;

        Ok((path, ret, id))
    }

    pub fn get_size<P: AsRef<Path>>(&self, path : P) -> Result<usize> {
        let path = self.get_file_name(path.as_ref())?;

        let md = std::fs::metadata(path.clone()).map_err(|e|
            anyhow!(
                "Can't get size for : {}\n{}",
                path.to_string_lossy(),e
            )
            )?;

        Ok(md.len() as usize)
    }

    pub fn read_binary_chunk<P: AsRef<Path>>(&self, path: P, r: std::ops::Range<usize>) -> Result<(PathBuf, Vec<u8>)> {
        let (path, buff) = self.read_binary(path)?;

        let buff_r = 0..buff.len();

        let start = r.start;
        let last = ( r.len() + r.start ) -1;

        if buff_r.contains(&start) && buff_r.contains(&last) {
            Ok((path, buff[r].into() ))
        } else {
            Err(anyhow!(
                "Cannot read binary chunk. Range exceeds size of file {}: file size is {}, tried to grab up to {}",
                path.to_string_lossy(), buff_r.len(), last
                ))
        }
    }

    pub fn read_binary<P: AsRef<Path>>(&self, path: P) -> Result<(PathBuf, Vec<u8>)> {
        use std::fs::File;
        use std::io::Read;
        let mut buffer = vec![];

        let str_path = path.as_ref().to_string_lossy();
        let path = self.get_file_name(path.as_ref()).map_err(|e| {
            anyhow!(
                "File {} doesn't exist in any search paths\nTried:\n{}",
                str_path,
                e)
        })?;

        let mut file = File::open(path.clone())?;
        file.read_to_end(&mut buffer)?;
        Ok((path, buffer))
    }

    fn get_file_name<P: AsRef<Path>>(&self, file_name: P) -> Result<PathBuf> {
        let file_name = file_name.as_ref();

        let mut tried: Vec<String> = vec![file_name.to_string_lossy().into()];

        if file_name.exists() {
            Ok(file_name.to_path_buf())
        } else {
            if !file_name.has_root() {
                for i in &self.search_paths {
                    let x = i.join(file_name);
                    if x.exists() {
                        return Ok(x);
                    } else {
                        tried.push(x.to_string_lossy().into());
                    }
                }
            }

            Err(anyhow!(format!("Can't find file {}\n Tried: {}", file_name.to_string_lossy(), tried.join("\n"))))
        }
    }
}
