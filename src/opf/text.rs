use std::io::prelude::*;
use std::fs::{self, File};
use encoding_rs::*;


pub fn open_file(path: &String) -> Result<String, Box<dyn std::error::Error>> {
    const UTF16_LE: [u8; 2] = [255, 254];
    let mut file = File::open(&path)?;
    const READOUT_BYTE_NUM: usize = 2;
    let mut buffer: [u8; READOUT_BYTE_NUM] = [0; READOUT_BYTE_NUM];
    if let Ok(_) = file.read_exact(&mut buffer) {
        match buffer {
            UTF16_LE => {
                Ok(read_as_utf16le(&path)?)
            }
            _ => {
                match read_as_utf8(&path) {
                    Ok(result) => {
                        Ok(result)
                    },
                    Err(_) => {
                        Ok(force_read_as_shift_jis(&path)?)
                    },
                }
            }
        }
    } else {
        // INFO: 240226 2 byte 以下の場合は、utf-8 で決め打ちする。
        read_as_utf8(&path)
    }
}

fn read_as_utf8(path: &String) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(&path)?;
    let mut result = String::new();
    file.read_to_string(&mut result)?;
    Ok(result)
}

fn read_as_utf16le(path: &String) -> Result<String, Box<dyn std::error::Error>> {
    let s = fs::read(path)?;
    let (res, _, _) = UTF_16LE.decode(&s);
    Ok(res.into_owned().replace("\r\n", "\n"))
}

fn force_read_as_shift_jis(path: &String) -> Result<String, Box<dyn std::error::Error>> {
    let s = fs::read(path)?;
    let (res, _, _) = SHIFT_JIS.decode(&s);
    Ok(res.into_owned().replace("\r\n", "\n"))
}


#[cfg(test)]
mod tests {
    #[test]
    /// 読み出せるかどうか。
    fn test_open_file() {
        use crate::opf::text::open_file;

        let path = String::from("./misc/utf16le.json");
        let result = open_file(&path).unwrap();
        assert_eq!(result, String::from(r#"{"id":"ピヨピヨ", "pw":"piyopiyo"}"#));
    }

    #[test]
    /// エンコーディングが異なっても、同様に読み出せるか。
    fn test_open_file2() {
        use crate::opf::text::open_file;

        let path_shift_jis = String::from("./misc/sample_013_shift-jis.ps1");
        let result_shift_jis = open_file(&path_shift_jis).unwrap();

        let path_utf8 = String::from("./misc/sample_013_shift-jis.ps1");
        let result_utf8 = open_file(&path_utf8).unwrap();
        assert_eq!(result_shift_jis, result_utf8);
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
        use crate::opf::text::force_read_as_shift_jis;

        let path = String::from("./misc/sample_012.ps1");
        let result = force_read_as_shift_jis(&path).unwrap();
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
}"#);

        assert_eq!(result, expected);
    }
}