use std::{cell::Cell, collections::HashMap, sync::{Arc, RwLock}};

use crate::{processing_error::ProcessingError, resp::message::Message};

pub type SharedMemory = Arc<RwLock<HashMap<String, String>>>;
pub type KeyExpiration = Arc<RwLock<HashMap<String, u128>>>;


pub fn process_resp_message(message: &Message, memory: SharedMemory, key_expiration: KeyExpiration) -> Message {
    match message {
        Message::Array(Some(items)) => match process_resp_command(&items, memory, key_expiration) {
            Ok(response) => response,
            Err(err_text) => Message::Error(err_text.to_string()),
        },
        _ => {
            println!("Unprocessable message: {:?}", message);
            Message::Error("Unprocessable message".to_string())
        }
    }
}

fn process_resp_command(parts: &Vec<Message>, memory: SharedMemory, key_expiration: KeyExpiration) -> Result<Message, ProcessingError> {
    let (command, args) = split_to_command_args(&parts)?;

    match command.as_str()?.to_lowercase().as_str() {
        "ping" => Ok(command_ping()),
        "echo" => command_echo(args),
        "set" => command_set(args, memory, key_expiration),
        "get" => command_get(args, memory, key_expiration),
        _ => Err(ProcessingError::from("Expected command"))
    }    
}

fn command_ping() -> Message {
    Message::simple_string("PONG")
}

fn command_echo(args: &[Message]) -> Result<Message, ProcessingError> {
    if args.len() != 1 {
        return Err(ProcessingError::from(format!("[echo] expected 1 argument but got {}", args.len())));
    }
    let argument_text = args.first().unwrap().as_str()?;

    Ok(Message::bulk_string(argument_text))
}

fn command_set(args: &[Message], memory: SharedMemory, key_expiration: KeyExpiration) -> Result<Message, ProcessingError> {
    let key = args.get(0).ok_or("[set] expected key")?.as_str()?;
    let value = args.get(1).ok_or("[set] expected value")?.as_str()?;
    let expire_type = args.get(2);
    let expire_value = args.get(3);

    let mut expire_timestamp: Option<u128> = None;

    if let Some(expire_type) = expire_type {
        let expire_value_str = expire_value.ok_or("[set] expected value when using expire")?.as_str()?;
        let expire_value_parsed: u128 = expire_value_str.parse().map_err(|_| ProcessingError::InvalidInteger)?;

        match expire_type.as_str()?.to_lowercase().as_str() {
            "ex" => {
                expire_timestamp = Some(now() + (expire_value_parsed as u128) * 1000);
            },
            "px" => {
                expire_timestamp = Some(now() + expire_value_parsed as u128);
            },
            "exat" => {
                expire_timestamp = Some(expire_value_parsed as u128 * 1000);
            },
            "pxat" => {
                expire_timestamp = Some(expire_value_parsed as u128);
            },
            arg => {
                return Err(ProcessingError::from(format!("[set] unsupported arg: {}", arg)));
            }
        }
    }

    let mut memory_lock = memory.write().expect("Memory lock poisoned");
    memory_lock.insert(key.to_string(), value.to_string());
    
    if let Some(expire_timestamp) = expire_timestamp {
        let mut key_expiration_lock = key_expiration.write().expect("Memory lock poisoned");
        key_expiration_lock.insert(key.to_string(), expire_timestamp);
    }

    Ok(Message::simple_string("OK"))
}

fn command_get(args: &[Message], memory: SharedMemory, key_expiration: KeyExpiration) -> Result<Message, ProcessingError> {
    let key = args.get(0).ok_or("[set] expected key")?.as_str()?;

    let key_expiration_read_lock = key_expiration.read().expect("Memory lock poisoned");
    let key_timestamp = key_expiration_read_lock.get(key);
    
    if let Some(&key_timestamp) = key_timestamp {
        drop(key_expiration_read_lock);
        if now() > key_timestamp {
            key_expiration
                .write()
                .expect("Memory lock poisoned")
                .remove(key);
            memory
                .write()
                .expect("Memory lock poisoned")
                .remove(key);
            return Ok(Message::BulkString(None));
        }
    }

    let memory_read_lock = memory.read().expect("Memory lock poisoned");
    let value = memory_read_lock.get(key);

    if let Some(value) = value {
        Ok(Message::bulk_string(value))
    } else {
        Ok(Message::BulkString(None))
    }
}

fn split_to_command_args<T>(vec: &[T]) -> Result<(&T, &[T]), ProcessingError> {
    match vec.split_first() {
        Some((head, tail)) => Ok((head, tail)),
        None => Err(ProcessingError::from("Vector is empty"))
    }
}

#[cfg(not(test))]
pub fn now() -> u128 {
    std::time::UNIX_EPOCH.elapsed().unwrap().as_millis()
}

#[cfg(test)]
pub fn now() -> u128 {
    TIMESTAMP.with(|timestamp| timestamp.get() )
}

thread_local! {
    static TIMESTAMP: Cell<u128> = const { Cell::new(0) };
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn travel_to(timestamp: u128) {
        TIMESTAMP.with(|ts| ts.set(timestamp));
    }

    #[test]
    fn message_ping() {
        let memory: SharedMemory = Arc::new(RwLock::new(HashMap::new()));
        let expires: KeyExpiration = Arc::new(RwLock::new(HashMap::new()));
        let request = Message::array(vec![Message::bulk_string("ping")]);
        let response = process_resp_message(&request, memory, expires);
        assert_eq!(
            response,
            Message::simple_string("PONG")
        );
    }

    #[test]
    fn message_set() {
        let memory: SharedMemory = Arc::new(RwLock::new(HashMap::new()));
        let expires: KeyExpiration = Arc::new(RwLock::new(HashMap::new()));
        let request = Message::array(vec![
            Message::bulk_string("set"),
            Message::bulk_string("test_key"),
            Message::bulk_string("test_value")
        ]);
        let response = process_resp_message(&request, memory.clone(), expires.clone());
        assert_eq!(response, Message::simple_string("OK"));
        assert_eq!(memory.clone().read().unwrap().get("test_key").unwrap(), "test_value");
    }

    #[test]
    fn test_expiration_options_ex() {
        let init_timestamp = std::time::UNIX_EPOCH.elapsed().unwrap().as_millis();
        message_set_get_with_options(
            "ex", 
            "30", 
            init_timestamp, 
            init_timestamp + 29_000, 
            init_timestamp + 31_000
        );
    }

    #[test]
    fn test_expiration_options_px() {
        let init_timestamp = std::time::UNIX_EPOCH.elapsed().unwrap().as_millis();
        message_set_get_with_options(
            "px", 
            "30000", 
            init_timestamp, 
            init_timestamp + 29_000, 
            init_timestamp + 31_000
        );
    }

    #[test]
    fn test_expiration_options_exat() {
        let init_timestamp = std::time::UNIX_EPOCH.elapsed().unwrap().as_millis();
        let expire_at_timestamp_in_seconds = ((init_timestamp + 30_000) / 1000).to_string();
        message_set_get_with_options(
            "exat", 
            &expire_at_timestamp_in_seconds, 
            init_timestamp, 
            init_timestamp + 29_000, 
            init_timestamp + 31_000
        );
    }

    #[test]
    fn test_expiration_options_pxat() {
        let init_timestamp = std::time::UNIX_EPOCH.elapsed().unwrap().as_millis();
        let expire_at_timestamp_in_milliseconds = (init_timestamp + 30_000).to_string();
        message_set_get_with_options(
            "pxat",  
            &expire_at_timestamp_in_milliseconds, 
            init_timestamp, 
            init_timestamp + 29_000, 
            init_timestamp + 31_000
        );
    }

    fn message_set_get_with_options(option_name: &str, option_value: &str, init_timestamp: u128, before_timestamp: u128, after_timestamp: u128) {
        // Setup
        travel_to(init_timestamp);
        let memory: SharedMemory = Arc::new(RwLock::new(HashMap::new()));
        let expires: KeyExpiration = Arc::new(RwLock::new(HashMap::new()));

        // Set value with expiration
        let request = Message::array(vec![
            Message::bulk_string("set"),
            Message::bulk_string("test_key"),
            Message::bulk_string("test_value"),
            Message::bulk_string(option_name),
            Message::bulk_string(option_value)
        ]);
        let response = process_resp_message(&request, memory.clone(), expires.clone());
        assert_eq!(response, Message::simple_string("OK"));

        // Get value before expiration
        travel_to(before_timestamp);
        let request = Message::array(vec![
            Message::bulk_string("get"),
            Message::bulk_string("test_key")
        ]);
        let response = process_resp_message(&request, memory.clone(), expires.clone());
        assert_eq!(response, Message::bulk_string("test_value"));

        // Get value after expiration
        travel_to(after_timestamp);
        let request = Message::array(vec![
            Message::bulk_string("get"),
            Message::bulk_string("test_key")
        ]);
        let response = process_resp_message(&request, memory.clone(), expires.clone());
        assert_eq!(response, Message::BulkString(None));

        // memory is cleared after expire
        assert_eq!(memory.clone().read().unwrap().len(), 0);
        assert_eq!(expires.clone().read().unwrap().len(), 0);
    }
}