include!(concat!(env!("OUT_DIR"), "/helptext.rs"));

use lazy_static::lazy_static;

lazy_static!{
    pub static ref HELP : Err = Err::new();
}

impl std::fmt::Display for ErrCode {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!()
    }
}

