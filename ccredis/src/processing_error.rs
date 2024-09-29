#[derive(Debug)]
pub enum ProcessingError {
    InvalidUtf8,     
    InvalidInteger, 
    Other(String),   
}

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessingError::InvalidUtf8 => write!(f, "Invalid UTF-8 sequence encountered"),
            ProcessingError::InvalidInteger => write!(f, "Invalid integer format encountered"),
            ProcessingError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ProcessingError {}

impl From<&str> for ProcessingError {
    fn from(error: &str) -> Self {
        ProcessingError::Other(error.to_string())
    }
}

impl From<String> for ProcessingError {
    fn from(error: String) -> Self {
        ProcessingError::Other(error)
    }
}