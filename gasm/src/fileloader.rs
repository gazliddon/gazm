use std::path::{Path, PathBuf,};
use std::fs;

pub struct FileLoader {
    search_paths : Vec<PathBuf>
}

impl Default for FileLoader {
    fn default() -> Self {
        let vec : Vec<&str> = vec![];
        Self::from_search_paths(&vec)
    }
}

impl FileLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_search_paths<P: AsRef<Path>>(paths : &[P]) -> Self {
        let search_paths : Vec<PathBuf> = paths.iter().map(|x| PathBuf::from(x.as_ref())).collect();
        Self {
            search_paths
        }
    }

    pub fn read_to_string <P: AsRef<Path>>(&self, path : P) -> std::io::Result<(PathBuf, String )> {
        if let Some(path) = self.get_file_name(path.as_ref()) {
            let ret = fs::read_to_string(path.clone())?;
            Ok((path,ret))
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::NotFound))
        }
    }

    fn get_file_name<P: AsRef<Path>>(&self, file_name : P) -> Option<PathBuf> {
        let file_name = file_name.as_ref();

        if file_name.exists() {
            Some(file_name.to_path_buf())
        } else {
            if !file_name.has_root() {
                for i in &self.search_paths {
                    let x = i.join(file_name);
                    if x.exists() {
                        return Some(x)
                    }
                }
            }
            None
        }
    }
}
