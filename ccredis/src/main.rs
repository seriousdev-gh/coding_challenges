use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};

mod resp;
use resp::{message::Message, message_parser::MessageParser};

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        println!("[TCP] Stream opened");
        let mut str = stream.unwrap();
        handle_client(&mut str);
    }
    Ok(())
}

fn handle_client(stream: &mut TcpStream) {
    let mut parser = MessageParser::new();
    let mut writer_stream = stream.try_clone().unwrap();
    for byte in stream.bytes() {
        if let Ok(byte) = byte {
            if let Ok(Some(message)) = parser.add_byte(byte) {
                println!("Received request: {:?}", message);
                let response = process_resp_message(message);
                println!("Sending response: {:?}", response);
                response.write_to(&mut writer_stream).unwrap();
            }
        } else {
            println!("[TCP] Failed to read byte");
        }
    }
    println!("[TCP] Connection closed");
}

fn bulk_as_text(message: Option<&Message>) -> Result<&str, String> {
    match message {
        Some(Message::BulkString(Some(content))) => {
            std::str::from_utf8(content).map_err(|_| "Invalid command".to_string())
        },
        _ => Err("Expected bulk string".to_string()),
    }
}

fn command_ping() -> Message {
    Message::simple_string("PONG")
}

fn command_echo(request: &str) -> Message {
    Message::bulk_string(request)
}

fn process_resp_command(parts: Vec<Message>) -> Result<Message, String> {
    let command_text = bulk_as_text(parts.first())?;

    match command_text {
        "PING" => Ok(command_ping()),
        "ECHO" => {
            let argument_text = bulk_as_text(parts.last())?;
            Ok(command_echo(argument_text))
        },
        _ => Err("Expected command".to_string())
    }    
}

fn process_resp_message(message: Message) -> Message {
    match message {
        Message::Array(Some(items)) => match process_resp_command(items) {
            Ok(response) => response,
            Err(err_text) => Message::Error(err_text),
        },
        _ => {
            println!("Unprocessable message: {:?}", message);
            Message::Error("Unprocessable message".to_string())
        }
    }
}
