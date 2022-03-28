
use crate::ctx::Vars;
use crate::error::{GasmError, GResult };
use crate::{binary, fixerupper::FixerUpper};
use crate::evaluator::Evaluator;
use utils::sources::{FileIo, SourceFileLoader, SymbolError, SymbolNodeId, SymbolWriter};
use crate::ast::{ AstNodeRef, AstTree, AstNodeId };
use utils::sources::SourceMapping;
use crate::item::Item;
use std::path::Path;
use std::path::PathBuf;

pub struct AsmCtx<'a> {
    pub fixer_upper: FixerUpper,
    pub eval: Evaluator<'a>,
    pub direct_page: Option<u8>,
    pub source_map: &'a mut SourceMapping,
    pub binary : &'a mut binary::Binary,
    pub vars : &'a Vars,
}

impl<'a> AsmCtx<'a> {

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
        &mut self.eval.source_file_loader
    }


    pub fn write_bin_file<P: AsRef<Path>, C: AsRef<[u8]>>(&mut self, path: P, data: C) -> PathBuf {
        let path = self.vars.expand_vars(path.as_ref().to_string_lossy());
        self.loader().write(path, data)
    }

    pub fn eval_macro_args(
        &mut self,
        scope: &String,
        args_id: AstNodeId,
        macro_id: AstNodeId,
        tree: &AstTree,
    ) {
        let node = tree.get(args_id).unwrap();
        let macro_node = tree.get(macro_id).unwrap();
        self.eval.eval_macro_args(scope, node, macro_node);
    }

    pub fn get_file_size(&self, path: &Path) -> GResult<usize> {
        use utils::sources::FileIo;
        let path = self.vars.expand_vars(path.to_string_lossy());
        let ret = self.eval.source_file_loader.get_size(path)?;
        Ok(ret)
    }

    pub fn read_binary(&mut self, path : &Path) -> GResult<(PathBuf, Vec<u8> )> {
        let path = self.vars.expand_vars(path.to_string_lossy());
        let ret = self.eval.source_file_loader.read_binary(path)?;
        Ok(ret)
    }

    pub fn read_binary_chunk(&mut self, path : &Path,  r : std::ops::Range<usize>) -> GResult<(PathBuf, Vec<u8> )> {
        let path = self.vars.expand_vars(path.to_string_lossy());
        let ret = self.eval.source_file_loader.read_binary_chunk(path, r)?;
        Ok(ret)
    }


}
