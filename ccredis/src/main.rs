use core::str;

// #[derive(Debug, Clone)]
// struct MessageParseError(String);
// impl std::error::Error for MessageParseError {}
// impl std::fmt::Display for MessageParseError {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "Error while parsing message: {}", self.0)
//     }
// }

#[derive(Debug, PartialEq)]
enum Message {
    Array(Option<Vec<Message>>),
    BulkString(Option<Vec<u8>>),
    SimpleString(String),
    Integer(i64),
    Error(String),
}

impl Message {
    fn bulk_string(string: &str) -> Message {
        Message::BulkString(Some(string.to_string().into_bytes()))
    }

    fn simple_string(string: &str) -> Message {
        Message::SimpleString(string.to_string())
    }

    fn error(string: &str) -> Message {
        Message::Error(string.to_string())
    }

    fn array(messages: Vec<Message>) -> Message {
        Message::Array(Some(messages))
    }
}

#[derive(Debug)]
enum State {
    ParseType,
    ParseArraySize,
    ParseBulkStringSize,
    ReadBulkStringContent,
    ParseSimpleString,
    ParseError,
    ParseInteger,
}

struct MessageParser {
    array_stack: Vec<(Vec<Message>, usize)>,
    result: Option<Message>,
    buf: Vec<u8>,
    prev_byte: u8,
    state: State,
    bulk_string_size: usize,
}

impl MessageParser {
    fn new() -> Self {
        Self {
            array_stack: Vec::new(),
            result: None,
            buf: Vec::new(),
            bulk_string_size: 0,
            prev_byte: 0,
            state: State::ParseType,
        }
    }

    fn value(self) -> Option<Message> {
        return self.result;
    }

    // TODO: implement errors instead of panic
    fn add_byte(&mut self, byte: u8) -> bool {
        println!(
            "add byte: {:?} state: {:?} buffer: {:?}",
            byte as char,
            self.state,
            str::from_utf8(self.buf.as_slice())
        );
        let mut parsed_item: Option<Message> = Option::None;
        match self.state {
            State::ParseType => match byte {
                b'\r' => {}
                b'\n' => {
                    if self.is_line_end(byte) && self.result.is_some() {
                        return true;
                    }
                }
                b'*' => self.state = State::ParseArraySize,
                b'+' => self.state = State::ParseSimpleString,
                b'-' => self.state = State::ParseError,
                b':' => self.state = State::ParseInteger,
                b'$' => self.state = State::ParseBulkStringSize,
                _ => {
                    panic!("Invalid first byte: {:?}", byte as char);
                }
            },
            State::ParseArraySize => {
                self.buf.push(byte);
                if self.buf == "-1".as_bytes() {
                    self.state = State::ParseType;
                    self.buf.clear();
                    parsed_item = Some(Message::Array(None));
                } else if self.buf == "0".as_bytes() {
                    self.state = State::ParseType;
                    self.buf.clear();
                    parsed_item = Some(Message::array(Vec::new()));
                } else if self.is_line_end(byte) {
                    let size = self.parse_buffer_as_size();
                    self.array_stack.push((Vec::new(), size));
                    self.state = State::ParseType;
                    self.buf.clear();
                }
            }
            State::ParseBulkStringSize => {
                self.buf.push(byte);
                if self.buf == "-1".as_bytes() {
                    self.state = State::ParseType;
                    self.buf.clear();
                    parsed_item = Some(Message::BulkString(None));
                } else if self.is_line_end(byte) {
                    self.bulk_string_size = self.parse_buffer_as_size();
                    self.state = State::ReadBulkStringContent;
                    self.buf.clear();
                }
            }
            State::ReadBulkStringContent => {
                if self.buf.len() == self.bulk_string_size {
                    parsed_item = Some(Message::BulkString(Some(self.buf.to_owned())));
                    self.buf.clear();
                    self.state = State::ParseType;
                } else {
                    self.buf.push(byte);
                }
            }
            State::ParseSimpleString => {
                self.buf.push(byte);
                if self.is_line_end(byte) {
                    parsed_item = Some(Message::SimpleString(
                        self.parse_buffer_as_str().to_string(),
                    ));
                    self.buf.clear();
                    self.state = State::ParseType;
                }
            }
            State::ParseError => {
                self.buf.push(byte);
                if self.is_line_end(byte) {
                    parsed_item = Some(Message::Error(self.parse_buffer_as_str().to_string()));
                    self.buf.clear();
                    self.state = State::ParseType;
                }
            }
            State::ParseInteger => {
                self.buf.push(byte);
                if self.is_line_end(byte) {
                    parsed_item = Some(Message::Integer(self.parse_buffer_as_int()));
                    self.buf.clear();
                    self.state = State::ParseType;
                }
            }
        }

        if let Some(item) = parsed_item {
            self.result = self.process_parsed_item(item);
            if self.result.is_some() && self.is_line_end(byte) {
                return true;
            }
        }

        self.prev_byte = byte;
        false
    }

    // puts parsed message to current array on stack
    // if array is completed recursively checks stack
    // then stack is empty returns Some(message)
    fn process_parsed_item(&mut self, message: Message) -> Option<Message> {
        if let Some(arr) = self.array_stack.last_mut() {
            arr.0.push(message);
            if arr.0.len() == arr.1 {
                let array_message = Message::Array(Some(self.array_stack.pop().unwrap().0));
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

    fn parse_buffer_as_str(&self) -> &str {
        std::str::from_utf8(&self.buf)
            .map(|s| s.trim_end())
            .unwrap()
    }

    fn parse_buffer_as_size(&self) -> usize {
        self.parse_buffer_as_str().parse().unwrap()
    }

    fn parse_buffer_as_int(&self) -> i64 {
        self.parse_buffer_as_str().parse().unwrap()
    }

    fn is_line_end(&mut self, byte: u8) -> bool {
        self.prev_byte == b'\r' && byte == b'\n'
    }
}

// fn handle_client(stream: &mut TcpStream) {
//     let mut buf: Vec<u8> = Vec::new();
//     let mut prev_byte: u8 = 0;
//     for byte in stream.bytes() {
//         if let Ok(byte) = byte {
//             if prev_byte == b'\r' && byte == b'\n' {
//                 println!(
//                     "[TCP] Recieved part: {:?}",
//                     std::str::from_utf8(&buf).unwrap_or("<...>")
//                 );
//                 buf.clear();
//             } else {
//                 buf.push(byte);
//             }

//             prev_byte = byte;
//         } else {
//             println!("[TCP] Failed to read byte");
//         }
//     }
//     println!("[TCP] Connection closed");
// }

fn main() -> std::io::Result<()> {
    //     let listener = TcpListener::bind("127.0.0.1:6379")?;

    //     for stream in listener.incoming() {
    //         println!("[TCP] Stream opened");
    //         let mut str = stream.unwrap();
    //         handle_client(&mut str);
    //     }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_string(string: &str) -> Message {
        let mut parser = MessageParser::new();
        for (i, byte) in string.as_bytes().iter().enumerate() {
            assert_eq!(
                parser.add_byte(*byte),
                i == string.len() - 1,
                "Received: {:?}",
                *byte as char
            );
        }
        parser.value().unwrap()
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

    #[test]
    fn test_message_equality() {
        assert_ne!(
            Message::SimpleString("OK".to_string()),
            Message::Array(Some(vec![Message::SimpleString("OK".to_string())]))
        );
    }
}
