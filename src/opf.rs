use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use std::io::prelude::*;
use std::fs::{self, File};
use encoding_rs::*;


pub fn open_file(path: &String) -> Result<String, std::io::Error> {  // HACK: 240228 Box Error 型使って渡すのが良いのでは？
    const UTF16_LE: [u8; 2] = [255, 254];
    let mut file = File::open(&path)?;
    const READOUT_BYTE_NUM: usize = 2;
    let mut buffer: [u8; READOUT_BYTE_NUM] = [0; READOUT_BYTE_NUM];
    if let Ok(_) = file.read_exact(&mut buffer) {
        match buffer {
            UTF16_LE => {
                let mut buffer: Vec<u8> = Vec::new();
                file.read_to_end(&mut buffer)?;
                let utf16: Vec<u16> = from_u8_to_u16_le(&buffer);  // HACK: 240229 encoding_rs を導入することで対応すること。(不要関数も一緒に削除してしまうこと。)
                let result = decode_utf16_to_utf8(&utf16);
                Ok(result)
            }
            _ => {
                match read_as_utf8(&path) {
                    Ok(result) => {
                        Ok(result)
                    },
                    Err(_) => {
                        Ok(force_read_as_shift_jis(&path))
                    },
                }
            }
        }
    } else {
        // INFO: 240226 2 byte 以下の場合は、utf-8 で決め打ちする。
        read_as_utf8(&path)
    }
}

fn read_as_utf8(path: &String) -> Result<String, std::io::Error> {
    let mut file = File::open(&path)?;
    let mut result = String::new();
    file.read_to_string(&mut result)?;
    Ok(result)
}

fn force_read_as_shift_jis(path: &String) -> String {
    let s = fs::read(path).unwrap();  // FIXME: 240229 エラー処理を考えれていないので、修正せよ。
    let (res, _, _) = SHIFT_JIS.decode(&s);
    res.into_owned()
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

    #[test]
    fn test_encoding_rs_shift_jis() {
        use std::fs;
        use encoding_rs::*;

        let path = "./misc/sample_012.ps1";
        let s = fs::read(path).unwrap();
        let (res, _, _) = SHIFT_JIS.decode(&s);  // INFO: 240229 使用可能エンコーディング -> https://docs.rs/encoding_rs/0.8.33/encoding_rs/all.html
        let result = res.into_owned();
        let expected = String::from(r#"Set-Location ($PSScriptRoot)

Function test() {
    <#
    Shift-jis のつもりで書いたコードです。
    #>
    try {
        Write-Output 'This is test !!'
    } catch {
        Write-Host $_  # INFO: remove me
    }
}"#).replace("\n", "\r\n");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_force_read_as_shift_jis() {
        use crate::opf::force_read_as_shift_jis;

        let path = String::from("./misc/sample_012.ps1");
        let result = force_read_as_shift_jis(&path);
        let expected = String::from(r#"Set-Location ($PSScriptRoot)

Function test() {
    <#
    Shift-jis のつもりで書いたコードです。
    #>
    try {
        Write-Output 'This is test !!'
    } catch {
        Write-Host $_  # INFO: remove me
    }
}"#).replace("\n", "\r\n");  // EDIT: 240229 その後、rmc 実行すると、想定外のエラー起きたので、テストコード書いて修正せよ。

        assert_eq!(result, expected);
    }
}