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
fn count_file_stats_wihtout_arguments() {
    let mut cmd = Command::cargo_bin("wc").unwrap();
    cmd
        .arg("resources/test.txt")
        .assert()
        .stdout("7145 58164 342190 resources/test.txt\n");
}