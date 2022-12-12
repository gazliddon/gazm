pub fn split_scopes<'a>(txt: &'a str) -> Vec<&'a str> {
    let ret = txt.split("::").collect();
    ret
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopePath {
    is_relative: bool,
    path_parts: Vec<String>,
}

impl ScopePath {
    pub fn from_base_path(base: &ScopePath, path: &ScopePath) -> Option<Self> {
        if base.is_abs() && path.is_relative() {
            let mut path_parts = base.path_parts.clone();
            path_parts.extend(path.path_parts.iter().cloned());
            Some(Self {
                is_relative: false,
                path_parts,
            })
        } else {
            None
        }
    }

    pub fn is_root(&self) -> bool {
        self.is_abs() && self.path_parts.get(0) == Some(&"".to_string()) && self.path_parts.len() == 1
    }

    pub fn from_parts(parts: Vec<&str>) -> Self {
        let is_relative = match parts.get(0) {
            // If empty it's an error
            None => panic!("Zero parts!"),
            // If the opening scope is "" (eg ::foo)
            // then this is an absolute path
            Some(&"")=> {
                false
            }

            // Otherwise this is a relative path
            _ => true,
        };

        Self {
            is_relative,
            path_parts: parts.iter().map(|x| String::from(*x)).collect(),
        }
    }

    pub fn get_rel_parts(&self) -> &[String] {
        if self.is_relative {
            &self.path_parts
        } else {
            &self.path_parts[1..]
        }
    }

    pub fn new<T: AsRef<str>>(txt: T) -> Self {
        Self::from_parts(split_scopes(txt.as_ref()))
    }

    pub fn is_relative(&self) -> bool {
        self.is_relative
    }

    pub fn set_is_relative(&mut self, is_relative: bool) {
        self.is_relative = is_relative
    }

    pub fn is_abs(&self) -> bool {
        !self.is_relative()
    }

    pub fn get_parts(&self) -> &[String] {
        &self.path_parts
    }

    pub fn concat(&self, path: &ScopePath) -> ScopePath {
        assert!(self.is_abs());
        assert!(path.is_relative());
        let new_path = format!("{}::{}", self,path);
        Self::new(&new_path)
    }
}

impl<T: AsRef<str>> From<T> for ScopePath {
    fn from(x: T) -> Self {
        ScopePath::new(x)
    }
}

impl std::fmt::Display for ScopePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let joined = self.path_parts.join("::");

        // if self.is_abs() {
        //     joined = format!("::{joined}")
        // }

        write!(f, "{joined}")
    }
}

pub struct SymbolPath {
    pub path: ScopePath,
    pub name: String,
}

impl SymbolPath {
    pub fn new<P: Into<ScopePath>, N: Into<String>>(path: P, name: N) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
        }
    }

    pub fn from_full_path(txt: &str) -> Self {
        let split = split_scopes(txt);

        match split.len() {
            0 => panic!(),
            _ => {
                let path = &split[..split.len()-1];
                let name = split[split.len()-1];
                let path = ScopePath::from_parts(path.to_vec());
                Self::new(path,name)
            }
        }
    }
}
impl std::fmt::Display for SymbolPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.path, self.name)
    }
}

#[allow(unused_imports)]
mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

    #[test]
    fn test_symbol_path() {
        let txt = "::scope::hello";

        let sym_path = SymbolPath::from_full_path(txt);
        assert!(sym_path.path.is_abs());
        assert_eq!(&sym_path.name,"hello");
        assert_eq!(sym_path.path.to_string(), "::scope")
    }

    #[test]
    fn test_split() {
        let txt = "::hello::10";
        println!("{:#?}", split_scopes(txt));

        let txt = String::from("::hello::10");
        let desired = format!("{txt}");
        let x = ScopePath::new(&txt);
        assert_eq!(&x.to_string(), &desired);
        assert!(x.is_abs());

        let txt = String::from("::hello::10");
        let x = ScopePath::new(&txt);
        assert_eq!(&x.to_string(), &txt);
        assert!(x.is_abs());

        let txt = String::from("clang::hello::10");
        let x = ScopePath::new(&txt);
        assert_eq!(&x.to_string(), &txt);
        assert!(x.is_relative());
    }
}
