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
            let old_scope = self.asm_out.symbols.get_current_scope_id();
            self.asm_out
                .symbols
                .set_scope_from_id(eval_scope_id)
                .unwrap();

            // println!("Evaling macro argos for {_name}");

            let mut args_evaled = 0;
            let num_of_args = params_vec_of_id.len();
            // Set the scope to this macros scope
            let it = params_vec_of_id.iter().zip(caller_node.children());

            // Lazy evaluation of parameters
            it.for_each(|(symbol_id, arg_value)| {
                let si = self.asm_out.symbols.get_symbol_info_from_id(*symbol_id);

                match &si {
                    // Already evaluated
                    Ok(SymbolInfo { value: Some(_), .. }) => {
                        args_evaled += 1;
                    }

                    // Symbol exists but has no value
                    Ok(SymbolInfo { value: None, .. }) => {
                        // Try and evaluate the node
                        if let Ok(value) = self.eval_node(arg_value) {
                            // Success, add the symbol!
                            self.asm_out
                                .symbols
                                .set_symbol(*symbol_id, value)
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

            // Pop the macro scope
            self.asm_out.symbols.set_scope_from_id(old_scope).unwrap();
            // Return if all args were evaluated or not
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

    pub fn eval_node(&self, node: AstNodeRef) -> GResult<i64> {
        let info = self.get_source_info(&node.value().pos).unwrap();
        eval(&self.asm_out.symbols, node).map_err(|err| {
            let e = match &err.source {
                crate::eval::EvalErrorEnum::SymbolNotFoud(name) => {
                    let scope = self.get_symbols().get_current_scope_fqn();
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

    pub fn eval_with_pc(&mut self, n: AstNodeRef, pc: u64) -> GResult<i64> {
        self.asm_out
            .symbols
            .add_symbol_with_value("*", pc as i64)
            .unwrap();
        let ret = self.eval_node(n)?;
        self.asm_out.symbols.remove_symbol_name("*");
        Ok(ret)
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

    pub fn eval_first_arg(&self, node: AstNodeRef) -> GResult<(i64, AstNodeId)> {
        let c = node
            .first_child()
            .ok_or_else(|| self.user_error("Missing argument", node, true))?;
        let v = self.eval_node(c)?;
        Ok((v, c.id()))
    }

    pub fn eval_two_args(&self, node: AstNodeRef) -> GResult<(i64, i64)> {
        let args = self.eval_n_args(node, 2)?;
        assert!(args.len() == 2);
        Ok((args[0], args[1]))
    }

    pub fn eval_n_args(&self, node: AstNodeRef, n: usize) -> GResult<Vec<i64>> {
        node.children()
            .take(n)
            .map(|node| self.eval_node(node))
            .collect()
    }
}

// impl<'a> Evaluator<'a> {
//     pub fn new(symbols: &'a mut SymbolTree, source_file_loader: &'a mut SourceFileLoader) -> Self {
//         Self {
//             symbols,
//             source_file_loader,
//         }
//     }

//     pub fn loader_mut(&mut self) -> &mut SourceFileLoader {
//         self.source_file_loader
//     }
//     pub fn loader(&self) -> &SourceFileLoader {
//         self.source_file_loader
//     }

//     /// Evaluate all macro args
//     /// if all arguments were evaluated returns true

//     pub fn eval_macro_args(
//         &mut self,
//         scope: &str,
//         node: AstNodeRef,
//         macro_node: AstNodeRef,
//     ) -> bool {
//         if let MacroDef(_name, params) = &macro_node.value().item {
//             self.symbols.set_scope(scope);
//             let mut assignments = vec![];

//             let mut args_evaled = 0;

//             let args = node.children();

//             args.zip(params).for_each(|(arg, param)| {
//                 if !self
//                     .symbols
//                     .get_current_scope_symbols()
//                     .symbol_exists_from_name(param)
//                 {
//                     let evaled = self.eval_node(arg);
//                     if let Ok(value) = evaled {
//                         assignments.push((param.clone(), value));
//                         args_evaled += 1;
//                     }
//                 } else {
//                     args_evaled += 1;
//                 }
//             });

//             for (param, value) in assignments {
//                 let res = self.symbols.add_symbol_with_value(&param, value);

//                 if res.is_err() {
//                     println!("{}", self.symbols.get_current_scope_symbols());
//                     panic!();
//                 }
//             }
//             self.symbols.pop_scope();

//             args_evaled == params.len()
//         } else {
//             panic!()
//         }
//     }

//     pub fn get_children(&self, node: AstNodeRef) -> Vec<AstNodeId> {
//         node.children().map(|n| n.id()).collect()
//     }

//     pub fn eval_node(&self, node: AstNodeRef) -> GResult<i64> {
//         let info = self.get_source_info(&node.value().pos).unwrap();
//         eval(self, node).map_err(|err| {
//             let e = match &err.source {
//                 crate::eval::EvalErrorEnum::SymbolNotFoud(name) => {
//                     let scope = self.symbols.get_current_scope_fqn();
//                     let mut err = err.clone();
//                     err.source =
//                         crate::eval::EvalErrorEnum::SymbolNotFoud(format!("{scope}::{name}"));
//                     UserError::from_ast_error(err.into(), &info)
//                 }
//                 _ => UserError::from_ast_error(err.into(), &info),
//             };
//             e.into()
//         })
//     }

//     pub fn get_source_info(&self, pos: &Position) -> Result<SourceInfo, SourceErrorType> {
//         self.source_file_loader.sources.get_source_info(pos)
//     }

//     pub fn eval_with_pc(&mut self, n: AstNodeRef, pc: u64) -> GResult<i64> {
//         self.symbols.add_symbol_with_value("*", pc as i64).unwrap();
//         let ret = self.eval_node(n)?;
//         self.symbols.remove_symbol_name("*");
//         Ok(ret)
//     }
//     pub fn user_error<S: Into<String>>(
//         &self,
//         err: S,
//         node: AstNodeRef,
//         is_failure: bool,
//     ) -> UserError {
//         let info = self.get_source_info(&node.value().pos).unwrap();
//         UserError::from_text(err, &info, is_failure)
//     }

//     pub fn eval_first_arg(&self, node: AstNodeRef) -> GResult<(i64, AstNodeId)> {
//         let c = node
//             .first_child()
//             .ok_or_else(|| self.user_error("Missing argument", node, true))?;
//         let v = self.eval_node(c)?;
//         Ok((v, c.id()))
//     }

//     pub fn eval_two_args(&self, node: AstNodeRef) -> GResult<(i64, i64)> {
//         let args = self.eval_n_args(node, 2)?;
//         assert!(args.len() == 2);
//         Ok((args[0], args[1]))
//     }

//     pub fn eval_n_args(&self, node: AstNodeRef, n: usize) -> GResult<Vec<i64>> {
//         node.children()
//             .take(n)
//             .map(|node| self.eval_node(node))
//             .collect()
//     }
// }
