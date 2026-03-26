use std::ffi::OsString;
use std::path::PathBuf;

pub fn rewrite_path(file_path_into: impl Into<PathBuf>, file_name: impl Into<String>, file_extension: impl Into<String>) -> PathBuf {
    // &OsStrとかOsStringは初めて見ました.
    // でも見た感じとかメソッド探索で覚えれそうです

    // stemはファイルの名前全体の幹の部分(拡張子を除いた名前の部分)
    // extensionはその名の通り, 拡張子のことを表す
    let file_path = file_path_into.into();
    let new_name = file_name.into();
    let new_extension = file_extension.into();

    let mut stem = match file_path.file_stem() {
        Some(stem) => stem.to_os_string(),
        None => OsString::new(),
    };
    let mut extension = match file_path.extension() {
        Some(extension) => extension.to_os_string(),
        None => OsString::new(),
    };

    // clearで中身を空のOsStringに
    stem.clear();
    // pushで文字を追加
    stem.push(new_name);

    // 拡張子
    extension.clear();
    extension.push(new_extension);

    let mut new_file_path = file_path.clone();
    new_file_path.set_file_name(stem);
    new_file_path.set_extension(extension);

    new_file_path
}