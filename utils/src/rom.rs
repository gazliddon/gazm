use super::chunk::{ Chunk, Location };
use super::error;
use std::path::PathBuf;

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
    pub source_file_path : PathBuf,
    
    symbols : std::collections::HashMap<String,Symbol>,
}

impl Rom {
    pub fn get_source_location(&self, _addr : u16) -> Option<&Location> {
        self.addr_to_loc[_addr as usize].as_ref()
    }

    pub fn add_symbol(&mut self, name : &str, value : u16) {
        let name = name.to_string();
        self.symbols.insert(name.clone(), Symbol {name, value});
    }

    pub fn get_symbol(&self, name : &str) -> Option< ( &Symbol, Option<&Location> ) > {
        if let Some(sym) = self.symbols.get(name) {
            let loc = self.get_source_location(sym.value);
            Some(( sym, loc))
        } else {
            None
        }
    }

    pub fn from_chunks( chunks : Vec<Chunk> ) -> error::Result<Self> {
        let (data, addr_to_loc) = make_rom(&chunks)?;

        let rom = Rom {
            chunks,
            data,
            addr_to_loc,
            source_file_path : PathBuf::from("."),
            symbols : std::collections::HashMap::new()
        };

        Ok(rom)
    }
}


