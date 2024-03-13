use calamine::{Reader, open_workbook, Xlsx};
use std::fs::{self, File};
use std::path::PathBuf;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;
use crate::rmc::common::remove_comment;


#[derive(Debug, PartialEq)]
pub struct BasFile {
    file_name: String,
    code: String,
}

impl BasFile {
    /// self.code から、コメント削除する。
    pub fn remove_comment(&mut self, targets: &Vec<String>) {  // HACK: 240313 opf なのに、rmc の所掌のことを実行していたので、修正せよ。トレイト使えば、別の箇所で書ける？
        self.code = remove_comment(&self.code, targets, "'");
    }

    // TODO: 240213 複数行のコメントを削除する。例えば、コメントアウトのシングルクオーテーション (') までのスペースの数が同じだったら連続文字列とみなすとか？

    pub fn save(&self, dst_dir: &String) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(dst_dir)?;
        let mut dst = PathBuf::from(dst_dir);
        dst.push(&self.file_name);
        let mut file = File::create(dst)?;
        write!(file, "{}", self.code)?;
        Ok(())
    }

    // TODO: 240311 TEST_ から始まる関数を削除する機能を追加する。(= 単体テストコードを削除する。)
    // TODO: 240311 docstring に相当する。Function or Sub の直下のコメントアウトを削除する？
}

pub fn retrieve_bas_file_name_and_code(path: &String) -> Vec<BasFile> {
    let mut result: Vec<BasFile> = vec![];
    
    let mut workbook: Xlsx<_> = open_workbook(path).expect("Cannot open file");  // HACK: 240311 path が非存在のケースを考え、Result<T> で返すのが良い気もする。
    if let Some(Ok(vba)) = workbook.vba_project() {
        let module_names = vba.get_module_names();
        println!("{:?}", module_names);

        for module_name in module_names {
            let code = vba.get_module(module_name)
                .unwrap()
                .replace("\r\n", "\n");
            const NOT_BAS_WORD: &str = "Attribute VB_Base";
            if code.contains(NOT_BAS_WORD) == false {  // FIXME: 240310 Thisworkbook のコードにも適用できるようにする。
                let file_name = format!("{}.bas", module_name);
                let bas_file: BasFile = BasFile { file_name, code };
                result.push(bas_file);
            }
        }
    }
    result
}


pub fn update_vba_code_with_removed_comments(src_excel_with_macro: &String, bas_src_dir: &String, dst: &String) {  // HACK: 240313 いい関数名が思いつかなかったので、python 側と合わせて修正すること。
    let _ = Command::new("./vba.exe")
        .args([
            "--src",
            src_excel_with_macro as &str,
            "--bas-dir",
            bas_src_dir as &str,
            "--dst",
            dst as &str,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())  // FIXME: 240313 エラー処理を記述する。(現状では、何も知らせないため不親切な仕様になってしまっている。)
        .spawn()
        .unwrap();
}


#[cfg(test)]
mod tests {
    #[test]
    /// エクセルマクロのファイルを読み出せるかどうか。
    fn test_retrieve_bas_file_name_and_code() {
        use crate::opf::xlsm::{retrieve_bas_file_name_and_code, BasFile};

        let path = String::from("./misc/macro_sample_001.xlsm");
        let result = retrieve_bas_file_name_and_code(&path);
        assert_eq!(result.len(), 1);

        let expect = BasFile {
            file_name: String::from("Utils.bas"),
            code: String::from(r#"Attribute VB_Name = "Utils"
Option Explicit

Sub HellWorld()
    Msgbox "Hello World !!!"  ' FIXME: 240311 あああ
End Sub
"#),
        };
        assert_eq!(result[0], expect);
    }

    #[test]
    fn test_remove_comment() {
        use crate::opf::xlsm::{retrieve_bas_file_name_and_code, BasFile};

        let path = String::from("./misc/macro_sample_001.xlsm");
        let mut result_vec = retrieve_bas_file_name_and_code(&path);
        assert_eq!(result_vec.len(), 1);

        let result = &mut result_vec[0];
        result.remove_comment(&vec![String::from("FIXME:")]);

        let expect = BasFile {
            file_name: String::from("Utils.bas"),
            code: String::from(r#"Attribute VB_Name = "Utils"
Option Explicit

Sub HellWorld()
    Msgbox "Hello World !!!"
End Sub
"#),
        };
        assert_eq!(result, &expect);
    }
}