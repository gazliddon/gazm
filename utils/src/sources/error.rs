use std::io;

#[derive(thiserror::Error, Debug)]
pub enum SourceErrorType {
    #[error("Sort this out gaz")]
    Misc,
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Id not found: {0}")]
    IdNotFound(u64),

    #[error(transparent)]
    Io {
        #[from]
        source: io::Error,
    },

    #[error(transparent)]
    Json {
        #[from]
        source: serde_json::Error,
    },
}


