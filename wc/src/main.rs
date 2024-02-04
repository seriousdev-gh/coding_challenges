use std::{env, fs::File, io::{BufRead, BufReader}, process::exit};

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

    if command == "" || path == "" {
        println!("Error: no argumants");
        println!("Usage: `wc -c file.txt` or `wc -l file.txt` ");
        exit(1);
    }

    let file = File::open(path.clone()).unwrap();

    match command.as_str() {
        "-c" => {
            let len = file.metadata().unwrap().len();
            println!("{} {}", len, path);
        }
        "-l" => {
            let mut lines = 0;
            let mut buf = String::new();
            let mut reader = BufReader::new(file);
            while reader.read_line(&mut buf).unwrap() != 0 {
                lines += 1;
                buf.clear();
            }
            println!("{} {}", lines, path);
        }
        _ => {
            println!("Unsupported arguments: {} {}", command, path);
        }
    }
}
