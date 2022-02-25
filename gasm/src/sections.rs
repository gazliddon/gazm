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

    /// Does this section overlap with other
    pub fn overlaps(&self, other : &Self) -> bool {
        // TODO need to make sure we don't have any zero length sections
        let ostart = other.logical_range.start;
        let oend = other.logical_range.end;
        let olast = ( ostart + oend ) - 1;

        self.logical_range.contains(&ostart) ||
        self.logical_range.contains(&olast)
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
        self.get_current_section_binary().unwrap()
    }
}

impl DerefMut for Sections {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_current_section_binary_mut().unwrap()
    }
}

use romloader::ResultExt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SectionError {
    #[error("No section selected as current")]
    NoCurrentSection,
    #[error("Org {org:04X} does not fit in current section")]
    OrgOutsideSectionBounds {bounds : std::ops::Range<usize>, org: usize},
    #[error("Section name already in use")]
    SectionNameInUse(SectionDescriptor),
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

    pub fn set_current_section(&mut self, name : &str) -> Result<(), SectionError>{
        let x = self.name_to_section.get(name).ok_or(
            SectionError::UknownSectionName(name.to_string())
            )?;
        self.current_section = Some(*x);
        Ok(())
    }
    /// Get the current section
    pub fn get_current_section(&self) -> Result<&(SectionDescriptor, Binary ), SectionError> {
        self.current_section.map(|idx|
            &self.sections[idx]).ok_or(SectionError::NoCurrentSection)
    }

    /// Get a mutable version of the current section
    pub fn get_current_section_mut(&mut self) -> Result<&mut (SectionDescriptor, Binary ), SectionError> {
        self.current_section.map(|idx|
            &mut self.sections[idx]).ok_or(SectionError::NoCurrentSection)
    }

    /// Get the current section's binary
    pub fn get_current_section_binary(&self) -> Result<&Binary, SectionError> {
        self.get_current_section().map(|(_,b)| b)
    }

    /// Get a mutable version of the current section's binary chunk
    pub fn get_current_section_binary_mut(&mut self) -> Result<&mut Binary, SectionError> {
        self.get_current_section_mut().map(|(_,b)| b)
    }

    /// org to a logical address within the current section
    pub fn logical_org(&mut self, addr : usize) -> Result<(), SectionError>{
        let b = self.get_current_section_binary_mut()?;
        let offset = b.get_write_offset();
        b.set_write_address(addr, offset);
        Ok(())
    }

    fn get_section(&self,  name : &str) -> Option<&(SectionDescriptor, Binary)> {
        self.name_to_section.get(name).map(|id| &self.sections[*id])
    }

    /// Add a section
    pub fn add_section(&mut self, name : &str, logical_range: std::ops::Range<usize>, physical_base : usize, access_type : AccessType) -> Result<usize,SectionError> {
        // check for section already existing
        if let Some((s,_)) = self.get_section(name) {
                return Err(SectionError::SectionNameInUse(s.clone()))
        }
        
        // check for overlaps
        let section = SectionDescriptor::new(name , logical_range, physical_base,  access_type);

        for (s,_) in &self.sections {
            if s.overlaps(&section) {
                return Err(SectionError::OverlapsWithExistingSection(s.clone()))
            }
        }

        let binary = section.make_binary();
        let id = self.sections.len();

        self.sections.push((section,binary));
        self.name_to_section.insert(name.to_string(), id);

        Ok(id)
    }
}
