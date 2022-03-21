use crate::{binary, fixerupper::FixerUpper};
use crate::evaluator::Evaluator;
use utils::sources::{ SymbolError,SymbolWriter };
use crate::ast::{ AstNodeRef, AstTree, AstNodeId };
use utils::sources::SourceMapping;

pub struct AsmCtx<'a> {
    pub fixer_upper: &'a mut FixerUpper,
    pub eval: &'a mut Evaluator<'a>,
    pub direct_page: Option<u8>,
    pub source_map: SourceMapping,
    pub binary : binary::Binary,
}

impl<'a> AsmCtx<'a> {
    pub fn set_dp(&mut self, dp: i64) {
        if dp < 0 {
            self.direct_page = None
        } else {
            self.direct_page = Some(dp as u64 as u8)
        }
    }
    pub fn set_root(&mut self) {
        panic!()
    }

    pub fn pop_scope(&mut self) {
        self.eval.get_symbols_mut().pop_scope()
    }

    pub fn set_scope(&mut self, name: &str) {
        self.eval.get_symbols_mut().set_scope(name)
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
}
