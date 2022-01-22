use super::chunk::Chunk;
use super::location::Location;

use super::error;
use std::collections::{HashMap, HashSet};

use log::{info, warn};


////////////////////////////////////////////////////////////////////////////////

pub type SourceLine = super::sources::SourceLine<Location>;

////////////////////////////////////////////////////////////////////////////////
fn load_lines(file: &str) -> error::Result<Vec<String>> {
    use std::fs;
    let err = format!("Couldn't not read {}", file);
    let contents = fs::read_to_string(file).expect(&err);
    Ok(contents.lines().map(|x| x.to_string()).collect())
}

pub struct SourceStore {
    annotated_files: HashMap<String, Vec<SourceLine>>,
    source_dir: String,
    loc_to_addr: HashMap<Location, u16>,
    addr_to_loc: HashMap<u16, Location>,
}

impl crate::sources::LocationTrait for Location {

    fn get_line_number(&self) -> usize {
        self.line
    }

    fn get_file(&self) -> &String {
        &self.file
    }
}
use crate::sources::SourceDataBase;

impl SourceDataBase<Location> for SourceStore {
    fn addr_to_source_line(&self, addr: u16) -> Option<&SourceLine> {
        let loc = self.addr_to_loc.get(&addr)?;
        self.loc_to_source_line(loc)
    }
    fn get(&self, file: &str) -> Option<&Vec<SourceLine>> {
        let key = self.make_key(file);
        self.annotated_files.get(&key)
    }

    fn addr_to_loc(&self, _addr: u16) -> Option<&Location> {
        self.addr_to_loc.get(&_addr)
    }
}

impl SourceStore {

    pub fn new(source_dir: &str, chunks: &[Chunk]) -> Self {
        let mut addr_to_loc = HashMap::new();
        let mut loc_to_addr = HashMap::new();
        let mut file_set = HashSet::new();

        info!("Interpreting {} chunks", chunks.len());

        // Cycle through the chunks, load all source
        for chunk in chunks {
            file_set.insert(chunk.location.file.clone());
            addr_to_loc.insert(chunk.addr, chunk.location.clone());
            loc_to_addr.insert(chunk.location.clone(), chunk.addr);
        }

        let mut annotated_files = HashMap::new();

        for file in &file_set {
            let key = Self::make_key_source_dir(source_dir, file);

            if let Ok(source_lines) = load_lines(&key) {
                let lines = source_lines
                    .iter()
                    .enumerate()
                    .map(|(i, line)| {
                        let loc = Location::new(file, i);
                        let addr = loc_to_addr.get(&loc).cloned();
                        SourceLine {
                            loc,
                            addr,
                            line: Some(line.clone()),
                        }
                    })
                    .collect();

                let annotated_source = lines ;

                info!("Annotated source file {}", &file);
                annotated_files.insert(key, annotated_source);
            } else {
                warn!("Can't annotate source file {}", file);
            }
        }

        Self {
            loc_to_addr,
            addr_to_loc,
            source_dir: source_dir.to_string(),
            annotated_files,
        }
    }

    fn make_key_source_dir(source_dir: &str, file: &str) -> String {
        format!("{}/{}", source_dir, file)
    }

    fn make_key(&self, file: &str) -> String {
        Self::make_key_source_dir(&self.source_dir, file)
    }


}
