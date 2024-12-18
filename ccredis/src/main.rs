use std::{
    env, fs::File, io::{BufReader, BufWriter, Read, Write}, net::{TcpListener, TcpStream}, sync::{atomic::{self, AtomicBool, Ordering}, Arc, RwLock}, thread, time
};

mod resp;
mod message_processor;
use message_processor::{MessageProcessor, KeyExpiration, SharedMemory};
mod processing_error;
use resp::{message::Message, message_parser::MessageParser};

use std::collections::HashMap;
use rand::seq::IteratorRandom;
use rand::thread_rng;

static DEBUG: AtomicBool = AtomicBool::new(false);

const AMOUNT_OF_KEYS_CHECKED_FOR_EXPIRATION: usize = 8;
const AMOUNT_OF_MILLISECONDS_BETWEEN_EXPIRATION_CHECKS: u64 = 1000;

fn main() -> std::io::Result<()> {
    set_globals();

    let memory: SharedMemory = Arc::new(RwLock::new(HashMap::new()));
    let key_expiration: KeyExpiration = Arc::new(RwLock::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:6379")?;
    let db_file_path = "db.txt";

    let _ = load(memory.clone(), key_expiration.clone(), db_file_path.to_string());

    {
        let memory = memory.clone();
        let key_expiration = key_expiration.clone();
        thread::spawn(move || {
            key_expirer_worker(memory.clone(), key_expiration.clone());
        });
    }

    for stream in listener.incoming() {
        let memory = memory.clone();
        let key_expiration = key_expiration.clone();
        std::thread::spawn(move || {
            match stream {
                Ok(mut str) => handle_client(&mut str, memory, key_expiration, db_file_path.to_string()),
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

fn load(memory: SharedMemory, key_expiration: KeyExpiration, db_file_path: String) -> Result<(), std::io::Error> {
    let file = File::open(db_file_path.clone())?;
    let mut parser = MessageParser::new();
    let message_processor = MessageProcessor { memory, key_expiration, db_file_path };
    for byte in BufReader::new(file).bytes() {
        if let Ok(byte) = byte {
            match parser.add_byte(byte) {
                Ok(Some(Message::Array(Some(commands)))) => {
                    for command in commands {
                        let response = message_processor.process_resp_message(&command);
                        assert_ne!(response.type_as_str(), "Error");
                    }
                    println!("[Load] Memory loaded from file");
                },
                Err(err) => {
                    println!("[Load] Failed to parse byte [{}]", err);
                },
                Ok(None) => {} // message is not parsed yet,
                _ => {
                    println!("[Load] Unknown message type");
                }
            }
        } else {
            println!("[Load] Failed to read byte");
        }
    }
    Ok(())
}

fn handle_client(stream: &mut TcpStream, memory: SharedMemory, key_expiration: KeyExpiration, db_file_path: String) {
    println!("[TCP] Client connected");
    let mut parser = MessageParser::new();
    let message_processor = MessageProcessor { memory, key_expiration, db_file_path };
    let mut writer_stream = BufWriter::new(stream.try_clone().unwrap());
    for byte in BufReader::new(stream).bytes() {
        if let Ok(byte) = byte {
            match parser.add_byte(byte) {
                Ok(Some(message)) => {
                    debug(&format!("Received request: {:?}", message));
                    let response = message_processor.process_resp_message(&message);
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
            return
        }
    }
    println!("[TCP] Connection closed");
}

fn key_expirer_worker(memory: SharedMemory, key_expiration: KeyExpiration) {
    let interval = time::Duration::from_millis(AMOUNT_OF_MILLISECONDS_BETWEEN_EXPIRATION_CHECKS);
    let mut rng = thread_rng();
    loop {
        thread::sleep(interval);

        let expiration_read_lock = key_expiration.read().unwrap();
        let current_timestamp = message_processor::now();
        let mut keys_to_remove: Vec<String> = Vec::new();

        for (key, timestamp) in expiration_read_lock.iter().choose_multiple(&mut rng, AMOUNT_OF_KEYS_CHECKED_FOR_EXPIRATION) {
            if current_timestamp > *timestamp {
                keys_to_remove.push(key.to_string());
            }
        }
        drop(expiration_read_lock);

        if keys_to_remove.len() > 0 {
            let mut expiration_write_lock = key_expiration.write().unwrap();
            let mut memory_write_lock = memory.write().unwrap();
            for key in keys_to_remove {
                expiration_write_lock.remove(&key);
                memory_write_lock.remove(&key);
            }
        }
    }
}

fn debug(log: &str) {
    if DEBUG.load(atomic::Ordering::Relaxed) {
        println!("[Debug]{}", log);
    }
}