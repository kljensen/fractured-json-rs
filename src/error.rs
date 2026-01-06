use thiserror::Error;

pub type Result<T> = std::result::Result<T, FracturedJsonError>;

#[derive(Debug, Error)]
pub enum FracturedJsonError {
    #[error("Parse error: {0}")]
    ParseError(#[from] jsonc_parser::errors::ParseError),

    #[error("Invalid option: {0}")]
    InvalidOption(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Formatting error: {0}")]
    FormattingError(String),
}
