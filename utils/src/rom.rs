
use super::chunk::{ Chunk, Location };

use super::error;
use crate::sourcestore::SourceStore;

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
        .map(|c| c.map(|v| v.location.clone()))
        .collect();

    Ok(( rom, addr_to_loc))
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

    pub fn get_source_location(&self, _addr : u16) -> Option<&Location> {
        self.addr_to_loc[_addr as usize].as_ref()
    }

    pub fn get_source_line(&self, _addr : u16) -> Option<String> {
        self
            .get_source_location(_addr)
            .map(|loc| self.sources.get_line(&loc))
            .unwrap_or(None)
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
        self.symbols.get(name).map(|sym| {
            let loc = self.get_source_location(sym.value).cloned();
            ( sym, loc) })
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


