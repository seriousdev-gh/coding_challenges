use std::io::Write;

use crate::processing_error::ProcessingError;

#[derive(Debug, PartialEq)]
pub enum Message {
    Array(Option<Vec<Message>>),
    BulkString(Option<Vec<u8>>),
    SimpleString(String),
    Integer(i64),
    Error(String),
}

impl Message {
    pub fn bulk_string(string: &str) -> Message {
        Message::BulkString(Some(string.to_string().into_bytes()))
    }

    pub fn simple_string(string: &str) -> Message {
        Message::SimpleString(string.to_string())
    }

    pub fn error(string: &str) -> Message {
        Message::Error(string.to_string())
    }

    pub fn array(messages: Vec<Message>) -> Message {
        Message::Array(Some(messages))
    }

    pub fn from_cli(command: &str) -> Message {
        let mut messages: Vec<Message> = Vec::new();
        for string in command.split(' ') {
            messages.push(Message::BulkString(Some(string.to_string().into_bytes())));
        }
        Message::Array(Some(messages))
    }

    pub fn type_as_str(&self) -> &str {
        match self {
            Message::Array(_) => "Array",
            Message::BulkString(_) => "BulkString",
            Message::SimpleString(_) => "SimpleString",
            Message::Integer(_) => "Integer",
            Message::Error(_) => "Error",
        }
    }

    pub fn as_str(&self) -> Result<&str, ProcessingError> {
        match self {
            Self::BulkString(Some(content)) => {  
                let text = std::str::from_utf8(content).map_err(|_| ProcessingError::InvalidUtf8)?;
                Ok(text)
            },
            Self::BulkString(None) => {  
                Err(ProcessingError::Other("Expected bulk string to contain something".to_string()))
            },
            Self::SimpleString(content) => {  
                Ok(content)
            },
            _ => {
                Err(ProcessingError::Other(format!("Invalid message type {} expected BulkString or SimpleString", self.type_as_str())))
            }
        }
    }

    pub fn as_int(&self) -> Result<i64, ProcessingError> {
        match self {
            Self::Integer(content) => {  
                Ok(*content)
            },
            _ => {
                Err(ProcessingError::Other(format!("Invalid message type {} expected Integer", self.type_as_str())))
            }
        }
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            Message::Array(Some(items)) => {
                write!(writer, "*{}\r\n", items.len())?;
                for item in items {
                    item.write_to(writer)?;
                }
            }
            Message::Array(None) => {
                write!(writer, "*-1\r\n")?;
            }
            Message::BulkString(Some(data)) => {
                write!(writer, "${}\r\n", data.len())?;
                writer.write_all(&data)?;
                write!(writer, "\r\n")?;
            }
            Message::BulkString(None) => {
                write!(writer, "$-1\r\n")?;
            }
            Message::SimpleString(value) => {
                write!(writer, "+{}\r\n", value)?;
            }
            Message::Error(error_message) => {
                write!(writer, "-{}\r\n", error_message)?;
            }
            Message::Integer(value) => {
                write!(writer, ":{}\r\n", value)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_equality() {
        assert_ne!(
            Message::SimpleString("OK".to_string()),
            Message::Array(Some(vec![Message::SimpleString("OK".to_string())]))
        );
    }
}
