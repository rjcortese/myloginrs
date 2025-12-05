use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_read() {
    let file_path = PathBuf::from("tests/test_mylogin.cnf");
    let output_str = "[client]\n\
        user = localuser\n\
        password = abc123\n\
        host = localhost\n\
        port = 1234\n";

    assert_eq!(myloginrs::read(Some(&file_path)), String::from(output_str));
}

#[test]
fn test_parse() {
    let login_path = "client";
    let file_path = PathBuf::from("tests/test_mylogin.cnf");
    let output_map: HashMap<String, String> = vec![
        (String::from("user"), String::from("localuser")),
        (String::from("password"), String::from("abc123")),
        (String::from("host"), String::from("localhost")),
        (String::from("port"), String::from("1234")),
    ]
    .into_iter()
    .collect();

    assert_eq!(myloginrs::parse(login_path, Some(&file_path)), output_map);
}
