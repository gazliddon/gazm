/// Generics for evaluating infix expressions
mod postfix;
mod eval;
mod error;
pub use postfix::*;
pub use eval::*;
pub use error::*;

use std::fmt::Debug;

pub fn infix_expr_to_value<
    V: EvalTraits,
    I: ExprItemTraits<V> + Clone + From<V> + Debug + GetPriority,
    E: EvalExpr<V, I, ERR>,
    ERR: From<GenericEvalErrorKind>,
>(i : &[I], evaluator : &E) -> Result<V, ERR>{
    let post_fix_expr = to_postfix(i).map_err(|e| GenericEvalErrorKind::PostFixError(e))?;
    let res = evaluate_postfix_expr(post_fix_expr.into_iter(), evaluator).map_err(|(_,e)| e)?;
    Ok(res)
}
