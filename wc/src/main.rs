use std::{env, fs::File, io::{BufRead, BufReader, Read}, process::exit};
use utf8_chars::BufReadCharsExt;

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
        eprintln!("Error: no arguments");
        eprintln!("Usage: `wc -c file.txt` or `wc -l file.txt` ");
        exit(1);
    }

    let file = File::open(path.clone()).unwrap();

    match command.as_str() {
        "-c" => {
            let len = file.metadata().unwrap().len();

            println!("{len} {path}");
        }
        "-l" => {
            let mut lines = 0;
            let mut buf = String::new();
            let mut reader = BufReader::new(file);

            // TODO: optimize for memory. What if file contains single line gigabytes long?
            while reader.read_line(&mut buf).unwrap() != 0 {
                lines += 1;
                buf.clear();
            }

            println!("{lines} {path}");
        }
        "-w" => {
            let mut words = 0;
            let mut buf = String::new();
            let mut reader = BufReader::new(file);

            // TODO: optimize for memory. What if file contains single line gigabytes long?
            while reader.read_line(&mut buf).unwrap() != 0 {
                words += buf.split_whitespace().count();
                buf.clear();
            }

            println!("{words} {path}");
        }
        "-m" => {
            let mut reader = BufReader::new(file);

            let chars = reader.chars().count();

            println!("{chars} {path}");
        }
        _ => {
            eprintln!("Unsupported arguments: {command} {path}");
        }
    }
}