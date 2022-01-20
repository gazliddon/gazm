
use super::chunk::Chunk;

use super::location::Location;

use super::error;
use crate::sourcestore::SourceStore;

////////////////////////////////////////////////////////////////////////////////
pub struct Symbol {
    name : String,
    value : u16,
}

pub type RomData = [u8;0x1_0000];

use std::collections::HashMap;

pub struct Rom {
    pub data : RomData,
    chunks : Vec<Chunk>,
    addr_to_loc : Vec<Option<Location>>,
    location_to_addr_range: HashMap<Location,std::ops::Range<usize>>,

    pub sources : SourceStore,
}

impl Rom {

    fn get_location_addr_range(&self, loc : &Location) -> Option<&std::ops::Range<usize>> {
        self.location_to_addr_range.get(loc)
    }

    pub fn get_source_location(&self, _addr : u16) -> Option<&Location> {
        self.addr_to_loc[_addr as usize].as_ref()
    }

    pub fn get_source_line(&self, _addr : u16) -> Option<&String> {
        self
            .get_source_location(_addr)
            .and_then(|loc| self.sources.get_line(loc))
    }

    pub fn get_slice(&self, addr : u16, size : usize) -> &[u8]  {
        let addr = addr as usize;

        if addr + (size -1 ) > 0x1_0000 {
            panic!("FUCKED")
        }

        &self.data[addr..(addr+size)]
    }

    pub fn from_chunks( chunks : Vec<Chunk> ) -> error::Result<Self> {

        let mut used : Vec<Option<&Chunk>> = vec![None;0x1_0000];
        let mut data : RomData = [0;0x1_0000];

        let mut location_to_addr_range = HashMap::new();

        for c in &chunks {
            location_to_addr_range.insert(c.location.clone(),c.addr_range());

            for addr in c.addr_range() {
                if let Some(cref) = used [addr] {
                    return Err(error::Error::Collison( c.clone(), cref.clone()));
                } else {
                    used[ addr ] = Some(c);
                    data[addr] = c.data[addr - c.addr as usize];
                }
            }
        }

        let addr_to_loc = used
            .into_iter()
            .map(|c| c.cloned().map(|v| v.location))
            .collect();

        let sources = SourceStore::new("asm", &chunks);

        let rom = Rom {
            chunks,
            data, 
            addr_to_loc ,
            location_to_addr_range,
            sources 
        };

        Ok(rom)
    }
}


