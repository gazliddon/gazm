use emu6809::isa::Dbase;

lazy_static::lazy_static! {
    pub static ref ISA_DBASE : Dbase = Dbase::new();
}
