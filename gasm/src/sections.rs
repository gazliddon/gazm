use crate::binary::*;

use std::{collections::HashMap, ops::Deref, ops::DerefMut};

#[derive(Debug, Clone)]
pub struct SectionDescriptor {
    name: String,
    logical_range: std::ops::Range<usize>,
    physical_range: std::ops::Range<usize>,
    access_type : AccessType,
}
impl SectionDescriptor {
    pub fn new(name : &str, logical_range: std::ops::Range<usize>, physical_base : usize, access_type : AccessType) -> Self {
        let physical_range = physical_base..physical_base+logical_range.len();
        Self { name: name.to_string(), logical_range, physical_range, access_type }
    }

    fn get_offset(&self) -> isize {
        let p_start = self.physical_range.start as isize;
        let l_start = self.logical_range.start as isize;
        p_start - l_start
    }

    pub fn make_binary(&self) -> Binary {
        let mut ret = Binary::new(self.logical_range.len(), self.access_type.clone());
        ret.set_write_offset(self.get_offset());
        ret
    }
}

struct Sections {
    current_section: Option<usize>,
    sections : Vec<(SectionDescriptor, Binary )>,
    name_to_section : HashMap<String,usize>,
}

impl Deref for Sections {
    type Target = Binary;
    fn deref(&self) -> &Self::Target {
        self.get_current_section().unwrap()
    }
}

impl DerefMut for Sections {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_current_section_mut().unwrap()
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SectionError {
    #[error("No section selected as current")]
    NoCurrentSection,
    #[error("Org {org:04X} does not fit in current section")]
    OrgOutsideSectionBounds {bounds : std::ops::Range<usize>, org: usize},
    #[error("Section name already in use")]
    SectionNameInUse,
    #[error("Overlaps with section {0:?}")]
    OverlapsWithExistingSection(SectionDescriptor),
    #[error("Can't find a section with name {0}")]
    UknownSectionName(String),
}

impl Sections {
    pub fn new() -> Self {
        Sections {
            current_section : None,
            sections: vec![],
            name_to_section: Default::default(),
        }
    }

    pub fn set_current_section(&mut self, name : &str) {
        // TODO add error
        let x = self.name_to_section.get(name).unwrap();
        self.current_section = Some(*x)
    }

    pub fn get_current_section(&self) -> Option<&Binary> {
        self.current_section.map(|idx|
            &self.sections[idx].1)
    }

    pub fn get_current_section_mut(&mut self) -> Option<&mut Binary> {
        self.current_section.map(|idx|
            &mut self.sections[idx].1)
    }

    // org to a logical address within the current section
    pub fn logical_org(&mut self, addr : usize) {
        // TODO add error condition
        let b = self.get_current_section_mut().unwrap();
        let offset = b.get_write_offset();
        b.set_write_address(addr, offset);
    }

    pub fn add_section(&mut self, name : &str, logical_range: std::ops::Range<usize>, physical_base : usize, access_type : AccessType) -> usize {
        // TODO check for overlaps
        // TODO check for section already existing
        // TODO add error

        let section = SectionDescriptor::new(name , logical_range, physical_base,  access_type);
        let binary = section.make_binary();
        let id = self.sections.len();

        self.sections.push((section,binary));
        self.name_to_section.insert(name.to_string(), id);
        id
    }
}
