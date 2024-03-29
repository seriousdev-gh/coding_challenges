use std::{env, fs::File, io::{BufRead, BufReader}, process::exit};
use utf8_chars::BufReadCharsExt;

fn print_usage() {
    println!("Usage: wc [options] <file>");
    println!("With no options specifies prints lines, words and bytes count");
    println!("  -c  count bytes");
    println!("  -l  count lines");
    println!("  -w  count words");
    println!("  -m  count characters");

    exit(1);
}

fn main() {
    let mut file = String::from("");
    let mut command = String::from("");

    for arg in env::args().skip(1) {
        if arg.starts_with("-") {
            command = arg;
        } else {
            file = arg;
        }
    }

    if command == "" && file == "" {
        eprintln!("Error: arguments not provided.");
        print_usage();
    }

    if file == "" {
        eprintln!("Error: file not provided.");
        print_usage();
    }


    match command.as_str() {
        "-c" => {
            let bytes = file_byte_count(&file);
            println!("{bytes} {file}");
        }
        "-l" => {
            let lines = file_line_count(&file);
            println!("{lines} {file}");
        }
        "-w" => {
            let words = file_word_count(&file);
            println!("{words} {file}");
        }
        "-m" => {
            let chars = file_character_count(&file);
            println!("{chars} {file}");
        }
        "" => {
            let bytes = file_byte_count(&file);
            let lines = file_line_count(&file);
            let words = file_word_count(&file);
            println!("{lines} {words} {bytes} {file}");
        }
        _ => {
            eprintln!("Unsupported arguments: {command} {file}");
        }
    }
}

fn file_character_count(path: &String) -> usize {
    let file = File::open(path.clone()).unwrap();
    let mut reader = BufReader::new(file);

    let chars = reader.chars().count();
    chars
}

fn file_line_count(path: &String) -> usize {
    let file = File::open(path.clone()).unwrap();
    let mut lines = 0;
    let mut buf = String::new();
    let mut reader = BufReader::new(file);

    // TODO: optimize for memory. What if file contains single line gigabytes long?
    while reader.read_line(&mut buf).unwrap() != 0 {
        lines += 1;
        buf.clear();
    }
    lines
}

fn file_word_count(path: &String) -> usize {
    let file = File::open(path.clone()).unwrap();
    let mut words = 0;
    let mut buf = String::new();
    let mut reader = BufReader::new(file);

    // TODO: optimize for memory. What if file contains single line gigabytes long?
    while reader.read_line(&mut buf).unwrap() != 0 {
        words += buf.split_whitespace().count();
        buf.clear();
    }
    words
}

fn file_byte_count(path: &String) -> usize {
    let file = File::open(path.clone()).unwrap();
    let len = file.metadata().unwrap().len();
    len as usize
}
