use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use std::io::prelude::*;
use std::fs::File;


pub fn open_file(path: &String) -> Option<String> {
    const UTF16_LE: [u8; 2] = [255, 254];
    if let Ok(mut file) = File::open(&path) {
        const READOUT_BYTE_NUM: usize = 2;
        let mut buffer: [u8; READOUT_BYTE_NUM] = [0; READOUT_BYTE_NUM];
        if let Ok(_) = file.read_exact(&mut buffer) {
            match buffer {
                UTF16_LE => {
                    let mut buffer: Vec<u8> = Vec::new();
                    match file.read_to_end(&mut buffer) {
                        Ok(_) => {
                            let utf16: Vec<u16> = from_u8_to_u16_le(&buffer);
                            let result = decode_utf16_to_utf8(&utf16);
                            Some(result)
                        }
                        Err(_) => {None}
                    }
                }
                _ => {
                    // INFO: 240221 read as utf-8
                    read_as_utf8(&path)
                }
            }
        } else {
            // INFO: 240226 2 byte 以下の場合は、utf-8 で決め打ちする。
            read_as_utf8(&path)
        }
    } else {
        None
    }
}

fn read_as_utf8(path: &String) -> Option<String> {
    let mut file = File::open(&path).unwrap();
    let mut result = String::new();
    match file.read_to_string(&mut result) {
        Ok(_) => {Some(result)}
        Err(_) => {None}
    }
}

fn decode_utf16_to_utf8(source: &[u16]) -> String {
    decode_utf16(source.iter().cloned())
        .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
        .collect()
}

fn from_u8_to_u16_le(bytes: &[u8]) -> Vec<u16> {
    bytes
        .chunks_exact(2) // INFO: 240224 バイト列を 2 バイトごとに分割
        .map(|chunk| {
            let byte1 = chunk[0] as u16;
            let byte2 = chunk[1] as u16;
            (byte2 << 8) | byte1
        })
        .collect()
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_from_u8_to_u16_le() { 
        use crate::opf::from_u8_to_u16_le;
        
        let src: [u8; 6] = [1, 0, 3, 0, 4, 0];
        let expected_result: [u16; 3] = [1, 3, 4];
        let result = from_u8_to_u16_le(&src);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_decode_utf16_to_utf8() {
        use crate ::opf::decode_utf16_to_utf8;

        let src: [u16; 1] = [0x3001];
        assert_eq!(decode_utf16_to_utf8(&src), String::from("、"))
    }

    #[test]
    fn test_open_file() {
        use crate::opf::open_file;

        let path = String::from("./misc/utf16le.json");
        let result = open_file(&path).unwrap();
        assert_eq!(result, String::from(r#"{"id":"ピヨピヨ", "pw":"piyopiyo"}"#));
    }
}