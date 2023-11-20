#![forbid(unused_imports)]

use crate::{
    ast::{Ast, AstNodeId, AstNodeRef},
    error::{GResult, UserError},
    gazmeval::eval,
    gazmsymbols::SymbolInfo,
    frontend::Item::*,
};

use super::Assembler;

impl Assembler {
    /// Evaluate all macro args
    /// if all arguments were evaluated returns true
    pub fn eval_macro_args_node(
        &mut self,
        scope_id: u64,
        caller_id: AstNodeId,
        tree: &Ast,
    ) -> bool {
        let node = tree.as_ref().get(caller_id).unwrap();
        self.eval_macro_args(scope_id, node)
    }

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
                let reader = self.asm_out.symbols.get_reader(eval_scope_id);
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
                                .set_symbol_for_id(*symbol_id, value)
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

    pub fn get_node_children(&self, node: AstNodeRef) -> Vec<AstNodeId> {
        node.children().map(|n| n.id()).collect()
    }

    pub fn eval_node(&self, node: AstNodeRef, current_scope_id: u64) -> GResult<i64> {
        let info = self.get_source_info(&node.value().pos).unwrap();
        let reader = self.asm_out.symbols.get_reader(current_scope_id);

        eval(&reader, node).map_err(|err| {
            let e = match &err.source {
                crate::gazmeval::EvalErrorEnum::SymbolNotFoud(name) => {
                    let scope = self.get_symbols().get_fqn_from_id(current_scope_id);
                    let mut err = err.clone();
                    err.source =
                        crate::gazmeval::EvalErrorEnum::SymbolNotFoud(format!("{scope}::{name}"));
                    UserError::from_ast_error(err.into(), &info)
                }
                _ => UserError::from_ast_error(err.into(), &info),
            };
            e.into()
        })
    }


    pub fn eval_first_arg(
        &self,
        node: AstNodeRef,
        current_scope_id: u64,
    ) -> GResult<(i64, AstNodeId)> {
        let c = node
            .first_child()
            .ok_or_else(|| self.make_user_error("Missing argument", node, true))?;
        let v = self.eval_node(c, current_scope_id)?;
        Ok((v, c.id()))
    }

    pub fn eval_two_args(&self, node: AstNodeRef, current_scope_id: u64) -> GResult<(i64, i64)> {
        let args = self.eval_n_args(node, 2, current_scope_id)?;
        assert!(args.len() == 2);
        Ok((args[0], args[1]))
    }

    pub fn eval_n_args(
        &self,
        node: AstNodeRef,
        n: usize,
        current_scope_id: u64,
    ) -> GResult<Vec<i64>> {
        node.children()
            .take(n)
            .map(|node| self.eval_node(node, current_scope_id))
            .collect()
    }

}
