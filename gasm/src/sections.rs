use crate::binary::*;

use std::{collections::HashMap, ops::Deref, ops::DerefMut, thread::AccessError, vec};

use serde::{Deserialize, Serialize};

fn default_access_type() -> AccessType {
    AccessType::ReadWrite
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct SerializedSection {
    name: String,
    start: usize,
    size: usize,
    org: Option<usize>,
    #[serde(default = "default_access_type")]
    access: AccessType,
}

impl From<SerializedSection> for SectionDescriptor {
    fn from(x: SerializedSection) -> Self {
        let logical_range = x.start..x.start + x.size;
        let physical_range = if let Some(org) = x.org {
            org..org + x.size
        } else {
            logical_range.clone()
        };

        SectionDescriptor {
            name: x.name,
            logical_range,
            physical_range,
            access_type: x.access,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SectionDescriptor {
    name: String,
    logical_range: std::ops::Range<usize>,
    physical_range: std::ops::Range<usize>,
    access_type: AccessType,
}

struct Section {
    descriptor: SectionDescriptor,
    binary: Binary,
}

impl From<SectionDescriptor> for Section {
    fn from(descriptor: SectionDescriptor) -> Self {
        Self {
            binary: make_binary(&descriptor),
            descriptor,
        }
    }
}

pub fn make_binary(desc: &SectionDescriptor) -> Binary {
    let p_start = desc.physical_range.start as isize;
    let l_start = desc.logical_range.start as isize;
    let offset = p_start - l_start;
    let mut ret = Binary::new(desc.logical_range.len(), desc.access_type.clone());
    ret.set_write_offset(offset);
    ret
}

impl Section {
    fn new(
        name: &str,
        logical_range: std::ops::Range<usize>,
        physical_base: usize,
        access_type: AccessType,
    ) -> Self {
        let desc = SectionDescriptor::new(name, logical_range, physical_base, access_type);
        desc.into()
    }

    /// Does this section overlap with other
    pub fn overlaps(&self, other: &Self) -> bool {
        let ostart = other.descriptor.logical_range.start;
        let oend = other.descriptor.logical_range.end;
        let olast = (ostart + oend) - 1;
        self.descriptor.logical_range.contains(&ostart)
            || self.descriptor.logical_range.contains(&olast)
    }
}

impl SectionDescriptor {
    /// Create a section descritor, private to this module
    fn new(
        name: &str,
        logical_range: std::ops::Range<usize>,
        physical_base: usize,
        access_type: AccessType,
    ) -> Self {
        assert!(logical_range.len() > 0);
        let physical_range = physical_base..physical_base + logical_range.len();
        Self {
            name: name.to_string(),
            logical_range,
            physical_range,
            access_type,
        }
    }
}

struct Sections {
    current_section: Option<usize>,
    sections: Vec<Section>,
    name_to_section: HashMap<String, usize>,
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
    #[error("Sections cannot be zero in size")]
    ZeroSizedSection,
    #[error("No section selected as current")]
    NoCurrentSection,
    #[error("Org {org:04X} does not fit in current section")]
    OrgOutsideSectionBounds {
        bounds: std::ops::Range<usize>,
        org: usize,
    },
    #[error("Section name already in use")]
    SectionNameInUse(SectionDescriptor),
    #[error("Overlaps with section {0:?}")]
    OverlapsWithExistingSection(SectionDescriptor),
    #[error("Can't find a section with name {0}")]
    UknownSectionName(String),
    #[error("Error loading section file {0}")]
    ErrorLoadingSectionFile(String),
}

impl Sections {
    pub fn new() -> Self {
        Sections {
            current_section: None,
            sections: vec![],
            name_to_section: Default::default(),
        }
    }

    pub fn from_file(file: &str) -> Result<Sections, SectionError> {
        use std::fs::read_to_string;

        let x = read_to_string(file)
            .map_err(|_| SectionError::ErrorLoadingSectionFile(file.to_string()))?;

        let s: Vec<SerializedSection> = serde_yaml::from_str(&x)
            .map_err(|_| SectionError::ErrorLoadingSectionFile(file.to_string()))?;

        let s2: Vec<SectionDescriptor> = s.into_iter().map(|x| x.into()).collect();
        let mut ret = Self::new();
        for s in s2 {
            ret.add_section_descriptor(s).unwrap();
        }
        Ok(ret)
    }

    fn add_section_descriptor(
        &mut self,
        section: SectionDescriptor,
    ) -> Result<usize, SectionError> {
        let name = section.name.clone();
        let section = section.into();

        for s in &self.sections {
            if s.overlaps(&section) {
                return Err(SectionError::OverlapsWithExistingSection(
                    s.descriptor.clone(),
                ));
            }
        }

        let id = self.sections.len();

        self.sections.push(section);
        self.name_to_section.insert(name.clone(), id);

        Ok(id)
    }

    pub fn set_current_section(&mut self, name: &str) -> Result<(), SectionError> {
        let x = self
            .name_to_section
            .get(name)
            .ok_or(SectionError::UknownSectionName(name.to_string()))?;
        self.current_section = Some(*x);
        Ok(())
    }
    /// Get the current section
    pub fn get_current_section(&self) -> Result<&Section, SectionError> {
        self.current_section
            .map(|idx| &self.sections[idx])
            .ok_or(SectionError::NoCurrentSection)
    }

    /// Get a mutable version of the current section
    pub fn get_current_section_mut(&mut self) -> Result<&mut Section, SectionError> {
        self.current_section
            .map(|idx| &mut self.sections[idx])
            .ok_or(SectionError::NoCurrentSection)
    }

    /// Get the current section's binary
    pub fn get_current_section_binary(&self) -> Result<&Binary, SectionError> {
        self.get_current_section().map(|s| &s.binary)
    }

    /// Get a mutable version of the current section's binary chunk
    pub fn get_current_section_binary_mut(&mut self) -> Result<&mut Binary, SectionError> {
        self.get_current_section_mut().map(|s| &mut s.binary)
    }

    /// org to a logical address within the current section
    pub fn logical_org(&mut self, addr: usize) -> Result<(), SectionError> {
        let b = self.get_current_section_binary_mut()?;
        let offset = b.get_write_offset();
        b.set_write_address(addr, offset);
        Ok(())
    }

    fn get_section(&self, name: &str) -> Option<&Section> {
        self.name_to_section.get(name).map(|id| &self.sections[*id])
    }

    /// Add a section
    pub fn add_section(
        &mut self,
        name: &str,
        logical_range: std::ops::Range<usize>,
        physical_base: usize,
        access_type: AccessType,
    ) -> Result<usize, SectionError> {
        // Check to make sure the size is > 0
        if logical_range.len() < 1 {
            return Err(SectionError::ZeroSizedSection);
        }

        // check for section already existing
        if let Some(s) = self.get_section(name) {
            return Err(SectionError::SectionNameInUse(s.descriptor.clone()));
        }

        // check for overlaps
        let section = SectionDescriptor::new(name, logical_range, physical_base, access_type);
        let section = section.into();

        for s in &self.sections {
            if s.overlaps(&section) {
                return Err(SectionError::OverlapsWithExistingSection(
                    s.descriptor.clone(),
                ));
            }
        }

        let id = self.sections.len();

        self.sections.push(section);
        self.name_to_section.insert(name.to_string(), id);

        Ok(id)
    }
}
