#[derive(Debug)]
pub enum ParseError {
    InvalidByte(u8), // When an unexpected byte is encountered
    InvalidUtf8,     // When there's an error decoding UTF-8
    InvalidInteger,  // When parsing an integer fails
    Other(String),   // Generic error case with a custom message
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidByte(byte) => write!(f, "Invalid byte encountered: 0x{:02x}", byte),
            ParseError::InvalidUtf8 => write!(f, "Invalid UTF-8 sequence encountered"),
            ParseError::InvalidInteger => write!(f, "Invalid integer format encountered"),
            ParseError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ParseError {}