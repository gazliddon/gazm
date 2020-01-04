use super::chunk::{ Chunk, Location };
pub use quick_error::ResultExt;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Collison(a : Chunk, b : Chunk)

        Io(location : Location, err : std::io::Error) {
            context(loc : &'a Location, err: std::io::Error)
                -> (loc.clone(), err)
        }

        Parsing(location : Location, text : String) {
            context(loc : &'a Location, err: std::num::ParseIntError)
                -> (loc.clone(), err.to_string())
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

