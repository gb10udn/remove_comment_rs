use calamine::{Reader, open_workbook, Xlsx};


#[derive(Debug, PartialEq)]
struct BasFile {
    file_name: String,
    code: String,
}

pub fn extract_bas(path: &String) -> Vec<BasFile> {
    let mut result: Vec<BasFile> = vec![];
    
    let mut workbook: Xlsx<_> = open_workbook(path).expect("Cannot open file");
    if let Some(Ok(vba)) = workbook.vba_project() {
        let module_names = vba.get_module_names();
        println!("{:?}", module_names);

        for module_name in module_names {
            let code = vba.get_module(module_name)
                .unwrap()
                .replace("\r\n", "\n");
            const NOT_BAS_WORD: &str = "Attribute VB_Base";
            if code.contains(NOT_BAS_WORD) == false {  // FIXME: 240310 これで定義してしまうと、Thisworkbook にコードを書いてる場合に取り出せない。。。
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
    Msgbox "Hello World !!!"
End Sub
"#),
        };

        assert_eq!(result[0], expect);
    }
}