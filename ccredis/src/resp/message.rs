use std::io::Write;

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

    pub fn is_array(&self) -> bool {
        match self {
            Self::Array(_) => true,
            _ => false
        }
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            Message::Array(Some(items)) => {
                // Array with elements
                write!(writer, "*{}\r\n", items.len())?;
                for item in items {
                    item.write_to(writer)?;
                }
            }
            Message::Array(None) => {
                // Null Array
                write!(writer, "*-1\r\n")?;
            }
            Message::BulkString(Some(data)) => {
                // Bulk String with data
                write!(writer, "${}\r\n", data.len())?;
                writer.write_all(&data)?;
                write!(writer, "\r\n")?;
            }
            Message::BulkString(None) => {
                // Null Bulk String
                write!(writer, "$-1\r\n")?;
            }
            Message::SimpleString(value) => {
                // Simple String
                write!(writer, "+{}\r\n", value)?;
            }
            Message::Error(error_message) => {
                // Error
                write!(writer, "-{}\r\n", error_message)?;
            }
            Message::Integer(value) => {
                // Integer
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
