mod symvalue;
/// Replacement symbols for gazm
/// using the generic symbol table stuff in utils
mod value;

pub use symvalue::*;
pub use value::*;

use symbols;

pub type ScopeId = u64;
pub type SymbolId = u64;
pub type SymValue = Value;

pub type SymbolInfo = symbols::SymbolInfo<ScopeId, SymbolId, SymValue>;
pub type SymbolScopeId = symbols::SymbolScopeId<ScopeId, SymbolId>;
pub type SymbolError = symbols::SymbolError<ScopeId, SymbolId>;

pub type SymbolTreeWriter<'a> =
    symbols::symboltreewriter::SymbolTreeWriter<'a, ScopeId, SymbolId, SymValue>;
pub type SymbolTreeReader<'a> =
    symbols::symboltreereader::SymbolTreeReader<'a, ScopeId, SymbolId, SymValue>;
pub type SymbolTree = symbols::symboltree::SymbolTree<ScopeId, SymbolId, SymValue>;

mod test {
    use symbols::ScopedName;

    use super::*;

    #[test]
    fn testit() {
        let mut sym_tree = SymbolTree::new();

        let tab = vec![
            ("::a_macro", Value::Macro),
            ("::another_macro", Value::Macro),
            ("::test::sym", Value::Float(1.0)),
            ("::test::sym2", Value::Text("Hello".to_owned())),
            ("::test::sym3", Value::Null),
            ("::gaz::one", Value::Unsigned(1)),
            ("::gaz::two", Value::Signed(-20)),
            ("::gaz::three", Value::Null),
        ];

        for (name, v) in tab {
            let res = sym_tree.create_fqn(name).expect("Can't create symbol");
            sym_tree.set_value_for_id(res, v).expect("Can't set value");
        }

        {
            let scope_id = sym_tree.get_scope_id("::gaz").expect("Urgh");
            let sym = sym_tree.get_symbol_info_from_name("::test::sym2").unwrap();
            sym_tree
                .add_reference_symbol("reference_symbol", scope_id, sym.symbol_id)
                .unwrap();
        }

        let indent_size = 4;
        sym_tree.walk_tree(&|v, depth| {
            use symbols::Walker;
            let indent = " ".repeat(depth * indent_size);
            let sym_indent = " ".repeat(depth * indent_size + indent_size / 2);

            match v {
                Walker::Scope(name) => {
                    let name = if name == "" { "ROOT" } else { name };

                    println!("{indent}Scope: {name}")
                }
                Walker::Sym(name, v) => println!("{sym_indent}Sym: {name} = {:?}", v),
                Walker::RefSym(name, orig_bame, v) => {
                    println!("{sym_indent}Ref: {name} -> {orig_bame} = {:?}", v)
                }
            }
        });
        assert!(false);
    }
}
