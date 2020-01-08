use super::chunk::{ Chunk, Location };

use super::error;
// use error::{ResultExt};

// use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader};

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

pub struct SourceStore {
    files : HashMap<String, SourceFile>,
    source_dir : String
}

impl SourceStore {

    pub fn new(source_dir : &str) -> Self {
        Self {
            files: HashMap::new(),
            source_dir : source_dir.to_string()
        }
    }
    fn make_key(&self, file : &str) -> String {
        format!("{}/{}", self.source_dir, file)
    }

    pub fn get(&mut self, file : &str) -> error::Result<&SourceFile> {
        let key = self.make_key(file);

        if !self.files.contains_key(&key) {
            let source_file = SourceFile::new(&key)?;
            self.files.insert(key.clone(), source_file);
        }

        Ok( self.files.get(&key).unwrap() )
    }

    pub fn get_line(&mut self, loc : &Location) -> error::Result<&String> {
        let source_file = self.get(loc.file.to_str().unwrap())?;

        if let Some(line) = source_file.line(loc.line_number) {
            Ok(line)
        } else {
            Err(error::Error::Misc("Can't find line".to_string()))
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Symbol {
    name : String,
    value : u16,
}

fn make_rom(chunks : &[Chunk]) -> error::Result<( RomData, Vec<Option<Location>> )> {
    let mut used : Vec<Option<&Chunk>> = vec![None;0x10_000];

    let mut rom : RomData = [0;0x10_000];

    for c in chunks {
        for addr in c.addr_range() {
            if let Some(cref) = used [addr] {
                return Err(error::Error::Collison( c.clone(), cref.clone()));
            } else {
                used[ addr ] = Some(c);
                rom[addr] = c.data[addr - c.addr as usize];
            }
        }
    }

    let addr_to_loc = used
        .into_iter()
        .map(|c| c.map(|v| v.location.clone()));

    Ok(( rom, addr_to_loc.collect() ))
}

pub type RomData = [u8;0x10_000];

pub struct Rom {
    pub data : RomData,
    chunks : Vec<Chunk>,
    addr_to_loc : Vec<Option<Location>>,
    symbols : std::collections::HashMap<String,Symbol>,

    pub sources : SourceStore,
}

impl Rom {

    pub fn get_source_location(&self, _addr : u16) -> Option<Location> {
        self.addr_to_loc[_addr as usize].as_ref().cloned()
    }

    pub fn get_source_line(&mut self, _addr : u16) -> error::Result<&String> {

        if let Some(loc) = self.get_source_location(_addr) {
            self.sources.get_line(&loc)
        } else {
            Err(error::Error::Misc("whoops".to_string()))
        }

    }

    pub fn add_symbol(&mut self, name : &str, value : u16) {
        let name = name.to_string();
        if self.symbols.get(&name).is_some() {
            // TODO fix this
            panic!("Duplicate symble!")
        }

        self.symbols.insert(name.clone(), Symbol {name, value});
    }

    pub fn get_symbol(&self, name : &str) -> Option< ( &Symbol, Option<Location> ) > {
        if let Some(sym) = self.symbols.get(name) {
            let loc = self.get_source_location(sym.value);
            Some(( sym, loc))
        } else {
            None
        }
    }

    pub fn get_slice(&self, addr : u16, size : u16) -> &[u8]  {
        let addr = addr as usize;
        let size = size as usize;

        if addr + (size -1 ) > 0x10_000 {
            panic!("FUCKED")
        }

        &self.data[addr..(addr+size)]
    }


    pub fn from_chunks( chunks : Vec<Chunk> ) -> error::Result<Self> {
        let (data, addr_to_loc) = make_rom(&chunks)?;

        let rom = Rom {
            chunks,
            data,
            addr_to_loc,
            symbols : std::collections::HashMap::new(),
            sources : SourceStore::new("asm")
        };

        Ok(rom)
    }
}


