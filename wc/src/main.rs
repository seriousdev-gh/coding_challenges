use std::{env, fs::File, io::{self, BufRead, BufReader}, process::exit};


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
    let mut file_path = String::from("");
    let mut command = String::from("");

    for arg in env::args().skip(1) {
        if arg.starts_with("-") {
            command = arg;
        } else {
            file_path = arg;
        }
    }

    match command.as_str() {
        "-c" => {
            let bytes = file_byte_count(&mut create_reader(&file_path));
            println!("{bytes} {file_path}");
        }
        "-l" => {
            let lines = file_line_count(&mut create_reader(&file_path));
            println!("{lines} {file_path}");
        }
        "-w" => {
            let words = file_word_count(&mut create_reader(&file_path));
            println!("{words} {file_path}");
        }
        "-m" => {
            let chars = file_character_count(&mut create_reader(&file_path));
            println!("{chars} {file_path}");
        }
        "" => {
            let bytes = file_byte_count(&mut create_reader(&file_path));
            let lines = file_line_count(&mut create_reader(&file_path));
            let words = file_word_count(&mut create_reader(&file_path));
            println!("{lines} {words} {bytes} {file_path}");
        }
        _ => {
            eprintln!("Unsupported arguments: {command} {file_path}");
            print_usage();
        }
    }
}

fn create_reader(file_path: &String) -> Box<dyn BufRead> {
    if *file_path == "" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        let file = File::open(file_path.clone()).expect("Could not open file: {file_path}");
        Box::new(BufReader::new(file))
    }
}

fn file_character_count(reader: &mut Box<dyn BufRead>) -> usize {
    use utf8_chars::BufReadCharsExt;
    reader.chars().count()
}

fn file_line_count(reader: &mut Box<dyn BufRead>) -> usize {
    let mut lines = 0;
    let mut buf = String::new();

    // TODO: optimize for memory. What if file contains single line gigabytes long?
    while reader.read_line(&mut buf).unwrap() != 0 {
        lines += 1;
        buf.clear();
    }
    lines
}

fn file_word_count(reader: &mut Box<dyn BufRead>) -> usize {
    let mut words = 0;
    let mut buf = String::new();

    // TODO: optimize for memory. What if file contains single line gigabytes long?
    while reader.read_line(&mut buf).unwrap() != 0 {
        words += buf.split_whitespace().count();
        buf.clear();
    }
    words
}

fn file_byte_count(reader: &mut Box<dyn BufRead>) -> usize {
    use std::io::Read;
    reader.bytes().count()
}
