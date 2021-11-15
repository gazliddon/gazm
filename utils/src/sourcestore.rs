use super::location::Location;
use super::chunk::Chunk;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use super::error;
use std::collections::{HashMap,HashSet} ;


////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct SourceFile {
    pub file : String,
    pub lines : Vec<String>
}

impl SourceFile {


    pub fn new_error(file: &str) -> error::Result<Self> {
        let ret = Self::new(file);

        if ret.is_err() {
            info!("Cannot load source {}", file);
        }

        ret
    }

    pub fn num_of_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn get_line(&self, line : usize) -> Option<&String> {
        if line >= self.lines.len() {
            None
        } else {
            Some(&self.lines[line])
        }
    }

    pub fn new( file : &str ) -> error::Result<Self> {
        
        let f = File::open(file)?;
        let f = BufReader::new(f);

        let lines : Result<Vec<_>, _> = f.lines().collect();

        let ret = Self {
            file : file.to_string(),
            lines : lines?
        };

        info!("loaded sourcefile : {} ", file);

        Ok(ret)
    }

}

#[derive(Clone, Debug)]
pub struct SourceLine {
    pub loc : Location,
    pub addr : Option<u16>,
    pub line : Option<String>,
}

#[derive(Clone, Debug)]
pub struct AnnotatedSourceFile {
    pub lines : Vec<SourceLine>
}

impl AnnotatedSourceFile {
    pub fn line(&self, line : usize) -> Option<&SourceLine> {
        self.lines.get(line)
    }

    pub fn num_of_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn text_line(&self, line : usize) -> Option<&String> {
        self.lines.get(line).map(|v| v.line.as_ref()).flatten()
    }
    pub fn text_line_string(&self, line : usize) -> String {
        self.text_line(line).cloned().unwrap_or(String::new())
    }
}

pub struct SourceStore {
    annotated_files: HashMap<String, AnnotatedSourceFile>,
    source_dir : String,
    loc_to_addr : HashMap<Location, u16>,
    addr_to_loc : HashMap<u16, Location>,
}

impl SourceStore {

    pub fn addr_to_source_line(&self, addr : u16) -> Option<&SourceLine> {
        let loc = self.addr_to_loc.get(&addr)?;
        self.loc_to_source_line(loc)
    }

    pub fn loc_to_source_line(&self, _loc : &Location) -> Option<&SourceLine> {
        let annotated_file = self.get(&_loc.file)?;
        annotated_file.lines.get(_loc.get_line_number())
    }

    pub fn new(source_dir : &str, chunks : &[Chunk]) -> Self {

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

            if let Ok(sf) = SourceFile::new_error(&key) {
                let lines =
                    sf.lines.iter().enumerate()
                    .map(|(i,line)| {
                        let loc = Location::new(file, i);
                        let addr = loc_to_addr.get(&loc).cloned();
                        SourceLine {
                            loc,addr, line :Some(line.clone()),
                        }}).collect();

                let annotated_source = AnnotatedSourceFile { lines };
                info!("Annotated source file {}", &file);
                annotated_files.insert(key,annotated_source);
            } else {
                warn!("Can't annotate source file {}", file);
            }
        }

        Self {
            loc_to_addr,
            addr_to_loc,
            source_dir : source_dir.to_string(),
            annotated_files,
        }
    }

    fn make_key_source_dir(source_dir : &str, file : &str) -> String {
        format!("{}/{}", source_dir, file)
    }

    fn make_key(&self, file : &str) -> String {
        Self::make_key_source_dir(&self.source_dir, file)
    }

    pub fn get(&self, file : &str) -> Option<&AnnotatedSourceFile> {
        let key = self.make_key(file);
        self.annotated_files.get(&key)
    }

    pub fn loc_to_addr(&self, loc : &Location) -> Option<u16> {
        self.loc_to_addr.get(loc).cloned()
    }

    pub fn addr_to_loc(&self, _addr : u16) -> Option<&Location> {
        self.addr_to_loc.get(&_addr)
    }

    pub fn get_line(&self, _loc : &Location) -> Option<&String> {
        panic!()
    }
}
