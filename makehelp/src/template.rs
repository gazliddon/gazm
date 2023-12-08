pub const TEMPLATE : & str = r#"

    #[derive(Clone,Debug)]
    pub struct HelpItem {
        short: String,
        full_text: String,
    }

    impl HelpItem {
        pub fn new(short: &str, full_text: &str) -> Self {
            HelpItem {
                short: short.to_string(),
                full_text: full_text.to_string(),
            }
        }
    }

    use std::collections::HashMap;

    #[derive(Eq,Hash,PartialEq,Clone,Debug, Copy)]
    pub enum ErrCode {
        {{enums}}
    }

    pub struct Err {
        code_to_data : HashMap<ErrCode,HelpItem>,
    }

    impl Err {
        pub fn new() -> Self {
            use ErrCode::*;
            let xx = [
                {{{data}}}
            ];

            Self {
                code_to_data : xx.into(),
            }
        }
        pub fn get(&self, e : ErrCode) -> String {
            let di = self.code_to_data.get(&e).unwrap();
            di.full_text.clone()
        }

        pub fn get_short(&self, e : ErrCode) -> String {
            let di = self.code_to_data.get(&e).unwrap();
            di.short.clone()
        }
    }
"#;

