use crate::RouteRule;

pub fn save_rules_to_file(rules: &[RouteRule]) -> std::io::Result<()> {
    
    let list_json = serde_json::to_vec_pretty(rules)?;
    std::fs::write("server_list.json", list_json)?;

    Ok(())
}