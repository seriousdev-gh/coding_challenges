use std::{cell::Cell, collections::{HashMap, VecDeque}, fs::File, io::BufWriter, sync::{Arc, RwLock}};

use crate::{processing_error::ProcessingError, resp::message::Message};

#[derive(Debug, PartialEq)]
pub enum Value {
    Single(Vec<u8>),
    List(VecDeque<Vec<u8>>)
}


impl From<&str> for Value {
    fn from(content: &str) -> Self {
        Value::Single(content.into())
    }
}

impl From<String> for Value {
    fn from(content: String) -> Self {
        Value::Single(content.into())
    }
}

pub type SharedMemory = Arc<RwLock<HashMap<String, Value>>>;
pub type KeyExpiration = Arc<RwLock<HashMap<String, u128>>>;


pub struct MessageProcessor {
    pub memory: SharedMemory, 
    pub key_expiration: KeyExpiration,
    pub db_file_path: String,
}

impl MessageProcessor {
    pub fn process_resp_message(&self, message: &Message) -> Message {
        match message {
            Message::Array(Some(items)) => match self.process_resp_command(items) {
                Ok(response) => response,
                Err(err_text) => Message::Error(err_text.to_string()),
            },
            _ => {
                println!("Unprocessable message: {:?}", message);
                Message::Error("Unprocessable message".to_string())
            }
        }
    }

    fn process_resp_command(&self, parts: &Vec<Message>) -> Result<Message, ProcessingError> {
        let (command, args) = split_to_command_args(&parts)?;

        match command.as_str()?.to_lowercase().as_str() {
            "ping" => Ok(self.command_ping()),
            "echo" => self.command_echo(args),
            "set" => self.command_set(args),
            "get" => self.command_get(args),
            "exists" => self.command_exists(args),
            "del" => self.command_del(args),
            "incr" => self.command_incr(args),
            "decr" => self.command_decr(args),
            "lpush" => self.command_lpush(args),
            "rpush" => self.command_rpush(args),
            "save" => self.command_save(),
            _ => Err(ProcessingError::from("Expected command"))
        }    
    }

    fn command_ping(&self) -> Message {
        Message::simple_string("PONG")
    }

    fn command_echo(&self, args: &[Message]) -> Result<Message, ProcessingError> {
        if args.len() != 1 {
            return Err(ProcessingError::from(format!("[echo] expected 1 argument but got {}", args.len())));
        }
        let argument_text = args.first().unwrap().as_str()?;

        Ok(Message::bulk_string(argument_text))
    }

    fn command_set(&self, args: &[Message]) -> Result<Message, ProcessingError> {
        let key = args.get(0).ok_or("[set] expected key")?.as_str()?;
        let value = args.get(1).ok_or("[set] expected value")?.extract_bulk_content()?;
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

        self.insert(key, value, expire_timestamp);

        Ok(Message::simple_string("OK"))
    }

    fn command_get(&self, args: &[Message]) -> Result<Message, ProcessingError> {
        let key = args.get(0).ok_or("[set] expected key")?.as_str()?;

        if !self.check_expiration(key) {
            return Ok(Message::BulkString(None));
        }

        let memory_read_lock = self.memory.read().expect("Memory lock poisoned");
        let value = memory_read_lock.get(key);

        match value {
            Some(Value::Single(bulk_string_content)) => Ok(Message::BulkString(Some(bulk_string_content.clone()))),
            Some(Value::List(_)) => Err("Wrong type. Expected single element, got list.".into()),
            None => Ok(Message::BulkString(None))
        }
    }

    fn command_exists(&self, args: &[Message]) -> Result<Message, ProcessingError> {
        let mut count = 0;
        for arg in args {
            let key = arg.as_str()?;

            if !self.check_expiration(key) {
                continue;
            }

            let memory_read_lock = self.memory.read().expect("Memory lock poisoned");
            if memory_read_lock.contains_key(key) {
                count += 1;
            }
        }
        Ok(Message::Integer(count))
    }

    fn command_del(&self, args: &[Message]) -> Result<Message, ProcessingError> {
        let mut removed = 0;
        for arg in args {
            let key = arg.as_str()?;

            if self.remove(key) {
                removed += 1;
            }
        }
        Ok(Message::Integer(removed))
    }

    fn command_incr(&self, args: &[Message]) -> Result<Message, ProcessingError> {
        let key = args.get(0).ok_or("[incr] expected key")?.as_str()?;

        if !self.check_expiration(key) {
            return Ok(Message::BulkString(None));
        }

        let mut memory_write_lock = self.memory.write().expect("Memory lock poisoned");
        let counter = memory_write_lock.entry(key.to_string()).or_insert("0".into());
        if let Value::Single(counter) = counter {
            let integer = std::str::from_utf8(counter).map_err(|_| ProcessingError::InvalidUtf8)?
                                        .parse::<i64>().map_err(|_| ProcessingError::InvalidInteger)? + 1;
            *counter = integer.to_string().into();

            Ok(Message::Integer(integer))
        } else {
            Err("Wrong type. Expected single element, got list.".into())
        }
    }
    
    fn command_decr(&self, args: &[Message]) -> Result<Message, ProcessingError> {
        let key = args.get(0).ok_or("[decr] expected key")?.as_str()?;

        if !self.check_expiration(key) {
            return Ok(Message::BulkString(None));
        }

        let mut memory_write_lock = self.memory.write().expect("Memory lock poisoned");
        let counter = memory_write_lock.entry(key.to_string()).or_insert("0".into());
        if let Value::Single(counter) = counter {
            let integer = std::str::from_utf8(counter).map_err(|_| ProcessingError::InvalidUtf8)?
                                        .parse::<i64>().map_err(|_| ProcessingError::InvalidInteger)? - 1;
            *counter = integer.to_string().into();

            Ok(Message::Integer(integer))
        } else {
            Err("Wrong type. Expected single element, got list.".into())
        }
    }

    fn command_lpush(&self, args: &[Message]) -> Result<Message, ProcessingError> {
        if args.len() <= 1 {
            return Err("[lpush] Expected at least two arguments: key, and list element".into());
        }


        let (key_message, element_messages) = split_to_command_args(args)?;
        let key = key_message.as_str()?;

        self.check_expiration(key);

        let mut memory_write_lock = self.memory.write().expect("Memory lock poisoned");
        let value = memory_write_lock.get_mut(key);

        match value {
            Some(Value::Single(_)) => return Err("Wrong type. Expected list element, got single.".into()),
            Some(Value::List(list)) => {
                for element in element_messages {
                    list.push_front(element.extract_bulk_content()?.clone());
                }
                return Ok(Message::Integer(list.len() as i64));
            },
            None => {
                let mut list: VecDeque<Vec<u8>> = VecDeque::new();
                for element in element_messages {
                    list.push_front(element.extract_bulk_content()?.clone());
                }
                let length = list.len();
                memory_write_lock.insert(key.to_string(), Value::List(list));
                return Ok(Message::Integer(length as i64));
            }
        }
    }


    fn command_rpush(&self, args: &[Message]) -> Result<Message, ProcessingError> {
        if args.len() <= 1 {
            return Err("[lpush] Expected at least two arguments: key, and list element".into());
        }


        let (key_message, element_messages) = split_to_command_args(args)?;
        let key = key_message.as_str()?;

        self.check_expiration(key);

        let mut memory_write_lock = self.memory.write().expect("Memory lock poisoned");
        let value = memory_write_lock.get_mut(key);

        match value {
            Some(Value::Single(_)) => return Err("Wrong type. Expected list element, got single.".into()),
            Some(Value::List(list)) => {
                for element in element_messages {
                    list.push_back(element.extract_bulk_content()?.clone());
                }
                return Ok(Message::Integer(list.len() as i64));
            },
            None => {
                let mut list: VecDeque<Vec<u8>> = VecDeque::new();
                for element in element_messages {
                    list.push_back(element.extract_bulk_content()?.clone());
                }
                let length = list.len();
                memory_write_lock.insert(key.to_string(), Value::List(list));
                return Ok(Message::Integer(length as i64));
            }
        }
    }

    fn command_save(&self) -> Result<Message, ProcessingError> {
        let file = File::create(self.db_file_path.clone()).map_err(|_| ProcessingError::Other("Cannot open the file for write".to_string()))?;
        let lock = self.memory.write().expect("Memory lock poisoned");
        let key_expiration_lock = self.key_expiration.write().expect("Memory lock poisoned");
        
        // format: 
        // i64 - number of keys, [
        //   i64 - key_length, [u8 * key_length] - key, 
        //   i64 - number of elements (0 for single value, >0 for lists),
        //   [i64 - value_length, [u8 * value_length] - value] * number of elements
        // ] * number of keys
        // file.write(&(lock.len() as i64).to_be_bytes());
        // for (key, value) in lock.iter() {
        //     file.write(&(key.len() as i64).to_be_bytes());
        //     file.write(&key.as_bytes());
        //     match value {
        //         Value::Single(content) => {
        //             file.write(&0_i64.to_be_bytes());
        //             file.write(&(content.len() as i64).to_be_bytes());
        //             file.write(&content);
        //         },
        //         Value::List(list) => {
        //             file.write(&(list.len() as i64).to_be_bytes());
        //             for element in list {
        //                 file.write(&(element.len() as i64).to_be_bytes());
        //                 file.write(&element);
        //             }
        //         }
        //     }
        // }

        let mut messages: Vec<Message> = Vec::new();
        for (key, value) in lock.iter() {
            let mut command: Vec<Message> = Vec::new();
            match value {
                Value::Single(content) => {
                    command.push(Message::bulk_string("SET"));
                    command.push(Message::bulk_string(key));
                    command.push(Message::BulkString(Some(content.clone())));

                    if let Some(expire_at) = key_expiration_lock.get(key) {
                        command.push(Message::bulk_string("PXAT"));
                        command.push(Message::bulk_string(&expire_at.to_string()));
                    }
                },
                Value::List(list) => {
                    command.push(Message::bulk_string("RPUSH"));
                    command.push(Message::bulk_string(key));
                    for element in list {
                        command.push(Message::BulkString(Some(element.clone())));
                    }
                }
            }
            messages.push(Message::Array(Some(command)));
        }

        Message::Array(Some(messages)).write_to(&mut BufWriter::new(file)).map_err(|_| ProcessingError::Other("Cant write message to writer".to_string()))?;

        Ok(Message::SimpleString("OK".to_string()))
    }

    fn insert(&self, key: &str, value: &Vec<u8>, expire_at: Option<u128>) {
        let mut memory_lock = self.memory.write().expect("Memory lock poisoned");
        memory_lock.insert(key.to_string(), Value::Single(value.clone()));

        let mut key_expiration_lock = self.key_expiration.write().expect("Memory lock poisoned");
        if let Some(expire_timestamp) = expire_at {
            key_expiration_lock.insert(key.to_string(), expire_timestamp);
        } else {
            key_expiration_lock.remove(key);
        }
    }

    // fn fetch(&self, key: &str) -> Option<String> {
    //     if !self.check_expiration(key) {
    //         return None;
    //     }

    //     let memory_read_lock = self.memory.read().expect("Memory lock poisoned");
    //     memory_read_lock.get(key)
    // }
    
    fn remove(&self, key: &str) -> bool {
        let mut existed = false;
        
        if self.memory
            .write()
            .expect("Memory lock poisoned")
            .remove(key)
            .is_some() {
                existed = true;
            };
        self.key_expiration
            .write()
            .expect("Memory lock poisoned")
            .remove(key);

        existed
    }

    fn check_expiration(&self, key: &str) -> bool {
        let key_expiration_read_lock = self.key_expiration.read().expect("Memory lock poisoned");
        let key_timestamp = key_expiration_read_lock.get(key);
        
        if let Some(&key_timestamp) = key_timestamp {
            drop(key_expiration_read_lock);
            if now() > key_timestamp {
                self.remove(key);
                return false;
            }
        }
        true
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

    fn from_cli(command: &str) -> Message {
        let mut messages: Vec<Message> = Vec::new();
        for string in command.split(' ') {
            messages.push(Message::BulkString(Some(string.to_string().into_bytes())));
        }
        Message::Array(Some(messages))
    }

    fn create_message_processor() -> MessageProcessor {
        let memory: SharedMemory = Arc::new(RwLock::new(HashMap::new()));
        let key_expiration: KeyExpiration = Arc::new(RwLock::new(HashMap::new()));
        let db_file_path = "tmp/db.bin".to_string();
        MessageProcessor { memory, key_expiration, db_file_path }
    }

    #[test]
    fn message_ping() {
        let processor = create_message_processor();
        let request = from_cli("PING");
        let response = processor.process_resp_message(&request);
        assert_eq!(
            response,
            Message::simple_string("PONG")
        );
    }

    #[test]
    fn message_set() {
        let processor = create_message_processor();

        let request = from_cli("SET test_key test_value");
        let response = processor.process_resp_message(&request);

        assert_eq!(response, Message::simple_string("OK"));
        assert_eq!(*processor.memory.read().unwrap().get("test_key").unwrap(), "test_value".into());
    }

    
    #[test]
    fn message_exists() {
        let processor = create_message_processor();
        processor.memory.write().unwrap().insert("foo".to_string(), "bar".into());

        let request = from_cli("EXISTS foo");
        let response = processor.process_resp_message(&request);
        assert_eq!(response, Message::Integer(1));

        let request = from_cli("EXISTS bar");
        let response = processor.process_resp_message(&request);
        assert_eq!(response, Message::Integer(0));
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
        let processor = create_message_processor();

        // Set value with expiration
        let request = Message::array(vec![
            Message::bulk_string("set"),
            Message::bulk_string("test_key"),
            Message::bulk_string("test_value"),
            Message::bulk_string(option_name),
            Message::bulk_string(option_value)
        ]);

        let response = processor.process_resp_message(&request);
        assert_eq!(response, Message::simple_string("OK"));

        // Get value before expiration
        travel_to(before_timestamp);
        let request = from_cli("GET test_key");

        let response = processor.process_resp_message(&request);
        assert_eq!(response, Message::bulk_string("test_value"));

        // Get value after expiration
        travel_to(after_timestamp);
        let request = from_cli("GET test_key");

        let response = processor.process_resp_message(&request);
        assert_eq!(response, Message::BulkString(None));

        // memory is cleared after expire
        assert_eq!(processor.memory.clone().read().unwrap().len(), 0);
        assert_eq!(processor.key_expiration.clone().read().unwrap().len(), 0);
    }

    #[test]
    fn test_incr() {
        let processor = create_message_processor();
        processor.memory.write().unwrap().insert("foo".to_string(), "68".into());

        let request = from_cli("INCR foo");
        let response = processor.process_resp_message(&request);
        assert_eq!(response, Message::Integer(69));
    }

    #[test]
    fn test_decr() {
        let processor = create_message_processor();
        processor.memory.write().unwrap().insert("foo".to_string(), "70".into());

        let request = from_cli("DECR foo");
        let response = processor.process_resp_message(&request);
        assert_eq!(response, Message::Integer(69));
    }

    #[test]
    fn test_lpush() {
        let processor = create_message_processor();
        let request = from_cli("LPUSH foo 1 2 3");

        let response = processor.process_resp_message(&request);
        assert_eq!(response, Message::Integer(3));

        let lock = processor.memory.read().unwrap();
        if let Some(Value::List(list)) = lock.get("foo") {
            assert_eq!(*list, VecDeque::from([Vec::from("3".as_bytes()), Vec::from("2".as_bytes()), Vec::from("1".as_bytes())]))
        } else {
            unreachable!("Expected list");
        }
    }

    #[test]
    fn test_rpush() {
        let processor = create_message_processor();
        let request = from_cli("RPUSH foo 1 2 3");

        let response = processor.process_resp_message(&request);
        assert_eq!(response, Message::Integer(3));

        let lock = processor.memory.read().unwrap();
        if let Some(Value::List(list)) = lock.get("foo") {
            assert_eq!(*list, VecDeque::from([Vec::from("1".as_bytes()), Vec::from("2".as_bytes()), Vec::from("3".as_bytes())]))
        } else {
            unreachable!("Expected list");
        }
    }

        #[test]
    fn test_rpush_with_append() {
        let processor = create_message_processor();

        let request = from_cli("RPUSH foo 1");
        let response = processor.process_resp_message(&request);
        assert_eq!(response, Message::Integer(1));

        
        let request = from_cli("RPUSH foo 2 3");
        let response = processor.process_resp_message(&request);
        assert_eq!(response, Message::Integer(3));

        let lock = processor.memory.read().unwrap();
        if let Some(Value::List(list)) = lock.get("foo") {
            assert_eq!(*list, VecDeque::from([Vec::from("1".as_bytes()), Vec::from("2".as_bytes()), Vec::from("3".as_bytes())]))
        } else {
            unreachable!("Expected list");
        }
    }
}