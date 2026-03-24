pub fn format_hex(data: &[u8]) -> String {
    let converted_data = data.iter()
        .map(|d| format!("{:02X}", d))
        .collect::<Vec<_>>()
        .join(" ");

    converted_data
}
