use core::str;

use super::message::*;
use super::parse_error::*;

#[derive(Debug)]
enum State {
    ParseType,
    ReadBuf,
    ReadBulkStringContent,
    AwaitBulkStringEnd,
}

#[derive(Debug)]
enum MessageType {
    Unknown,
    Array,
    BulkString,
    SimpleString,
    Integer,
    Error,
}

struct ArrayStackItem {
    items: Vec<Message>,
    size: usize,
}

pub struct MessageParser {
    array_stack: Vec<ArrayStackItem>,
    buf: Vec<u8>,
    prev_byte: u8,
    state: State,
    message_type: MessageType,
    bulk_string_size: usize,
}

impl MessageParser {
    pub fn new() -> Self {
        Self {
            array_stack: Vec::new(),
            buf: Vec::new(),
            bulk_string_size: 0,
            prev_byte: 0,
            state: State::ParseType,
            message_type: MessageType::Unknown,
        }
    }

    fn reset_state(&mut self) {
        self.array_stack.clear();
        self.buf.clear();
        self.bulk_string_size = 0;
        self.prev_byte = 0;
        self.state = State::ParseType;
        self.message_type = MessageType::Unknown;
    }

    // returns Result<Some> when message is fully parsed
    // returns Result<None> when message is partially parsed
    // returns Err<MessageParseError> in case of some error
    pub fn add_byte(&mut self, byte: u8) -> Result<Option<Message>, ParseError> {
        match self.state {
            State::ParseType => {
                match byte {
                    b'*' => self.message_type = MessageType::Array,
                    b'+' => self.message_type = MessageType::SimpleString,
                    b'-' => self.message_type = MessageType::Error,
                    b':' => self.message_type = MessageType::Integer,
                    b'$' => self.message_type = MessageType::BulkString,
                    _ => return Err(ParseError::InvalidByte(byte)),
                };
                self.buf.clear();
                self.state = State::ReadBuf;
            }

            State::ReadBuf => {
                self.buf.push(byte);
                if self.is_line_end(byte) {
                    self.state = State::ParseType;
                    let parsed_item = self.parse_buffer()?;

                    if let Some(result) = self.try_result(parsed_item) {
                        self.reset_state();
                        return Ok(Some(result));
                    }
                }
            }

            State::ReadBulkStringContent => {
                if self.buf.len() == self.bulk_string_size {
                    self.state = State::AwaitBulkStringEnd;
                } else {
                    self.buf.push(byte);
                }
            }

            State::AwaitBulkStringEnd => {
                if self.is_line_end(byte) {
                    self.state = State::ParseType;
                    let parsed_item = Some(Message::BulkString(Some(self.buf.clone())));
                    if let Some(result) = self.try_result(parsed_item) {
                        self.reset_state();
                        return Ok(Some(result));
                    }
                }
            }
        }

        self.prev_byte = byte;
        Ok(None)
    }

    fn try_result(&mut self, parsed_item: Option<Message>) -> Option<Message> {
        if let Some(item) = parsed_item {
            return self.process_parsed_item(item);
        }
        None
    }

    fn parse_buffer(&mut self) -> Result<Option<Message>, ParseError> {
        self.buf.truncate(self.buf.len().saturating_sub(2));
        match self.message_type {
            MessageType::SimpleString => {
                let as_string = self.parse_buffer_as_str()?;
                return Ok(Some(Message::simple_string(as_string)));
            }
            MessageType::Error => {
                let as_string = self.parse_buffer_as_str()?;
                return Ok(Some(Message::error(as_string)));
            }
            MessageType::Integer => {
                return Ok(Some(Message::Integer(self.parse_buffer_as_int()?)));
            }
            MessageType::BulkString => match self.parse_buffer_as_int()? {
                -1 => return Ok(Some(Message::BulkString(None))),
                size => {
                    self.bulk_string_size = size as usize;
                    self.buf.clear();
                    self.state = State::ReadBulkStringContent;
                }
            },
            MessageType::Array => match self.parse_buffer_as_int()? {
                -1 => return Ok(Some(Message::Array(None))),
                0 => return Ok(Some(Message::array(Vec::new()))),
                size => {
                    self.array_stack.push(ArrayStackItem {
                        items: Vec::new(),
                        size: size as usize,
                    });
                }
            },
            MessageType::Unknown => {
                return Err(ParseError::Other("Unknown message type".to_string()));
            }
        }
        Ok(None)
    }

    // puts parsed message to current array on stack
    // if array is completed recursively checks stack
    // then stack is empty returns Some(message)
    fn process_parsed_item(&mut self, message: Message) -> Option<Message> {
        if let Some(arr) = self.array_stack.last_mut() {
            arr.items.push(message);
            if arr.items.len() == arr.size {
                let array_message = Message::array(self.array_stack.pop().unwrap().items);
                let result = self.process_parsed_item(array_message);
                if result.is_some() {
                    return result;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            return Some(message);
        }
    }

    fn parse_buffer_as_str(&self) -> Result<&str, ParseError> {
        std::str::from_utf8(&self.buf).map_err(|_| ParseError::InvalidUtf8)
    }

    fn parse_buffer_as_int(&self) -> Result<i64, ParseError> {
        self.parse_buffer_as_str()?
            .parse()
            .map_err(|_| ParseError::InvalidInteger)
    }

    fn is_line_end(&self, byte: u8) -> bool {
        self.prev_byte == b'\r' && byte == b'\n'
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn parse_string(string: &str) -> Message {
        let mut parser = MessageParser::new();
        let mut message: Option<Message> = None;
        for (i, byte) in string.as_bytes().iter().enumerate() {
            message = parser.add_byte(*byte).unwrap();

            assert_eq!(
                message.is_some(),
                i == string.len() - 1,
                "Received: {:?}",
                *byte as char
            );
        }
        message.unwrap()
    }

    #[test]
    fn parse_bulk_string() {
        assert_eq!(
            parse_string("$5\r\nhello\r\n"),
            Message::bulk_string("hello")
        );
    }

    #[test]
    fn parse_simple_string() {
        assert_eq!(
            parse_string("+OK\r\n"),
            Message::SimpleString("OK".to_string())
        );
    }

    #[test]
    fn parse_one_element_array() {
        assert_eq!(
            parse_string("*1\r\n$4\r\nping\r\n"),
            Message::Array(Some(vec![Message::bulk_string("ping")]))
        );
    }

    #[test]
    fn parse_two_element_array_with_bulk_strings() {
        assert_eq!(
            parse_string("*2\r\n$4\r\necho\r\n$11\r\nhello world\r\n"),
            Message::Array(Some(vec![
                Message::bulk_string("echo"),
                Message::bulk_string("hello world")
            ]))
        );
    }

    #[test]
    fn parse_empty_bulk_string() {
        assert_eq!(
            parse_string("$0\r\n\r\n"),
            Message::BulkString(Some(Vec::new()))
        );
    }

    #[test]
    fn parse_null_bulk_string() {
        assert_eq!(parse_string("$-1\r\n"), Message::BulkString(None));
    }

    #[test]
    fn parse_empty_array() {
        assert_eq!(parse_string("*0\r\n"), Message::array(Vec::new()));
    }

    #[test]
    fn parse_null_array() {
        assert_eq!(parse_string("*-1\r\n"), Message::Array(None));
    }

    #[test]
    fn parse_nested_array() {
        assert_eq!(
            parse_string("*2\r\n+baz\r\n*2\r\n+foo\r\n*1\r\n+bar\r\n"),
            Message::array(vec![
                Message::simple_string("baz"),
                Message::array(vec![
                    Message::simple_string("foo"),
                    Message::array(vec![Message::simple_string("bar")])
                ])
            ])
        );
    }

    #[test]
    fn parse_error() {
        assert_eq!(
            parse_string("-Error message\r\n"),
            Message::Error("Error message".to_string())
        );
    }

    #[test]
    fn parse_negative_integer() {
        assert_eq!(parse_string(":-1\r\n"), Message::Integer(-1));
    }

    #[test]
    fn parse_array_of_all_types() {
        assert_eq!(
            parse_string("*4\r\n$4\r\nbulk\r\n+simple\r\n:-1\r\n-err\r\n"),
            Message::array(vec![
                Message::bulk_string("bulk"),
                Message::simple_string("simple"),
                Message::Integer(-1),
                Message::error("err")
            ])
        );
    }
}
