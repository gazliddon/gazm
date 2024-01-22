#![forbid(unused_imports)]

use grl_sources::{EditErrorKind, EditResult, TextEditTrait};

use super::{Assembler, AssemblerCpuTrait};
use crate::error::GResult;

use std::path::Path;

impl<C> Assembler<C>
where
    C: AssemblerCpuTrait,
{
    pub fn edit_source_file<P: AsRef<Path>, X>(
        &mut self,
        file: P,
        f: impl FnOnce(&mut dyn TextEditTrait) -> EditResult<X>,
    ) -> EditResult<X> {
        if let Ok(source) = self.sources_mut().get_source_mut(&file) {
            let old_hash = source.get_text().get_hash().clone();

            let res = f(source.get_text_mut())?;

            // Invalidate token cache if needed
            let new_hash = source.get_text().get_hash().clone();

            if new_hash != old_hash {
                self.get_token_store_mut().invalidate_tokens(&file);
            }
            Ok(res)
        } else {
            Err(EditErrorKind::NoSourceFile(
                file.as_ref().to_string_lossy().into(),
            ))
        }
    }

    pub fn replace_file_contents<P: AsRef<Path>>(
        &mut self,
        file: P,
        new_text: &str,
    ) -> GResult<()> {
        Ok(self.edit_source_file(&file, |editable| editable.replace_file(new_text))?)
    }
}
