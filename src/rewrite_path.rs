use std::ffi::OsString;
use std::path::PathBuf;

pub fn rewrite_path(file_path_into: impl Into<PathBuf>, file_name: impl Into<String>, file_extension: impl Into<String>) -> (PathBuf, PathBuf) {
    // &OsStrとかOsStringは初めて見ました.
    // でも見た感じとかメソッド探索で覚えれそうです

    // stemはファイルの名前全体の幹の部分(拡張子を除いた名前の部分)
    // extensionはその名の通り, 拡張子のことを表す
    let mut file_path = file_path_into.into();
    let file_stem = file_name.into();
    let file_extension = file_extension.into();

    let mut stem = OsString::new();
    let mut extension = OsString::new();

    stem.push(file_stem);
    extension.push(file_extension);

    let mut dir_path = file_path.clone();

    dir_path.set_file_name("");
    dir_path.set_extension("");

    file_path.set_file_name(stem);
    file_path.set_extension(extension);

    (file_path, dir_path)
}