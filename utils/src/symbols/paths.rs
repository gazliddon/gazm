pub fn split_scopes<'a>(txt: &'a str) -> Vec<&'a str> {
    let ret = txt.split("::").collect();
    ret
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopePath {
    is_relative: bool,
    parts: Vec<String>,
}

impl ScopePath {
    pub fn from_base_path(base : &ScopePath, path : &ScopePath) -> Option<Self>{
        if base.is_abs() && path.is_relative() {
            let mut parts = base.parts.clone();
            parts.extend(path.parts.iter().cloned());
            Some(Self {
                is_relative: false,
                parts
            })
        } else {
            None
        }
    }

    pub fn new<T: AsRef<str>>(txt: T) -> Self {
        let mut x = split_scopes(txt.as_ref());

        let is_relative = match x.get(0) {
            // If empty it's an abs scope pointing to root
            None => {
                false
            }

            // If it's then the path started with ::
            // so we pop the front item and classify as abs
            Some(&"")  => {
                x.remove(0);
                false
            },

            // Otherwise this is a relative path
            _ => {
                true
            },
        };

        Self {
            is_relative,
            parts: x.iter().map(|x| String::from(*x)).collect(),
        }
    }

    pub fn is_relative(&self) -> bool {
        self.is_relative
    }

    pub fn is_abs(&self) -> bool {
        !self.is_relative()
    }

    pub fn get_parts(&self) -> &[String] {
        &self.parts
    }
}

impl<T: AsRef<str>> From<T> for ScopePath {
    fn from(x: T) -> Self {
        ScopePath::new(x)
    }
}

impl std::fmt::Display for ScopePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut joined = self.parts.join("::");

        if self.is_abs() {
            joined = format!("::{joined}")
        }

        write!(f, "{joined}")
    }
}

pub struct SymbolPath {
    path: ScopePath,
    name: String,
}

impl SymbolPath {
    pub fn new<P: Into<ScopePath>, N: Into<String>>(path: P, name: N) -> Self {
        Self {
            path: path.into(),
            name: name.into(),
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
