use std::{fs::File, io::Read};

use assert_cmd::Command;

#[test]
fn count_file_length() {
    let mut cmd = Command::cargo_bin("wc").unwrap();
    cmd
        .arg("-c")
        .arg("resources/test.txt")
        .assert()
        .stdout("342190 resources/test.txt\n");
}

#[test]
fn count_file_lines() {
    let mut cmd = Command::cargo_bin("wc").unwrap();
    cmd
        .arg("-l")
        .arg("resources/test.txt")
        .assert()
        .stdout("7145 resources/test.txt\n");
}

#[test]
fn count_file_words() {
    let mut cmd = Command::cargo_bin("wc").unwrap();
    cmd
        .arg("-w")
        .arg("resources/test.txt")
        .assert()
        .stdout("58164 resources/test.txt\n");
}

#[test]
fn count_file_chars() {
    let mut cmd = Command::cargo_bin("wc").unwrap();
    cmd
        .arg("-m")
        .arg("resources/test.txt")
        .assert()
        .stdout("339292 resources/test.txt\n");
}

#[test]
fn count_file_stats_without_arguments() {
    let mut cmd = Command::cargo_bin("wc").unwrap();
    cmd
        .arg("resources/test.txt")
        .assert()
        .stdout("7145 58164 342190 resources/test.txt\n");
}

#[test]
fn count_input_lines() {
    let mut cmd = Command::cargo_bin("wc").unwrap();
    let mut input = String::new();

    let path = "resources/test.txt";
    let mut file = File::open(path).expect("Could not read file: {path}");

    file.read_to_string(&mut input).expect("Could not read to string");

    cmd
        .arg("-l")
        .write_stdin(input)
        .assert()
        .stdout("7145 \n");
}

#[test]
fn count_input_stats_without_arguments() {
    let mut cmd = Command::cargo_bin("wc").unwrap();
    let mut input = String::new();

    let path = "resources/test.txt";
    let mut file = File::open(path).expect("Could not read file: {path}");

    file.read_to_string(&mut input).expect("Could not read to string");

    cmd
        .write_stdin(input)
        .assert()
        .stdout("7145 58164 342190 \n");
}