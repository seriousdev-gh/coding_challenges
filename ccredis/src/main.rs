use std::{
    io::Read, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}
};

mod resp;
use resp::{message::Message, message_parser::MessageParser};

use std::collections::HashMap;

type Memory = Arc<Mutex<HashMap<String, String>>>;

fn main() -> std::io::Result<()> {
    let memory: Memory = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        println!("[TCP] Stream opened");
        let mut str = stream.unwrap();
        handle_client(&mut str, memory.clone());
    }
    Ok(())
}

fn handle_client(stream: &mut TcpStream, memory: Memory) {
    let mut parser = MessageParser::new();
    let mut writer_stream = stream.try_clone().unwrap();
    for byte in stream.bytes() {
        if let Ok(byte) = byte {
            if let Ok(Some(message)) = parser.add_byte(byte) {
                println!("Received request: {:?}", message);
                let response = process_resp_message(message, memory.clone());
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

fn process_resp_command(parts: Vec<Message>, memory: Memory) -> Result<Message, String> {
    let command_text = bulk_as_text(parts.first())?;

    match command_text {
        "PING" => Ok(command_ping()),
        "ECHO" => {
            let argument_text = bulk_as_text(parts.last())?;
            Ok(command_echo(argument_text))
        },
        "set" => {
            let key = bulk_as_text(parts.get(1))?;
            let value = bulk_as_text(parts.get(2))?;

            let mut memory_lock = memory.lock().unwrap();
            memory_lock.insert(key.to_string(), value.to_string());

            Ok(Message::simple_string("OK"))
        }
        "get" => {
            let key = bulk_as_text(parts.get(1))?;

            let memory_lock = memory.lock().unwrap();
            let value = memory_lock.get(key);

            if let Some(value) = value {
                Ok(Message::bulk_string(value))
            } else {
                Ok(Message::BulkString(None))
            }
        }
        _ => Err("Expected command".to_string())
    }    
}

fn process_resp_message(message: Message, memory: Memory) -> Message {
    match message {
        Message::Array(Some(items)) => match process_resp_command(items, memory) {
            Ok(response) => response,
            Err(err_text) => Message::Error(err_text),
        },
        _ => {
            println!("Unprocessable message: {:?}", message);
            Message::Error("Unprocessable message".to_string())
        }
    }
}
