#[derive(Clone, PartialEq,Debug, Copy)]
pub enum AsmSource {
    FromStr,
    FileId(u64)
}

impl Default for AsmSource {
    fn default() -> Self {
        AsmSource::FromStr
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    pub line : usize,
    pub col: usize,
    pub range: std::ops::Range<usize>,
    pub src : AsmSource,
}
