use calamine::{Reader, open_workbook, Xlsx};
use crate::rmc::common::remove_comment;


#[derive(Debug, PartialEq)]
pub struct BasFile {
    file_name: String,
    code: String,
}

impl BasFile {
    /// self.code から、コメント削除する。
    pub fn remove_comment(&mut self, targets: &Vec<String>) {
        self.code = remove_comment(&self.code, targets, "'");
    }

    // TODO: 240311 TEST_ から始まる関数を削除する機能を追加する。(= 単体テストコードを削除する。)
    // TODO: 240311 docstring に相当する。Function or Sub の直下のコメントアウトを削除する？
}

pub fn extract_bas(path: &String) -> Vec<BasFile> {
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


#[cfg(test)]
mod tests {
    #[test]
    /// エクセルマクロのファイルを読み出せるかどうか。
    fn test_extract_bas() {
        use crate::opf::xlsm::{extract_bas, BasFile};

        let path = String::from("./misc/macro_sample_001.xlsm");
        let result = extract_bas(&path);
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
        use crate::opf::xlsm::{extract_bas, BasFile};

        let path = String::from("./misc/macro_sample_001.xlsm");
        let mut result_vec = extract_bas(&path);
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