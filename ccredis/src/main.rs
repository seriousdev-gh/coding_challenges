use std::{
    env, io::{BufReader, BufWriter, Read, Write}, net::{TcpListener, TcpStream}, sync::{atomic::{self, AtomicBool, Ordering}, Arc, RwLock}
};

mod resp;
mod message_processor;
use message_processor::{process_resp_message, KeyExpiration, SharedMemory};
mod processing_error;
use resp::message_parser::MessageParser;

use std::collections::HashMap;


static DEBUG: AtomicBool = AtomicBool::new(false);

fn main() -> std::io::Result<()> {
    set_globals();

    let memory: SharedMemory = Arc::new(RwLock::new(HashMap::new()));
    let key_expiration: KeyExpiration = Arc::new(RwLock::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        let memory = memory.clone();
        let key_expiration = key_expiration.clone();
        std::thread::spawn(move || {
            match stream {
                Ok(mut str) => handle_client(&mut str, memory, key_expiration),
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

fn handle_client(stream: &mut TcpStream, memory: SharedMemory, key_expiration: KeyExpiration) {
    println!("[TCP] Client connected");
    let mut parser = MessageParser::new();
    let mut writer_stream = BufWriter::new(stream.try_clone().unwrap());
    for byte in BufReader::new(stream).bytes() {
        if let Ok(byte) = byte {
            match parser.add_byte(byte) {
                Ok(Some(message)) => {
                    debug(&format!("Received request: {:?}", message));
                    let response = process_resp_message(&message, memory.clone(), key_expiration.clone());
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

fn debug(log: &str) {
    if DEBUG.load(atomic::Ordering::Relaxed) {
        println!("[Debug]{}", log);
    }
}