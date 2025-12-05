// Copyright (c) 2020 R James Cortese
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Read and parse MySQL's
//! [.mylogin.cnf](https://dev.mysql.com/doc/refman/5.7/en/mysql-config-editor.html)
//! file.
//!
//! # Installation
//! Add `myloginrs` to `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! myloginrs = "0.2"
//! ```
//!
//!
//! # Examples
//!
//! To get a HashMap of login info for `"client"` just use the parse function:
//!
//! ```rust
//! # use std::path::PathBuf;
//! let file_path = PathBuf::from(
//!     "tests/test_mylogin.cnf",
//! );
//!
//! let client_info = myloginrs::parse("client", Some(&file_path));
//! ```
//!
//! Then you can use that HashMap with an `OptsBuilder` or other structs
//! from the [mysql](https://crates.io/crates/mysql):
//!
//! ```rust
//! # use std::path::PathBuf;
//! # use std::error::Error;
//! # use mysql::{OptsBuilder, Conn};
//! # fn main() -> Result<(), Box<dyn Error>> {
//! #    let file_path = PathBuf::from(
//! #        "tests/test_mylogin.cnf",
//! #    );
//! #    let client_info = myloginrs::parse("client", Some(&file_path));
//! let opts = OptsBuilder::new()
//!     .ip_or_hostname(Some(&client_info["host"]))
//!     .tcp_port(u16::from_str_radix(&client_info["port"], 10)?)
//!     .user(Some(&client_info["user"]))
//!     .pass(Some(&client_info["password"]));
//!
//! let _conn = Conn::new(opts);
//! #    Ok(())
//! # }
//! ```
//!
//! Starting with [mysql 20.1.0](https://crates.io/crates/mysql/20.1.0),
//! you can do the even simpler:
//!
//! ```rust
//! # use std::path::PathBuf;
//! # use std::error::Error;
//! # use mysql::{OptsBuilder, Conn};
//! # fn main() -> Result<(), Box<dyn Error>> {
//! #    let file_path = PathBuf::from(
//! #        "tests/test_mylogin.cnf",
//! #    );
//! #    let client_info = myloginrs::parse("client", Some(&file_path));
//! let opts = OptsBuilder::new().from_hash_map(&client_info).unwrap();
//! let _conn = Conn::new(opts);
//! #    Ok(())
//! # }
//! ```
//!
//! If you would rather get a String that contains the whole file,
//! use read:
//!
//! ```no_run
//! let mylogin_plaintext = myloginrs::read(None);
//!
//! println!("{}", mylogin_plaintext);
//! ```
//!
//! This second example passes `None` as the path to use the
//! default .mylogin.cnf location (`%APPDATA%\MySQL\.mylogin.cnf` on windows or
//! `~/.mylogin.cnf` on everything else).

extern crate ini;
extern crate openssl;
extern crate shellexpand;

use ini::Ini;
use openssl::symm::{decrypt, Cipher};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// .mylogin.cnf is AES 128-bit ECB encrypted

// Unused buffer at the beginning of the login path file.
const UNUSED_BUFFER_LENGTH: usize = 4;
// The length of the key stored in the file.
const LOGIN_KEY_LENGTH: usize = 20;
// Number of bytes used to store the length of ciphertext.
const CIPHER_STORE_LENGTH: usize = 4;
// Number of bytes in one AES block
const AES_BLOCK_SIZE: usize = 16;

/// Read the file at path and decrypt it.
/// Return the contents as a String.
pub fn read(path: Option<&Path>) -> String {
    let path = match path {
        Some(p) => PathBuf::from(p),
        None => get_login_path_file(),
    };
    // todo!("could check some file size stuff here...");
    let mut encrypted: Vec<u8> = fs::read(path).expect("Failed to read path");

    let plain_bytes = read_encrypted_file(&mut encrypted);

    String::from_utf8(plain_bytes).expect("Failed to convert to to utf8")
}

/// Parse the file at path, decrypting it and
/// return a HashMap containing the key value pairs
/// for the login_path heading.
pub fn parse(login_path: &str, path: Option<&Path>) -> HashMap<String, String> {
    let decrypt_file = read(path);
    let conf = Ini::load_from_str(&decrypt_file).unwrap();
    let section = conf.section(Some(login_path)).unwrap();

    let mut map = HashMap::new();

    for (k, v) in section.iter() {
        map.insert(k.to_string(), v.to_string());
    }
    map
}

fn get_login_path_file() -> PathBuf {
    match env::var("MYSQL_LOGIN_FILE") {
        Ok(mylogin_path) => PathBuf::from(mylogin_path),
        Err(_) => {
            let mylogin_path = get_default_path();
            PathBuf::from(mylogin_path)
        }
    }
}

fn get_default_path() -> String {
    if cfg!(windows) {
        let mylogin_root = env::var("APPDATA").expect("env var 'APPDATA' is not set");
        let mylogin_path = [&mylogin_root, "MySQL", ".mylogin.cnf"].concat();
        mylogin_path
    } else {
        String::from(shellexpand::tilde("~/.mylogin.cnf"))
    }
}

fn read_key(key_buffer: &mut [u8]) -> [u8; AES_BLOCK_SIZE] {
    let mut rkey: [u8; AES_BLOCK_SIZE] = [0; AES_BLOCK_SIZE];
    for i in 0..LOGIN_KEY_LENGTH {
        rkey[i % AES_BLOCK_SIZE] ^= key_buffer[i];
    }
    rkey
}

fn read_encrypted_file(encrypted: &mut Vec<u8>) -> Vec<u8> {
    let end_of_login_key = UNUSED_BUFFER_LENGTH + LOGIN_KEY_LENGTH;

    let key_slice = &mut encrypted[UNUSED_BUFFER_LENGTH..end_of_login_key];
    let key = read_key(key_slice);

    let mut ciphertext = &mut encrypted[end_of_login_key..];
    let cipher = Cipher::aes_128_ecb();

    let mut plaintext: Vec<u8> = Vec::new();

    while ciphertext.len() > CIPHER_STORE_LENGTH {
        let (chunk_len_slice, r) = ciphertext.split_at_mut(CIPHER_STORE_LENGTH);
        ciphertext = r;

        let mut chunk_len_array: [u8; 4] = [0; 4];
        chunk_len_array.clone_from_slice(chunk_len_slice);
        let chunk_len = u32::from_le_bytes(chunk_len_array);

        let (cipher_chunck, r) = ciphertext.split_at_mut(chunk_len as usize);
        ciphertext = r;
        let mut decrypted = decrypt(cipher, &key, None, &cipher_chunck).unwrap();

        plaintext.append(&mut decrypted);
    }
    plaintext
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    #[test]
    fn test_get_default_path_windows() {
        let string = get_default_path();

        let mylogin_root = env::var("APPDATA").expect("env var 'APPDATA' is not set");
        let mylogin_path = [&mylogin_root, "MySQL", ".mylogin.cnf"].concat();
        assert_eq!(mylogin_path, string);
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_get_default_path_not_windows() {
        let string = get_default_path();
        assert_eq!(String::from(shellexpand::tilde("~/.mylogin.cnf")), string);
    }

    #[test]
    fn test_get_login_path_file_from_env() {
        unsafe {
            env::set_var("MYSQL_LOGIN_FILE", "my_file_path");
        }
        let path = get_login_path_file();
        assert_eq!(PathBuf::from("my_file_path"), path);
    }
}
