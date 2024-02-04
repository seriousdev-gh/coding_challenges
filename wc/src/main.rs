use std::{env, fs::File};

fn main() {
    let mut path = String::from("");
    let mut command = String::from(""); 

    for arg in env::args().skip(1) {
        if arg.starts_with("-") {
            command = arg;
        } else {
            path = arg;
        }
    }

    match command.as_str() {
        "-c" => {
            let file = File::open(path.clone()).unwrap();
            let len = file.metadata().unwrap().len();
            println!("{} {}", len, path);
        }
        _ => {
            println!("Unsupported arguments: {} {}", command, path);
        }
    }
}


