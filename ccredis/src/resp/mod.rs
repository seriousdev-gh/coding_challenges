pub mod message;
pub mod message_parser;
pub mod parse_error;


#[cfg(test)]
mod tests {
    use message::Message;
    use message_parser::MessageParser;

    use super::*;

    fn serialize_deserialize(string: &str) -> String {
        let mut parser = MessageParser::new();
        let mut message: Option<Message> = None;
        for byte in string.as_bytes() {
            message = parser.add_byte(*byte).unwrap();
        }
        let mut buf: Vec<u8> = Vec::new();
        message.unwrap().write_to(&mut buf).unwrap();

        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn parse_bulk_string() {
        let expected_text = "$5\r\nhello\r\n";
        let actual_text = serialize_deserialize(expected_text);
        assert_eq!(expected_text, actual_text);
    }

    #[test]
    fn parse_simple_string() {
        let expected_text = "+OK\r\n";
        let actual_text = serialize_deserialize(expected_text);
        assert_eq!(expected_text, actual_text);
    }

    #[test]
    fn parse_one_element_array() {
        let expected_text = "*1\r\n$4\r\nping\r\n";
        let actual_text = serialize_deserialize(expected_text);
        assert_eq!(expected_text, actual_text);
    }

    #[test]
    fn parse_two_element_array_with_bulk_strings() {
        let expected_text = "*2\r\n$4\r\necho\r\n$11\r\nhello world\r\n";
        let actual_text = serialize_deserialize(expected_text);
        assert_eq!(expected_text, actual_text);
    }

    #[test]
    fn parse_empty_bulk_string() {
        let expected_text = "$0\r\n\r\n";
        let actual_text = serialize_deserialize(expected_text);
        assert_eq!(expected_text, actual_text);
    }

    #[test]
    fn parse_null_bulk_string() {
        let expected_text = "$-1\r\n";
        let actual_text = serialize_deserialize(expected_text);
        assert_eq!(expected_text, actual_text);
    }

    #[test]
    fn parse_empty_array() {
        let expected_text = "*0\r\n";
        let actual_text = serialize_deserialize(expected_text);
        assert_eq!(expected_text, actual_text);
    }

    #[test]
    fn parse_null_array() {
        let expected_text = "*-1\r\n";
        let actual_text = serialize_deserialize(expected_text);
        assert_eq!(expected_text, actual_text);
    }

    #[test]
    fn parse_nested_array() {
        let expected_text = "*2\r\n+baz\r\n*2\r\n+foo\r\n*1\r\n+bar\r\n";
        let actual_text = serialize_deserialize(expected_text);
        assert_eq!(expected_text, actual_text);
    }

    #[test]
    fn parse_error() {
        let expected_text = "-Error message\r\n";
        let actual_text = serialize_deserialize(expected_text);
        assert_eq!(expected_text, actual_text);
    }

    #[test]
    fn parse_negative_integer() {
        let expected_text = ":-1\r\n";
        let actual_text = serialize_deserialize(expected_text);
        assert_eq!(expected_text, actual_text);
    }

    #[test]
    fn parse_array_of_all_types() {
        let expected_text = "*4\r\n$4\r\nbulk\r\n+simple\r\n:-1\r\n-err\r\n";
        let actual_text = serialize_deserialize(expected_text);
        assert_eq!(expected_text, actual_text);
    }
}
