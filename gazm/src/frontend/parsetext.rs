use grl_sources::SourceFile; 

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParseText<'a> {
    pub start: usize,
    pub len: usize,
    pub source_file: &'a SourceFile,
}

impl<'a> AsRef<str> for ParseText<'a> {
    fn as_ref(&self) -> &str {
        &self.source_file.source.source[self.as_range()]
    }
}

impl<'a> ParseText<'a> {
    pub fn new(source_file: &'a SourceFile, range: std::ops::Range<usize>, ) -> Self {
        Self {
            start: range.start,
            len: range.len(),
            source_file,
        }
    }
}

impl<'a> ParseText<'a> {
    pub fn get_text(&self) -> &str {
        self.as_ref()
    }

    pub fn as_range(&self) -> std::ops::Range<usize> {
        self.start..self.start + self.len
    }

    pub fn as_text_pos(&self) -> grl_sources::TextPos {
        self.source_file.source.offset_to_text_pos(self.start).unwrap()
    }
}
