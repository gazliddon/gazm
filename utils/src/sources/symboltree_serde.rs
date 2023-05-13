use serde::ser::SerializeMap;
use std::collections::HashMap;
use super::{SymbolTree, SymbolNodeRef, SymbolWriter, SymbolResolutionBarrier};

fn split_fqn(text: &str) -> Vec<&str> {
    text.split("::").collect()
}
fn get_subscope<'a>(n: SymbolNodeRef<'a>, name: &str) -> Option<SymbolNodeRef<'a>> {
    n.children().find(|c| c.value().get_scope_name() == name)
}
impl serde::Serialize for SymbolTree {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let hm = self.to_hash_map();
        let mut map = serializer.serialize_map(Some(hm.len()))?;
        for (k, v) in hm {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for SymbolTree {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut ret = Self::new();
        let hm: HashMap<String, Option<i64>> = serde::Deserialize::deserialize(_deserializer)?;

        for (k, v) in hm {
            ret.add_fqn(&k, v)
        }

        Ok(ret)
    }
}


impl SymbolTree {
    pub fn to_hash_map(&self) -> HashMap<String, Option<i64>> {
        panic!()
    }

    pub fn to_json(&self) -> String {
        let hm = self.to_hash_map();
        serde_json::to_string_pretty(&hm).unwrap()
    }

    // This is shit, much shame
    pub fn add_fqn(&mut self, text: &str, val: Option<i64>) {
        let items: Vec<_> = split_fqn(text);

        let (path, sym) = match items.len() {
            0 => panic!("WTF"),
            1 => panic!("Neeed 2!"),
            _ => (&items[0..items.len() - 1], &items[items.len() - 1]),
        };

        assert!(path[0].is_empty());

        // pop the first one off
        let mut scope_id = self.tree.root().value().get_scope_id();

        for part in &path[1..] {
            let n = self.get_node_from_id(scope_id).unwrap();
            let n_id = n.value().get_scope_id();

            if let Some(new_id) = get_subscope(n, part) {
                scope_id = new_id.value().get_scope_id();
            } else {
                let new_scope_id =
                    self.insert_new_table(part, n_id, SymbolResolutionBarrier::default());
                scope_id = new_scope_id
            }
        }

        let mut n = self.get_node_mut_from_id(scope_id).unwrap();
        n.value().create_and_set_symbol(sym, val.unwrap()).unwrap();
    }
}
