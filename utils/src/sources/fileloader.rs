use std::fs;
use std::path::{Path, PathBuf};

use super::sourcestore::{SourceFile, Sources};
use anyhow::{anyhow, Result};

use crate::pathsearcher::{ Paths, PathSearcher };

#[derive(Debug)]
pub struct SourceFileLoader {
    pub bin_search_paths: Paths,
    pub source_search_paths: Paths,
    pub sources: Sources,
    id: u64,
    pub files_loaded: Vec<PathBuf>,
    pub files_written: Vec<PathBuf>,
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
            source_search_paths: Paths::from_paths(&search_paths),
            bin_search_paths: Paths::new(),
            sources: Sources::new(),
            id: 0,
            files_loaded: Default::default(),
            files_written: Default::default(),
        }
    }

    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&mut self, path: P, data: C) {
        let path = path.as_ref().to_path_buf();
        std::fs::write(&path, data).expect("Can't write bin file");
        self.files_written.push(path);
    }

    pub fn add_bin_search_path<P: AsRef<Path>>(&mut self, path: &P) {
        self.bin_search_paths.add_path(path);
    }

    pub fn add_bin_search_paths<P: AsRef<Path>>(&mut self, paths: &[P]) {
        for p in paths {
            self.add_bin_search_path(p)
        }
    }

    pub fn read_to_string<P: AsRef<Path>>(&mut self, path: P) -> Result<(PathBuf, String, u64)> {
        let path = self.source_search_paths.get_full_path(path.as_ref())
            .map_err(|e| self.mk_error(e))?;

        let ret = fs::read_to_string(path.clone())?;
        let id = self.id;
        let source_file = SourceFile::new(&path, &ret);
        self.sources.id_to_source_file.insert(id, source_file);
        self.id += 1;

        self.files_loaded.push(path.clone());

        Ok((path, ret, id))
    }

    fn mk_error(
        &self,
        e: crate::pathsearcher::SearchError,
    ) -> anyhow::Error {
        use crate::pathsearcher::SearchError::*;

        let e = match e {
            FileNotFound(f, v) => {
                let errs: Vec<String> = v.into_iter().map(|f| f.to_string_lossy().to_string()).collect();
                anyhow!(
                    "Can't load {}\n Tried:\n{}",
                    f.to_string_lossy(),
                    errs.join("\n")
                )
            }
            _ => {
                panic!()
            }
        };
        e
    }

    pub fn get_size<P: AsRef<Path>>(&self, path: P) -> Result<usize> {
        let path_finder = vec![&self.bin_search_paths, &self.source_search_paths];

        let path = path_finder
            .get_full_path(path.as_ref())
            .map_err(|e| self.mk_error(e))?;

        let md = std::fs::metadata(path.clone())
            .map_err(|e| anyhow!("Can't get size for : {}\n{}", path.to_string_lossy(), e))?;

        Ok(md.len() as usize)
    }

    pub fn read_binary_chunk<P: AsRef<Path>>(
        &mut self,
        path: P,
        r: std::ops::Range<usize>,
    ) -> Result<(PathBuf, Vec<u8>)> {
        let (path, buff) = self.read_binary(path)?;

        let buff_r = 0..buff.len();

        let start = r.start;
        let last = (r.len() + r.start) - 1;

        if buff_r.contains(&start) && buff_r.contains(&last) {
            Ok((path, buff[r].into()))
        } else {
            Err(anyhow!(
                "Cannot read binary chunk. Range exceeds size of file {}: file size is {}, tried to grab up to {}",
                path.to_string_lossy(), buff_r.len(), last
                ))
        }
    }

    fn read_binary<P: AsRef<Path>>(&mut self, path: P) -> Result<(PathBuf, Vec<u8>)> {
        use std::fs::File;
        use std::io::Read;
        let mut buffer = vec![];

        let path_finder = vec![&self.bin_search_paths, &self.source_search_paths];

        let path = path_finder
            .get_full_path(path.as_ref())
            .map_err(|e| self.mk_error(e))?;

        let mut file = File::open(path.clone())?;
        file.read_to_end(&mut buffer)?;

        self.files_loaded.push(path.clone());

        Ok((path, buffer))
    }
}
