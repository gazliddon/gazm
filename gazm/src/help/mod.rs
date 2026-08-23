include!(concat!(env!("OUT_DIR"), "/helptext.rs"));

use std::sync::LazyLock;

pub static HELP: LazyLock<Err> = LazyLock::new(Err::new);

impl std::fmt::Display for ErrCode {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!()
    }
}
