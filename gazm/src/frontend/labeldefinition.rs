#![forbid(unused_imports)]
use crate::gazmsymbols::SymbolScopeId;

#[derive(Debug, PartialEq, Hash, Clone)]
pub enum LabelDefinition {
    Text(String),
    TextScoped(String),
    Scoped(SymbolScopeId),
}

impl LabelDefinition {
    pub fn get_text(&self) -> Option<&str> {
        match self {
            LabelDefinition::TextScoped(x) | LabelDefinition::Text(x) => Some(x),
            LabelDefinition::Scoped(_) => None,
        }
    }

    pub fn get_id(&self) -> Option<SymbolScopeId> {
        use LabelDefinition::*;
        match self {
            TextScoped(..) | LabelDefinition::Text(..) => None,
            Scoped(id) => Some(*id),
        }
    }

    pub fn map_string<F>(&self, f: F) -> Self
    where
        F: FnOnce(&str) -> String,
    {
        use LabelDefinition::*;
        match self {
            TextScoped(x) => LabelDefinition::TextScoped(f(x)),
            Text(x) => LabelDefinition::Text(f(x)),
            Scoped(_) => self.clone(),
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            LabelDefinition::TextScoped(x) | LabelDefinition::Text(x) => x.clone(),
            LabelDefinition::Scoped(x) => format!("{x:?}"),
        }
    }
}

impl From<SymbolScopeId> for LabelDefinition {
    fn from(value: SymbolScopeId) -> Self {
        Self::Scoped(value)
    }
}

impl std::fmt::Display for LabelDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LabelDefinition::*;
        match self {
            Scoped(x) => write!(f, "Scoped({},{})", x.scope_id, x.symbol_id),
            TextScoped(x) => write!(f, "{x}"),
            Text(x) => write!(f, "{x}"),
        }
    }
}

