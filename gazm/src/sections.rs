#![forbid(unused_imports)]
use crate::assembler::AccessType;

use serde::{Deserialize, Serialize};

/// A named memory region established by the in-asm `section` directive.
///
/// The sizer tracks these during the layout walk and persists the final
/// descriptors into the v4 metadata header (`TargetInfo.sections`), giving
/// the debugger named regions and read-only/read-write hints.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SectionDescriptor {
    pub name: String,
    pub logical_range: std::ops::Range<usize>,
    pub physical_range: std::ops::Range<usize>,
    pub access_type: AccessType,
}

impl SectionDescriptor {
    pub fn new(
        name: &str,
        logical_range: std::ops::Range<usize>,
        physical_range: std::ops::Range<usize>,
        access_type: AccessType,
    ) -> Self {
        Self {
            name: name.to_string(),
            logical_range,
            physical_range,
            access_type,
        }
    }
}
