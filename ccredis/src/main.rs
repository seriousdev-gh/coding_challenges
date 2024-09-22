use core::str;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

#[derive(Debug)]
enum ParseError {
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

struct MessageParser {
    array_stack: Vec<ArrayStackItem>,
    buf: Vec<u8>,
    prev_byte: u8,
    state: State,
    message_type: MessageType,
    bulk_string_size: usize,
}

impl MessageParser {
    fn new() -> Self {
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
    fn add_byte(&mut self, byte: u8) -> Result<Option<Message>, ParseError> {
        println!(
            "add byte: {:?} state: {:?} message_type: {:?} buffer: {:?}",
            byte as char,
            self.state,
            self.message_type,
            str::from_utf8(&self.buf)
        );
        let mut parsed_item: Option<Message> = Option::None;

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
                    self.parse_buffer(&mut parsed_item)?;

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
                    parsed_item = Some(Message::BulkString(Some(self.buf.to_owned())));
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

    fn parse_buffer(&mut self, parsed_item: &mut Option<Message>) -> Result<(), ParseError> {
        self.buf.truncate(self.buf.len().saturating_sub(2));
        println!("Parse buffer: {:?}", str::from_utf8(&self.buf));
        let as_string = self.parse_buffer_as_str()?;
        match self.message_type {
            MessageType::SimpleString => {
                *parsed_item = Some(Message::simple_string(as_string));
            }
            MessageType::Error => {
                *parsed_item = Some(Message::error(as_string));
            }
            MessageType::Integer => {
                *parsed_item = Some(Message::Integer(self.parse_buffer_as_int()?));
            }
            MessageType::BulkString => match as_string {
                "-1" => *parsed_item = Some(Message::BulkString(None)),
                _ => {
                    self.bulk_string_size = self.parse_buffer_as_size()?;
                    self.buf.clear();
                    self.state = State::ReadBulkStringContent;
                }
            },
            MessageType::Array => match as_string {
                "-1" => *parsed_item = Some(Message::Array(None)),
                "0" => *parsed_item = Some(Message::array(Vec::new())),
                _ => {
                    let size = self.parse_buffer_as_size()?;
                    self.array_stack.push(ArrayStackItem {
                        items: Vec::new(),
                        size: size,
                    });
                }
            },
            MessageType::Unknown => {
                return Err(ParseError::Other("Unknown message type".to_string()));
            }
        }
        Ok(())
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

    fn parse_buffer_as_size(&self) -> Result<usize, ParseError> {
        self.parse_buffer_as_str()?
            .parse()
            .map_err(|_| ParseError::InvalidInteger)
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

impl Message {
    fn write_to<W: Write>(self, writer: &mut W) -> std::io::Result<()> {
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

fn process_resp_message(message: Message) -> Option<Message> {
    match message {
        Message::Array(Some(items)) => {
            if items.len() == 1 {
                let item = &items[0];
                println!("Received single item command: {:?}", item);
                match item {
                    Message::BulkString(Some(content)) => {
                        if content == "PING".to_string().as_bytes() {
                            return Some(Message::array(vec![Message::bulk_string("PONG") ]));
                        } else {
                            println!("Unprocessable command: {:?}", content);
                        }
                    }
                    _ => {
                        println!("Unprocessable entity: {:?}", item);
                    }
                }
            }
            if items.len() == 2 {
                let item = &items[0];
                println!("Received two item command: {:?}", item);
                match item {
                    Message::BulkString(Some(content)) => {
                        if content == "ECHO".to_string().as_bytes() {
                            if let Message::BulkString(text) = &items[1] {
                                return Some(Message::array(vec![Message::BulkString(text.clone()) ]));
                            } else {
                                println!("Unprocessable second item: {:?}", &items[1]);
                            }
                        } else {
                            println!("Unprocessable command: {:?}", content);
                        }
                    }
                    _ => {
                        println!("Unprocessable entity: {:?}", item);
                    }
                }
            }
        }
        _ => {
            println!("Unprocessable message: {:?}", message);
        }
    };
    None
}

fn handle_client(stream: &mut TcpStream) {
    let mut parser = MessageParser::new();
    let mut writer_stream = stream.try_clone().unwrap();
    for byte in stream.bytes() {
        if let Ok(byte) = byte {
            if let Ok(Some(message)) = parser.add_byte(byte) {
                println!("Received message: {:?}", message);
                if let Some(response) = process_resp_message(message) {
                    response.write_to(&mut writer_stream).unwrap();
                }
            }
        } else {
            println!("[TCP] Failed to read byte");
        }
    }
    println!("[TCP] Connection closed");
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        println!("[TCP] Stream opened");
        let mut str = stream.unwrap();
        handle_client(&mut str);
    }
    Ok(())
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

    #[test]
    fn test_message_equality() {
        assert_ne!(
            Message::SimpleString("OK".to_string()),
            Message::Array(Some(vec![Message::SimpleString("OK".to_string())]))
        );
    }
}
