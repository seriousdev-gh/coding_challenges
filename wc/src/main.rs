use std::{env, fs::File, io::{self, BufRead, BufReader}, process::exit};

struct Stats {
    bytes: u64,
    words: u64,
    lines: u64,
    chars: u64,
}

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
            let stats = collect_stats(&mut create_reader(&file_path));
            println!("{bytes} {file_path}", bytes = stats.bytes);
        }
        "-l" => {
            let stats = collect_stats(&mut create_reader(&file_path));
            println!("{lines} {file_path}", lines = stats.lines);
        }
        "-w" => {
            let stats = collect_stats(&mut create_reader(&file_path));
            println!("{words} {file_path}", words = stats.words);
        }
        "-m" => {
            let stats = collect_stats(&mut create_reader(&file_path));
            println!("{chars} {file_path}", chars = stats.chars);
        }
        "" => {
            let stats = collect_stats(&mut create_reader(&file_path));
            println!("{lines} {words} {bytes} {file_path}", lines = stats.lines, words = stats.words, bytes = stats.bytes);
        }
        _ => {
            eprintln!("Unsupported arguments: {command} {file_path}");
            print_usage();
        }
    }
}

fn collect_stats(reader: &mut Box<dyn BufRead>) -> Stats {
    let mut stats = Stats { bytes: 0, words: 0, lines: 0, chars: 0 };
    let mut previos_byte_is_whitespace = true;

    use utf8_chars::BufReadCharsExt;
    for byte_result in reader.chars() {
        if let Ok(char) = byte_result {
            let current_byte_is_whitespace = char.is_whitespace() && char != '\u{2028}';
            if !previos_byte_is_whitespace && current_byte_is_whitespace {
                stats.words += 1;
            }

            if char == '\n' {
                stats.lines += 1;
            }

            stats.chars += 1;
            stats.bytes += char.len_utf8() as u64;

            previos_byte_is_whitespace = current_byte_is_whitespace;
        } else {
            eprintln!("Invalid char: {:?}", byte_result);
        }
    }

    if !previos_byte_is_whitespace {
        stats.words += 1;
    }

    stats
}

fn create_reader(file_path: &String) -> Box<dyn BufRead> {
    if *file_path == "" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        let file = File::open(file_path.clone()).expect("Could not open file: {file_path}");
        Box::new(BufReader::new(file))
    }
}
