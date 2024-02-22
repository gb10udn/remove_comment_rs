use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use std::io::prelude::*;
use std::fs::File;


pub fn open_file(path: &String) -> Option<String> {
    const UTF16_LE: [u8; 2] = [255, 254];
    if let Ok(mut file) = File::open(&path) {
        let mut buffer: [u8; 2] = [0; 2];
        file.read_exact(&mut buffer).unwrap();  // INFO: 240221 先頭の2バイトを読み取る

        match buffer {
            UTF16_LE => {
                let mut buffer: Vec<u8> = Vec::new();
                file.read_to_end(&mut buffer).expect("Failed to read file");  // INFO: 240222 これで、1 byte ずつ読み出せる。
                let utf16: Vec<u16> = from_u8_to_u16_le(&buffer);
                let result = decode_utf16_to_utf8(&utf16);
                Some(result)
            }
            _ => {
                // INFO: 240221 read as utf-8
                let mut result = String::new();
                match file.read_to_string(&mut result){
                    Ok(_) => {Some(result)}
                    _ => {None}
                }
            }
        }
    } else {
        None
    }
}

fn decode_utf16_to_utf8(source: &[u16]) -> String {  // TODO: 240222 単体テストコードを用意せよ。
    decode_utf16(source.iter().cloned())
        .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
        .collect()
}

fn from_u8_to_u16_le(bytes: &[u8]) -> Vec<u16> {  // TODO: 240222 単体テストコードを用意せよ。
    bytes
        .chunks_exact(2) // バイト列を2バイトごとに分割
        .map(|chunk| {
            let byte1 = chunk[0] as u16;
            let byte2 = chunk[1] as u16;
            (byte2 << 8) | byte1
        })
        .collect()
}