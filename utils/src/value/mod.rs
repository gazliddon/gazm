mod symvalue;
/// Concrete version of symbol tables used in Gazm
mod value_ops;
pub use value_ops::*;

pub use symvalue::*;

// use crate::expsymbols::{ValueTraits,Scopes,SymbolWriter, SymbolPath};
// use crate::symbols::SymbolTree;

// impl<V: ValueTraits + From<Option<i64>>, ID: IdTraits> From<SymbolTree> for Scopes<V,ID> {
//     fn from(value: SymbolTree) -> Self {
//         panic!()
//         let mut scopes : Scopes<V,ID> = Scopes::new();

//         let x = value.to_hash_map();

//         for (k,v) in x {
//             let sym_path = SymbolPath::from_full_path(&k);
//             let id = scopes.create_scope_recursive(sym_path.path).unwrap();
//             let mut x = scopes.get_mut(id).unwrap();
//             x.value().add_symbol_with_value(&sym_path.name, v.into()).unwrap();
//         }

//         scopes
//     }
// }

// impl From<Option<i64>> for SymbolValue {
//     fn from(v: Option<i64>) -> Self {
//         let value =  match v {
//                 Some(value) => Value::Signed(value),
//                 None => Value::Null,
//             };

//         Self::new(value,None)
//     }
// }

