
impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}:{})", self.line, self.col)
    }
}

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

impl Position {
    pub fn new(line : usize, col: usize, range: std::ops::Range<usize>, src : AsmSource) -> Self {
        Self {line,col, range, src }
    }
}

impl<'a> Default for Position {
    fn default() -> Self {
        Self {
            line : 0,
            col: 0,
            range : 0..0,
            src: AsmSource::FromStr,
        }
    }
}
