use std::{env, fs::File, io::{self, BufRead, BufReader}, process::exit};

struct Stats {
    bytes: u64,
    words: u64,
    lines: u64,
    chars: u64,
}

fn print_usage_and_exit() {
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
    let mut commands: Vec<String> = Vec::new();

    for arg in env::args().skip(1) {
        if arg == "-c" || arg == "-l" || arg == "-w" || arg == "-m" {
            commands.push(arg);
        } else if arg.starts_with("-") {
            eprintln!("Unknown option: {arg}");
            print_usage_and_exit();
        } else {
            if file_path != "" {
                eprintln!("Multiple file paths is not supported (already provided path: {file_path}).");
                print_usage_and_exit();
            }
            file_path = arg;
        }
    }

    let stats = collect_stats(&mut create_reader(&file_path));
    let mut result: Vec<String> = Vec::new();

    if commands.contains(&"-l".to_string()) || commands.is_empty() {
        result.push(stats.lines.to_string());
    }

    if commands.contains(&"-w".to_string()) || commands.is_empty() {
        result.push(stats.words.to_string());
    }

    if commands.contains(&"-c".to_string()) || commands.is_empty() {
        result.push(stats.bytes.to_string());
    }

    if commands.contains(&"-m".to_string()) {
        result.push(stats.chars.to_string());
    }

    if file_path != "" {
        result.push(file_path);
    }

    println!("{}", result.join(" "));
}

fn collect_stats(reader: &mut Box<dyn BufRead>) -> Stats {
    let mut stats = Stats { bytes: 0, words: 0, lines: 0, chars: 0 };
    let mut previos_byte_is_whitespace = true;

    use utf8_chars::BufReadCharsExt;
    for byte_result in reader.chars() {
        if let Ok(char) = byte_result {
            let current_byte_is_whitespace = char.is_whitespace();
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
