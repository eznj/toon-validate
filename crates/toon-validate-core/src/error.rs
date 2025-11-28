use thiserror::Error;

#[derive(Error, Debug)]
pub enum TqError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },
    
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Table {name} declared with {declared} rows but found {actual}")]
    TableRowMismatch {
        name: String,
        declared: usize,
        actual: usize,
    },
    
    #[error("Inconsistent table schema in {name}: {message}")]
    TableSchemaInconsistent { name: String, message: String },
    
    #[error("Invalid input format: {0}")]
    InvalidFormat(String),
}

pub type Result<T> = std::result::Result<T, TqError>;