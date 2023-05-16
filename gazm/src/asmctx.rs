use crate::ast::{AstNodeId, AstNodeRef, AstTree};
use crate::error::GResult;
use crate::item::Item;
use crate::{binary, fixerupper::FixerUpper};
use emu::utils::sources::{self, BinToWrite, Position, SymbolScopeId};
use sources::fileloader::{FileIo, SourceFileLoader};
use sources::{ SymbolError, };
use std::path::{Path, PathBuf};

use crate::ctx::Context;

pub struct AsmCtx<'a> {
    pub ctx: &'a mut Context,
    pub fixer_upper: FixerUpper,
}

impl<'a> AsmCtx<'a> {
    pub fn set_exec_addr(&mut self, addr: usize) {
        self.ctx.asm_out.exec_addr = Some(addr);
    }

    pub fn binary(&self) -> &binary::Binary {
        &self.ctx.asm_out.binary
    }

    pub fn binary_mut(&mut self) -> &mut binary::Binary {
        &mut self.ctx.asm_out.binary
    }

    pub fn add_fixup<I: Into<Item>>(&mut self, id: AstNodeId, v: I, scope_id: u64) -> (u64, AstNodeId) {
        self.fixer_upper.add_fixup(scope_id, id, v.into());
        (scope_id, id)
    }

    pub fn get_fixup_or_default(&self, id: AstNodeId, i: &Item, scope_id: u64) -> Item {
        self.fixer_upper.get_fixup_or_default(scope_id, id, i)
    }

    pub fn set_dp(&mut self, dp: i64) {
        if dp < 0 {
            self.ctx.asm_out.direct_page = None;
        } else {
            self.ctx.asm_out.direct_page = Some(dp as u64 as u8);
        }
    }

    pub fn set_symbol_value(
        &mut self,
        symbol_id: SymbolScopeId,
        val: usize,
    ) -> Result<(), SymbolError> {
        self.ctx.get_symbols_mut().set_symbol_from_id(symbol_id, val as i64)
    }

    pub fn add_bin_to_write<P: AsRef<Path>>(
        &mut self,
        path: P,
        range: std::ops::Range<usize>,
    ) -> GResult<PathBuf> {
        let physical_address = range.start;
        let count = range.len();

        let data = self
            .ctx
            .asm_out
            .binary
            .get_bytes(physical_address, count)?.to_vec();

        let path = self.get_abs_path(path);
        // Save a record of the file Written
        // this goes into the written sym file eventually

        let bin_to_write = BinToWrite::new(data, &path, range);
        self.ctx.asm_out.bin_to_write_chunks.push(bin_to_write);

        // return the path written to, may have been expanded
        Ok(path)
    }

    fn get_expanded_path<P: AsRef<Path>>(&self, path: P) -> PathBuf { 
        self
            .ctx
            .get_vars()
            .expand_vars_in_path(&path)
    }

    fn get_abs_path<P: AsRef<Path>>(&mut self, path: P) -> PathBuf {
        let path = self.get_expanded_path(path);
        emu::utils::fileutils::abs_path_from_cwd(path)
    }


    pub fn eval_macro_args(&mut self, scope_id: u64, caller_id: AstNodeId, tree: &AstTree) -> bool {
        let node = tree.get(caller_id).unwrap();
        self.ctx.eval_macro_args(scope_id, node)
    }

    pub fn get_file_size<P: AsRef<Path>>(&self, path: P) -> GResult<usize> {
        let path = self.get_expanded_path(&path);
        let ret = self.ctx.get_source_file_loader().get_size(path)?;
        Ok(ret)
    }

    pub fn read_binary<P: AsRef<Path>>(&mut self, path: P) -> GResult<(PathBuf, Vec<u8>)> {
        let path = self.get_expanded_path(path);
        let ret = self.ctx.get_source_file_loader_mut().read_binary(path)?;
        Ok(ret)
    }

    pub fn read_binary_chunk<P: AsRef<Path>>(
        &mut self,
        path: P,
        r: std::ops::Range<usize>,
    ) -> GResult<(PathBuf, Vec<u8>)> {
        let path = self.get_expanded_path(&path);
        let ret = self
            .ctx
            .get_source_file_loader_mut()
            .read_binary_chunk(path, r)?;
        Ok(ret)
    }

    pub fn add_source_mapping(&mut self, pos: &Position, pc: usize) {
        let (_, phys_range) = self.binary().range_to_write_address(pc);

        let si = self.ctx.get_source_info(pos);

        if let Ok(si) = si {
            let mem_text = if phys_range.is_empty() {
                String::new()
            } else {
                format!(
                    "{:02X?}",
                    self.ctx.asm_out.binary.get_bytes_range(phys_range.clone())
                )
            };

            let m_pc = format!("{:05X} {:04X} {} ", phys_range.start, pc, mem_text);
            let m = format!("{:50}{}", m_pc, si.line_str);

            if !mem_text.is_empty() {
                self.ctx.asm_out.lst_file.add(&m);
            }
        }
    }
}
