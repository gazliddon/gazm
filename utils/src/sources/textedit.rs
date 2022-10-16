#[derive(Clone, Debug)]
pub struct TextPos {
    pub line: usize,
    pub character: usize,
}


impl TextPos {
    pub fn new(line: usize, character: usize) -> Self {
        Self { line, character }
    }
}

/// Contains information for an edit to the in memrory text file
/// start..end is half open, end = the character after the last char to edit
pub struct TextEdit<'a> {
    pub start: TextPos,
    pub end: TextPos,
    pub text: &'a str,
}

impl<'a> TextEdit<'a> {
    pub fn new(start: TextPos, end: TextPos, text: &'a str) -> Self {
        Self { start, end, text }
    }
}


use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum EditErrorKind {
    #[error("Index out of range, asked for {0}, file size is {1}")]
    IndexOutOfRange(usize, usize),
    #[error("Character out of range: requesed {0}, line length is {1}")]
    CharacterOutOfRange(usize, usize),
    #[error("Line out of range: requested {0}, num of lines {1}")]
    LineOutOfRange(usize, usize),
}

pub type EditResult<T> = Result<T, EditErrorKind>;

pub trait TextEditTrait {
    fn edit(&mut self, _edit: &TextEdit) -> EditResult<()>;
    fn get_line(&self, _line_number: usize) -> EditResult<&str>;
    fn num_of_lines(&self) -> usize;

    fn delete_line(&mut self, line_number: usize) -> EditResult<()> {
        self.replace_line(line_number, "")
    }

    fn replace_line(&mut self, line_number: usize, txt: &str) -> EditResult<()> {
        let start = TextPos::new(line_number, 0);
        let end = TextPos::new(line_number + 1, 0);
        let text_edit = TextEdit::new(start, end, txt);
        self.edit(&text_edit)
    }

    fn insert_line(&mut self, line_number: usize, txt: &str) -> EditResult<()> {
        let txt = &format!("{}\n", txt);
        let start = TextPos::new(line_number, 0);
        let end = TextPos::new(line_number, 0);
        let text_edit = TextEdit::new(start, end, txt);
        self.edit(&text_edit)
    }
}

fn mk_offsets(source: &str) -> Vec<std::ops::Range<usize>> {
    source.lines().map(|x| get_range(source, x)).collect()
}

fn get_range(whole_buffer: &str, part: &str) -> std::ops::Range<usize> {
    let start = part.as_ptr() as usize - whole_buffer.as_ptr() as usize;
    let end = start + part.len();
    start..end
}

#[derive(Clone)]
pub struct TextFile {
    pub source: String,
    pub line_offsets: Vec<std::ops::Range<usize>>,
    pub hash: String,
}

impl std::fmt::Debug for TextFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.source)
    }
}
impl std::fmt::Display for TextFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.source)
    }
}

impl TextEditTrait for TextFile {
    fn edit(&mut self, edit: &TextEdit) -> EditResult<()> {
        let r = self.get_range(edit)?;
        let first = &self.source[..r.start];
        let last = &self.source[r.end..];
        let new_source = first.to_owned() + &edit.text + last;
        self.source = new_source;
        self.post_change();
        Ok(())
    }

    fn get_line(&self, line_number: usize) -> EditResult<&str> {
        self.line_offsets
            .get(line_number)
            .map(|r| &self.source[r.clone()])
            .ok_or_else(|| EditErrorKind::LineOutOfRange(line_number, self.num_of_lines()))
    }

    fn num_of_lines(&self) -> usize {
        self.line_offsets.len()
    }
}


impl TextFile {
    pub fn new(txt: &str) -> Self {
        let mut ret = Self {
            source: txt.to_string(),
            line_offsets: Default::default(),
            hash: Default::default(),
        };

        ret.post_change();
        ret
    }

    pub fn get_hash(&self) -> &String {
        &self.hash
    }

    fn post_change(&mut self) {
        self.line_offsets = mk_offsets(&self.source);
        self.rehash();
    }

    fn rehash(&mut self) {
        self.hash = crate::hash::get_hash_from_str(&self.source)
    }

    fn start_pos_to_index(&self, line: usize, character: usize) -> EditResult<usize> {
        if line >= self.num_of_lines() {
            return Err(EditErrorKind::LineOutOfRange(line, self.num_of_lines()));
        }

        let line_range = &self.line_offsets[line];
        let ret = line_range.start + character;

        if !line_range.contains(&ret) {
            return Err(EditErrorKind::CharacterOutOfRange(
                character,
                line_range.len(),
            ));
        }

        if ret >= self.source.len() {
            return Err(EditErrorKind::IndexOutOfRange(ret, self.source.len()));
        }

        Ok(ret)
    }

    fn end_pos_to_index(&self, line: usize, character: usize) -> EditResult<usize> {
        if line == self.num_of_lines() && character == 0 {
            return Ok(self.source.len());
        }

        if line > self.num_of_lines() {
            return Err(EditErrorKind::LineOutOfRange(line, self.num_of_lines()));
        }

        let line_range = &self.line_offsets[line];

        if character > line_range.len() {
            return Err(EditErrorKind::CharacterOutOfRange(
                character,
                line_range.len(),
            ));
        }

        Ok(line_range.start + character)
    }

    fn get_range(&self, edit: &TextEdit) -> EditResult<std::ops::Range<usize>> {
        let start_index = self.start_pos_to_index(edit.start.line, edit.start.character)?;
        let end_index = self.end_pos_to_index(edit.end.line, edit.end.character)?;
        assert!(start_index <= end_index);
        Ok(start_index..end_index)
    }
}

#[allow(unused_imports)]
mod test {
    const TEST_TEXT: &str = include_str!("../../assets/test.txt");

    use super::*;
    use lazy_static::lazy_static;
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    pub fn test_edit() {
        let mut text_file = TextFile::new(TEST_TEXT);

        assert_eq!(5, text_file.num_of_lines());

        // Line 0, the word 'one'
        let start = TextPos::new(0, 19);
        let end = TextPos::new(0, 22);
        let edit = TextEdit::new(start, end, "hello");

        text_file.edit(&edit).unwrap();
        assert_eq!("Hello this is line hello", text_file.get_line(0).unwrap());

        text_file.delete_line(0).unwrap();
        assert_eq!("This is line two", text_file.get_line(0).unwrap());
        assert_eq!(4, text_file.num_of_lines());

        let start = TextPos::new(3, 0);
        let end = TextPos::new(4, 0);
        let edit = TextEdit::new(start, end, "6809 rulez");
        text_file.edit(&edit).unwrap();
        assert_eq!("6809 rulez", text_file.get_line(3).unwrap());
    }
}
