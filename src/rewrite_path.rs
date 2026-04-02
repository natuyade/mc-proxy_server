use std::ffi::OsString;
use std::path::PathBuf;

pub fn rewrite_path(file_name: impl Into<String>, file_extension: impl Into<String>) -> std::io::Result<(PathBuf, PathBuf)> {
    // &OsStrとかOsStringは初めて見ました.
    // でも見た感じとかメソッド探索で覚えれそうです
    
    let file_path = directories::ProjectDirs::from(
        "jp",
        "natuyade",
        "mc-proxy"
    );

    let path = match file_path {
        Some(p) => p,
        None => return Err(std::io::Error::new(std::io::ErrorKind::NotSeekable, "No valid home directory path could be retrieved from the operating system."))
    };

    let local_appdata_path = directories::ProjectDirs::config_local_dir(&path);

    // stemはファイルの名前全体の幹の部分(拡張子を除いた名前の部分)
    // extensionはその名の通り, 拡張子のことを表す
    let mut file_path = local_appdata_path.to_path_buf();
    let file_stem = file_name.into();
    let file_extension = file_extension.into();

    let mut stem = OsString::new();
    let mut extension = OsString::new();

    stem.push(file_stem);
    extension.push(file_extension);

    // pathを先に完成させることで
    // file_nameでdir入力に対応させました
    file_path.set_file_name(stem);
    file_path.set_extension(extension);

    let mut dir_path = file_path.clone();

    dir_path.set_file_name("");
    dir_path.set_extension("");

    Ok((file_path, dir_path))
}