use crate::rewrite_path;
use crate::RouteRule;

pub fn save_rules_to_file(save_dir: bool, rules: &[RouteRule]) -> std::io::Result<()> {

    let list_json = serde_json::to_vec_pretty(rules)?;

    let (rewrote_path_file, rewrote_path_dir) = rewrite_path("rule_list", "json")?;

    if save_dir == true {
        match std::fs::write(&rewrote_path_file, &list_json) {
            Ok(()) => {}
            Err(_) => {
                std::fs::create_dir_all(&rewrote_path_dir)?;
                std::fs::write(&rewrote_path_file, &list_json)?
            }
        }
    } else {
        std::fs::write("rule_list.json", list_json)?;
    }

    Ok(())
}