use std::io::{Write, BufReader};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use chrono::Local;
use walkdir::WalkDir;
use clap::Parser;
mod rmc;
mod opf;
use serde::{Deserialize, Serialize};


fn main() -> Result<(), Box<dyn std::error::Error>> {  // TODO: 240228 result 返すのでなくて、結果を表示する方が、ユーザーにとって優しいかもしれない？
    let args = Args::parse();  // HACK: 240221 引数の渡し方は運用決めてから再度検討すること。
    let config = open_config("./config.json")?;

    let src: String;
    match args.src {
        Some(val) => {
            src = val;
        }
        None => {
            src = fs::canonicalize("./").unwrap().to_string_lossy().to_string();
        }
    }
    let src = remove_head_and_tail_double_quotation(&src);  // HACK: 240219 タイミングは要検討 (対話的にユーザー入力を取得しない限りは不要かも？)
    let src_ = Path::new(&src);

    let mut temp_dst = PathBuf::from(r".\dst_rmc");
    let now: String = Local::now()
        .format("%Y%m%d_%H%M%S")
        .to_string();
    temp_dst.push(&now);
    
    if src_.is_file() {  // HACK: 240313 単体ファイルでも、ベースディレクトリ指定でも、Vec 型に格納してから処理開始して、データフローを統一するとすっきり書けるかも？
        temp_dst.push(src_.file_name().unwrap());
        let dst = temp_dst.to_string_lossy().to_string();
        match try_to_remove_comment_and_save_one(&src, &dst, &config.remove_comments, &config.target_extensions, &config.remove_multiline_comment) {
            Ok(_) => {
                Ok(())
            },
            Err(err) => {
                Err(err.into())
            }
        }

    } else if src_.is_dir() {
        let folder_name = src_.file_name().unwrap();
        temp_dst.push(folder_name);
        let dst_base_dir = temp_dst.to_string_lossy().to_string();
        
        let path_vec = retrieve_path_vec(&src, &config.target_extensions);
        let mut error_messages: Vec<String> = vec![];
        for fpath in path_vec {  // TODO: 240228 最後に、ファイル何個が存在し、ターゲットのテキストファイルが何件で、処理したのが何件で、、、を表示するといいかも？
            let dst = fpath.replace(&src, &dst_base_dir);
            if let Err(err) = try_to_remove_comment_and_save_one(&fpath, &dst, &config.remove_comments, &config.target_extensions, &config.remove_multiline_comment) {
                error_messages.push(err.to_string());
            }
        }
        match error_messages.len() {
            0 => {
                Ok(())
            },
            _ => {
                Err(format!("{:?}", error_messages).into())
            },
        }
    } else {
        Err(format!("File Not Found -> {}", src).into())
    }

    // TODO: 240220 フルパスでどこのファイルを処理したかは、別途 log ファイルに残してあげるといいような気もする。(.exe ダブルクリックで実施するなら、必須かもしれない？)
}


// HACK: 240313 コンパクトにせよ。拡張子分岐のところが長いので、別関数にくくりだすとか？
fn try_to_remove_comment_and_save_one(src: &String, dst: &String, remove_comments: &Vec<String>, target_extensions: &Vec<String>, remove_multiline_comment: &bool) -> Result<(), Box<dyn std::error::Error>> {
    let src_ = Path::new(src);
    if let Some(ext) = src_.extension() {
        let ext = ext.to_str().unwrap();
        if target_extensions.contains(&ext.to_string()) {  // HACK: 240228 target ってのが、コメント削除のことか、コピー対象のことかが分かりにくい。target_text_file_extensions の方が長いけどわかりやすいかも？
            
            // [START] create dist basedir
            let dst = Path::new(dst);
            let base_path = dst
                .parent()
                .unwrap();
            fs::create_dir_all(base_path).unwrap();
            // [END] create dist basedir
            
            match ext {
                "xlsm" => {
                    let bas_file_vec = opf::xlsm::retrieve_bas_file_name_and_code(src);
                    for mut bas_file in bas_file_vec {
                        bas_file.remove_comment(&remove_comments);

                        // TODO: 240313 複数行コメント削除を実装して、ここに導入せよ。

                        let mut dst_bas = dst.parent().unwrap().to_path_buf();
                        dst_bas.push(dst.file_stem().unwrap().to_string_lossy().to_string());
                        bas_file.save(&dst_bas.to_string_lossy().to_string())?;  // INFO: 240313 rust -> python へのデータはファイル渡しとする。  // HACK: 240313 インメモリ sqlite でローカルサーバー立てて実行するとかっこいい気がする。
                    }

                    // FIXME: 240313 .replace(".xlsm", "") が少し強引と思うので、修正せよ。
                    opf::xlsm::update_vba_code_with_removed_comments(src, &dst.to_string_lossy().to_string().replace(".xlsm", ""), &String::from("./test.xlsm"));

                    Ok(())
                },
                _ => {
                    if let Ok(mut code) = opf::text::open_file(&src) {
                        match ext {
                            "py" => {
                                if *remove_multiline_comment {
                                    code = rmc::py::remove_multiline_comment(&code);
                                }
                                code = rmc::py::remove_comment(&code, &remove_comments);
                            }
                            "ps1" => {
                                if *remove_multiline_comment {
                                    code = rmc::ps::remove_multiline_comment(&code);
                                }
                                code = rmc::ps::remove_comment(&code, &remove_comments);
                            }
                            "psd1" => {
                                if *remove_multiline_comment {
                                    code = rmc::ps::remove_multiline_comment(&code);
                                }
                                code = rmc::ps::remove_comment(&code, &remove_comments);
                            }
                            "psm1" => {
                                if *remove_multiline_comment {
                                    code = rmc::ps::remove_multiline_comment(&code);
                                }
                                code = rmc::ps::remove_comment(&code, &remove_comments);
                            }
                            _ => {
                                // INFO: 240310 .json など、コピペするだけのファイル。  // FIXME: 240310 この場合、わざわざテキストファイルとして開く必要がない気がしてきた。エラーの温床だし。
                            }
                        }
                        
                        // [START] save text file
                        let mut file = File::create(dst)
                            .expect("file not found.");
                    
                        write!(file, "{}", code)
                            .expect("cannot write.");
                        // [END] save text file
        
                        Ok(())
        
                    } else {
                        Err(format!("Fail to open file -> {}", src).into())
                    }
                },
            }
        } else {
            Err(format!("").into())  // FIXME: 240228 ここの運用が少し微妙かも？taraget_ext でないファイルパスを渡すな、と。
        }
    } else {
        Err(format!("No Extension ? {}", src).into())  // INFO: 240228 拡張子を持たないテキストファイルは対象としない前提とした。
    }
}

/// base_dir 配下のファイルを再帰的に検索し、そのパスのベクタ型を返す関数。
fn retrieve_path_vec(base_dir: &String, target_extensions: &Vec<String>) -> Vec<String> {  // HACK: 240220 引数は、Path で与えてもいいのかも？
    let mut result: Vec<String> = vec![];
    for entry in WalkDir::new(base_dir) {
        if let Ok(val) = entry {
            if val.path().is_file() {
                if let Some(ext) = val.path().extension() {
                    let ext = ext.to_str().unwrap();
                    if target_extensions.contains(&ext.to_string()) {
                        let fpath = val.path().to_string_lossy().to_string();
                        result.push(fpath);
                    }
                }
            }
        }
    }
    result
}

fn remove_head_and_tail_double_quotation(arg: &String) -> String {
    let mut result = arg.clone();
    if result.ends_with("\n") == true {
        result.pop();  // INFO: 240113 標準入力で取得時の末尾の改行コードを除去するため。
    }
    if result.starts_with("\"") {
        result.remove(0);
    }
    if result.ends_with("\"") {
        result.pop();
    }
    result
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    remove_multiline_comment: bool,
    remove_comments: Vec<String>,
    target_extensions: Vec<String>,
}

fn open_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)?;
    Ok(config)
}

#[derive(Parser, Debug)]
#[command()]
struct Args {
    /// コメント削除する source のパス。ファイル or ディレクトリを指定する。
    #[arg(short = 's', long)]
    src: Option<String>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_remove_head_and_tail_double_quotation() { 
        use crate::remove_head_and_tail_double_quotation;

        assert_eq!(remove_head_and_tail_double_quotation(&String::from("abc\n")), String::from("abc"));
        assert_eq!(remove_head_and_tail_double_quotation(&String::from("\"abc\"\n")), String::from("abc"));
    }

    #[test]
    fn test_retrieve_path_vec() {
        use crate::retrieve_path_vec;

        let src = r".\misc";
        let target_extensions = vec![String::from("py")];
        assert_eq!(retrieve_path_vec(&src.to_string(), &target_extensions), vec![String::from(r".\misc\piyo\sample_002.py"), String::from(r".\misc\sample_001.py"),]);
        
        let target_extensions = vec![String::from("ps")];  // INFO: 240221 ps を指定すると、py ファイルは取得しない。
        assert_ne!(retrieve_path_vec(&src.to_string(), &target_extensions), vec![String::from(r".\misc\piyo\sample_002.py"), String::from(r".\misc\sample_001.py"),]);
    }
}