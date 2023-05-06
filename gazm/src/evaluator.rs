use emu::utils::sources::{
    fileloader::SourceFileLoader, Position, SourceErrorType, SourceInfo, SymbolError, SymbolInfo,
    SymbolQuery, SymbolScopeId, SymbolTree, SymbolWriter,
};

use crate::ast::{AstNodeId, AstNodeRef};
use crate::ctx::Context;
use crate::error::{GResult, UserError};
use crate::eval::eval;
use crate::item::Item::*;

/// Evaluates things
// pub struct Evaluator<'a> {
//     symbols: &'a mut SymbolTree,
//     source_file_loader: &'a mut SourceFileLoader,
// }

// impl<'a> SymbolQuery for Evaluator<'a> {
//     fn get_symbol_info(&self, name: &str) -> Result<&SymbolInfo, SymbolError> {
//         self.symbols.get_symbol_info(name)
//     }

//     fn get_symbol_info_from_id(&self, id: SymbolScopeId) -> Result<&SymbolInfo, SymbolError> {
//         self.symbols.get_symbol_info_from_id(id)
//     }
// }

impl Context {
    /// Evaluate all macro args
    /// if all arguments were evaluated returns true

    pub fn eval_macro_args(&mut self, eval_scope_id: u64, caller_node: AstNodeRef) -> bool {
        if let MacroCallProcessed {
            params_vec_of_id, ..
        } = &caller_node.value().item
        {
            let mut args_evaled = 0;
            let num_of_args = params_vec_of_id.len();
            // Set the scope to this macros scope
            let it = params_vec_of_id.iter().zip(caller_node.children());

            // Lazy evaluation of parameters
            it.for_each(|(symbol_id, arg_value)| {
                let reader = self.asm_out.symbols.get_symbol_reader(eval_scope_id);
                let si = reader.get_symbol_info_from_id(*symbol_id);

                match &si {
                    // Already evaluated
                    Ok(SymbolInfo { value: Some(_), .. }) => {
                        args_evaled += 1;
                    }

                    // Symbol exists but has no value
                    Ok(SymbolInfo { value: None, .. }) => {
                        // Try and evaluate the node
                        if let Ok(value) = self.eval_node(arg_value, eval_scope_id) {
                            // Success, add the symbol!
                            self.asm_out
                                .symbols
                                .set_symbol_from_id(*symbol_id, value)
                                .expect("Unexpected failure to add macro param symbol");
                            args_evaled += 1;
                        }
                    }

                    // Has not been evaluated
                    Err(..) => {
                        panic!();
                    }
                }
            });

            
            num_of_args == args_evaled
        } else {
            panic!()
        }
    }

    pub fn get_symbols_mut(&mut self) -> &mut SymbolTree {
        &mut self.asm_out.symbols
    }

    pub fn get_symbols(&self) -> &SymbolTree {
        &self.asm_out.symbols
    }

    pub fn get_children(&self, node: AstNodeRef) -> Vec<AstNodeId> {
        node.children().map(|n| n.id()).collect()
    }

    pub fn eval_node(&self, node: AstNodeRef, current_scope_id: u64) -> GResult<i64> {
        let info = self.get_source_info(&node.value().pos).unwrap();
        let reader = self.asm_out.symbols.get_symbol_reader(current_scope_id);

        eval(&reader, node).map_err(|err| {
            let e = match &err.source {
                crate::eval::EvalErrorEnum::SymbolNotFoud(name) => {
                    let scope = self.get_symbols().get_fqn_from_id(current_scope_id);
                    let mut err = err.clone();
                    err.source =
                        crate::eval::EvalErrorEnum::SymbolNotFoud(format!("{scope}::{name}"));
                    UserError::from_ast_error(err.into(), &info)
                }
                _ => UserError::from_ast_error(err.into(), &info),
            };
            e.into()
        })
    }

    pub fn get_source_info(&self, pos: &Position) -> Result<SourceInfo, SourceErrorType> {
        self.get_source_file_loader().sources.get_source_info(pos)
    }

    pub fn user_error<S: Into<String>>(
        &self,
        err: S,
        node: AstNodeRef,
        is_failure: bool,
    ) -> UserError {
        let info = self.get_source_info(&node.value().pos).unwrap();
        UserError::from_text(err, &info, is_failure)
    }

    pub fn eval_first_arg(&self, node: AstNodeRef, current_scope_id: u64) -> GResult<(i64, AstNodeId)> {
        let c = node
            .first_child()
            .ok_or_else(|| self.user_error("Missing argument", node, true))?;
        let v = self.eval_node(c, current_scope_id)?;
        Ok((v, c.id()))
    }

    pub fn eval_two_args(&self, node: AstNodeRef, current_scope_id: u64) -> GResult<(i64, i64)> {
        let args = self.eval_n_args(node, 2, current_scope_id)?;
        assert!(args.len() == 2);
        Ok((args[0], args[1]))
    }

    pub fn eval_n_args(&self, node: AstNodeRef, n: usize, current_scope_id: u64) -> GResult<Vec<i64>> {
        node.children()
            .take(n)
            .map(|node| self.eval_node(node, current_scope_id))
            .collect()
    }
}

