use std::fs::{File,self};
use std::path::Path;
use std::io::Read;

use std::sync::{ Mutex,Arc };

pub fn get_file_as_byte_vec<P: AsRef<Path>>(filename: P) -> Result<Vec<u8>, std::io::Error> {
    let mut f = File::open(&filename)?;
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    let _read = f.read(&mut buffer)?;
    Ok(buffer)
}

pub fn with_state<R, S>(data: &Arc<Mutex<S>>, f: impl FnOnce(&mut S) -> R) -> R {
    let state = &mut data.lock().expect("Could not lock mutex");
    f(state)
}



