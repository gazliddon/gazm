use emuz80::isa::Dbase;
use std::sync::LazyLock;

pub static ISA_DBASE: LazyLock<Dbase> = LazyLock::new(Dbase::new);
