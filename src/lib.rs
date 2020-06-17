/* Ported to rust from 
 * github.com/PyMySQL/myloginpath and 
 * github.com/ocelot-inc/ocelotgui/blob/master/readmylogin.c
 *
 * Funtions that read and decrypt MySQL's login path file.
 */


extern crate shellexpand;
extern crate openssl;
extern crate ini;

use std::env;
use std::path::{ Path, PathBuf };
use std::fs;
use std::collections::HashMap;
use openssl::symm::{ decrypt, Cipher };
use ini::Ini;

// .mylogin.cnf is AES 128-bit ECB encrypted

// Unused buffer at the beginning of the login path file.
const UNUSED_BUFFER_LENGTH: usize = 4;
// The length of the key stored in the file.
const LOGIN_KEY_LENGTH: usize = 20;
// Number of bytes used to store the length of ciphertext.
const CIPHER_STORE_LENGTH: usize = 4;
// Number of bytes in one AES block
const AES_BLOCK_SIZE: usize = 16;



pub fn read(path: Option<&Path>) -> String {
    let path = match path {
        Some(p) => PathBuf::from(p),
        None => get_login_path_file()
    };
    // todo!("could check some file size stuff here...");
    let mut encrypted: Vec<u8> = fs::read(path).expect("Failed to read path");

    let plain_bytes = read_encrypted_file(&mut encrypted);
    
    String::from_utf8(plain_bytes).expect("Failed to convert to to utf8")
}


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
        let mut decrypted = decrypt(
            cipher,
            &key,
            None,
            &cipher_chunck).unwrap();

        plaintext.append(&mut decrypted);
    }
    plaintext
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_read() {
        let file_path = PathBuf::from("test/test_mylogin.cnf");
        let output_str = "[client]\n\
            user = localuser\n\
            password = abc123\n\
            host = localhost\n\
            port = 1234\n";
        
        assert_eq!(read(Some(&file_path)), String::from(output_str));
    }
    
    #[test]
    fn test_parse() {
        let login_path = "client";
        let file_path = PathBuf::from("test/test_mylogin.cnf");
        let output_map: HashMap<String, String> = vec![
            (String::from("user"), String::from("localuser")),
            (String::from("password"), String::from("abc123")),
            (String::from("host"), String::from("localhost")),
            (String::from("port"), String::from("1234"))
        ].into_iter().collect();

        assert_eq!(parse(login_path, Some(&file_path)), output_map);
    }
}
