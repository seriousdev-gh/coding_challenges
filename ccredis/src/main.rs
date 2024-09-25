use std::{
    env, io::{BufReader, BufWriter, Read, Write}, net::{TcpListener, TcpStream}, sync::{atomic::{self, AtomicBool, Ordering}, Arc, RwLock}
};

mod resp;
use resp::{message::Message, message_parser::MessageParser};

use std::collections::HashMap;

type SharedMemory = Arc<RwLock<HashMap<String, String>>>;

static DEBUG: AtomicBool = AtomicBool::new(false);

fn main() -> std::io::Result<()> {
    set_globals();

    let memory: SharedMemory = Arc::new(RwLock::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        let memory = memory.clone();
        std::thread::spawn(move || {
            match stream {
                Ok(mut str) => handle_client(&mut str, memory),
                Err(e) => eprintln!("[TCP] Error accepting connection: {}", e),
            }
        });
    }
    Ok(())
}

fn set_globals() {
    let log_level = env::var("LOG_LEVEL").unwrap_or_default();
    let is_debug = log_level.eq_ignore_ascii_case("debug");

    DEBUG.store(is_debug, Ordering::Relaxed);
}

fn handle_client(stream: &mut TcpStream, memory: SharedMemory) {
    println!("[TCP] Client connected");
    let mut parser = MessageParser::new();
    let mut writer_stream = BufWriter::new(stream.try_clone().unwrap());
    for byte in BufReader::new(stream).bytes() {
        if let Ok(byte) = byte {
            match parser.add_byte(byte) {
                Ok(Some(message)) => {
                    debug(&format!("Received request: {:?}", message));
                    let response = process_resp_message(&message, memory.clone());
                    debug(&format!("Sending response: {:?}", response));
                    // TODO: handle errors
                    response.write_to(&mut writer_stream).unwrap();
                    writer_stream.flush().unwrap();
                },
                Err(err) => {
                    println!("[Parser] Failed to parse byte [{}]", err);
                },
                Ok(None) => {} // message is not parsed yet
            }
        } else {
            println!("[TCP] Failed to read byte");
        }
    }
    println!("[TCP] Connection closed");
}

fn process_resp_message(message: &Message, memory: SharedMemory) -> Message {
    match message {
        Message::Array(Some(items)) => match process_resp_command(&items, memory) {
            Ok(response) => response,
            Err(err_text) => Message::Error(err_text),
        },
        _ => {
            println!("Unprocessable message: {:?}", message);
            Message::Error("Unprocessable message".to_string())
        }
    }
}

fn process_resp_command(parts: &Vec<Message>, memory: SharedMemory) -> Result<Message, String> {
    let parts_strings = messages_to_strings(parts)?;

    let (&command, args) = split_to_command_args(&parts_strings)?;

    match command.to_lowercase().as_str() {
        "ping" => Ok(command_ping()),
        "echo" => command_echo(args),
        "set" => command_set(args, memory),
        "get" => command_get(args, memory),
        _ => Err("Expected command".to_string())
    }    
}

fn command_ping() -> Message {
    Message::simple_string("PONG")
}

fn command_echo(args: &[&str]) -> Result<Message, String> {
    if args.len() != 1 {
        return Err(format!("[echo] expected 1 argument but got {}", args.len()).to_string());
    }
    let &argument_text = args.first().unwrap();

    Ok(Message::bulk_string(argument_text))
}

fn command_set(args: &[&str], memory: SharedMemory) -> Result<Message, String> {
    if args.len() != 2 {
        return Err(format!("[set] expected 2 arguments but got {}", args.len()).to_string());
    }

    let key = *args.get(0).unwrap();
    let value = *args.get(1).unwrap();

    let mut memory_lock = memory.write().expect("Memory lock poisoned");
    memory_lock.insert(key.to_string(), value.to_string());

    Ok(Message::simple_string("OK"))
}

fn command_get(args: &[&str], memory: SharedMemory) -> Result<Message, String> {
    if args.len() != 1 {
        return Err(format!("[get] expected 1 argument but got {}", args.len()).to_string());
    }

    let key = *args.first().unwrap();

    let memory_lock = memory.read().expect("Memory lock poisoned");
    let value = memory_lock.get(key);

    if let Some(value) = value {
        Ok(Message::bulk_string(value))
    } else {
        Ok(Message::BulkString(None))
    }
}

fn messages_to_strings(parts: &Vec<Message>) -> Result<Vec<&str>, String> {
    let mut result: Vec<&str> = Vec::new();
    for message in parts {
        result.push(bulk_as_text(message)?);
    }
    Ok(result)
}

fn bulk_as_text(message: &Message) -> Result<&str, String> {
    match message {
        Message::BulkString(Some(content)) => {
            std::str::from_utf8(content).map_err(|_| "Invalid utf8".to_string())
        },
        _ => Err("Expected bulk string".to_string()),
    }
}

fn split_to_command_args<T>(vec: &[T]) -> Result<(&T, &[T]), String> {
    match vec.split_first() {
        Some((head, tail)) => Ok((head, tail)),
        None => Err(String::from("Vector is empty"))
    }
}

fn debug(log: &str) {
    if DEBUG.load(atomic::Ordering::Relaxed) {
        println!("[Debug]{}", log);
    }
}