
use crate::ctx::{LstFile, Vars};
use crate::error::{ErrorCollector, GResult, GazmError};
use crate::{binary, fixerupper::FixerUpper};
use crate::evaluator::Evaluator;
use utils::sources::{FileIo, SourceFileLoader, SymbolError, SymbolNodeId, SymbolWriter};
use crate::ast::{ AstNodeRef, AstTree, AstNodeId };
use utils::sources::SourceMapping;
use crate::item::Item;
use std::path::Path;
use std::path::PathBuf;
use crate::ctx::{Opts,BinWritten};

pub struct AsmCtx<'a> {
    pub fixer_upper: FixerUpper,
    pub eval: Evaluator<'a>,
    pub direct_page: Option<u8>,
    pub source_map: &'a mut SourceMapping,
    pub binary : &'a mut binary::Binary,
    pub vars : &'a Vars,
    /// Collected errors
    pub errors: &'a mut ErrorCollector,
    pub opts: &'a Opts,
    pub lst_file: &'a mut LstFile,
    /// Execution address
    pub exec_addr : &'a mut Option<usize>,
    /// Written binary chunks
    pub bin_chunks: &'a mut Vec<BinWritten>,
}


impl<'a> AsmCtx<'a> {

    pub fn set_exec_addr(&mut self, addr : usize) {
        *self.exec_addr = Some(addr)
    }

    pub fn add_fixup(
        &mut self,
        id : AstNodeId,
        v : Item
    ) -> (SymbolNodeId, AstNodeId){
        let scope = self.eval.symbols.get_current_scope();
        self.fixer_upper.add_fixup(scope, id, v);
        (scope,id)
    }


    pub fn get_node_item(&'a self, tree : &'a AstTree, id : AstNodeId) -> (AstNodeRef, Item) {
        let node = tree.get(id).unwrap();
        let this_i = &node.value().item;
        let i  = self.get_fixup_or_default(id, this_i);
        (node, i)
    }

    pub fn get_fixup_or_default(
        &self,
        id : AstNodeId,
        i : &Item
    ) -> Item {
        let scope = self.eval.symbols.get_current_scope();
        self.fixer_upper.get_fixup_or_default(scope, id, i)
    }

    pub fn set_dp(&mut self, dp: i64) {
        if dp < 0 {
            self.direct_page = None
        } else {
            self.direct_page = Some(dp as u64 as u8)
        }
    }

    pub fn set_root_scope(&mut self) {
        self.eval.get_symbols_mut().set_root();
    }

    pub fn pop_scope(&mut self) {
        self.eval.get_symbols_mut().pop_scope()
    }

    pub fn set_scope(&mut self, name: &str) {
        self.eval.get_symbols_mut().set_scope(name)
    }

    pub fn get_scope_fqn(&mut self) -> String {
        self.eval.get_symbols().get_current_scope_fqn()
    }

    pub fn add_symbol_with_value(&mut self,name : &str,val: usize) -> Result<u64, SymbolError> {
        self.eval.get_symbols_mut().add_symbol_with_value(name, val as i64)
    }

    pub fn set_pc_symbol(&mut self,val: usize) -> Result<u64, SymbolError> {
        self.add_symbol_with_value("*", val)
    }

    pub fn remove_pc_symbol(&mut self) {
        self.eval.get_symbols_mut().remove_symbol_name("*")
    }

    pub fn loader(&mut self) -> &mut SourceFileLoader {
        self.eval.source_file_loader
    }

    pub fn write_bin_file<P: AsRef<Path>>(&mut self, path : P, range : std::ops::Range<usize>) -> PathBuf {
        let physical_address = range.start;
        let count = range.len();

        let data = self
            .binary
            .get_bytes(physical_address as usize, count as usize)
            .to_vec();

        // Write the file
        // TODO This all needs produce errors if appropriate
        let ret = self.write_bin_file_data(&path, data);


        // Save a record of the file Written
        // this goes into the written sym file eventually
        let bw = BinWritten {
            file: path.as_ref().to_path_buf(),
            addr:range
        };

        self.bin_chunks.push(bw);

        // return the path written to, may have been expanded
        ret
    }


    fn write_bin_file_data<P: AsRef<Path>, C: AsRef<[u8]>>(&mut self, path: P, data: C) -> PathBuf {
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        let path = self.loader().write(path, data);
        path
    }

    pub fn eval_macro_args(
        &mut self,
        scope: &str,
        args_id: AstNodeId,
        macro_id: AstNodeId,
        tree: &AstTree,
    ) {
        let node = tree.get(args_id).unwrap();
        let macro_node = tree.get(macro_id).unwrap();
        self.eval.eval_macro_args(scope, node, macro_node);
    }

    pub fn get_file_size<P : AsRef<Path>>(&self, path: P) -> GResult<usize> {
        use utils::sources::FileIo;
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        let ret = self.eval.source_file_loader.get_size(path)?;
        Ok(ret)
    }

    pub fn read_binary<P: AsRef<Path>>(&mut self, path : P) -> GResult<(PathBuf, Vec<u8> )> {
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        let ret = self.eval.source_file_loader.read_binary(path)?;
        Ok(ret)
    }

    pub fn read_binary_chunk<P: AsRef<Path>>(&mut self, path : P,  r : std::ops::Range<usize>) -> GResult<(PathBuf, Vec<u8> )> {
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        let ret = self.eval.source_file_loader.read_binary_chunk(path, r)?;
        Ok(ret)
    }


}
