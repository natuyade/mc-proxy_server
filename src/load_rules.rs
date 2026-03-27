use std::sync::{Arc, RwLock};
use crate::rewrite_path::rewrite_path;
use crate::{RouteRule, SharedRules};

pub fn load_rules_from_file() -> std::io::Result<SharedRules> {


    let (rewrote_path_file, _) = rewrite_path("rule_list", "json")?;
    
    // localappdataにファイルがあれば読む
    // 読めなければ下へ
    let loaded_rules = match std::fs::read(rewrote_path_file) {
        Ok(file) => {
            let rules_vec = serde_json::from_slice::<Vec<RouteRule>>(&file)?;
            let rule_box: SharedRules = Arc::new(RwLock::new(rules_vec));
            rule_box
        }
        Err(_) => {
            // localappdataにファイルがなければ相対パスにあるかみる
            // 読めなければ下へ
            match std::fs::read("rule_list.json") {
                Ok(file) => {
                    let rules_vec = serde_json::from_slice::<Vec<RouteRule>>(&file)?;
                    let rule_box: SharedRules = Arc::new(RwLock::new(rules_vec));
                    rule_box
                }
                // どっちもなかったらmain側で初期設定(default)を返すためにエラーを出す
                Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not found the rules.json"))
            }
        }
    };


    Ok(loaded_rules)
}