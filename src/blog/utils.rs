pub fn extract_header(text: &str) -> Option<&str> {
    // Find the position of '#'
    let start_pos = text.find('#')?;

    // Get the substring starting after '#'
    let after_hash = &text[start_pos + 1..];

    // Find the next newline character
    let end_pos = after_hash.find('\n').unwrap_or(after_hash.len());

    // Extract and trim the text between '#' and newline
    let header = &after_hash[..end_pos].trim();

    if header.is_empty() {
        None
    } else {
        Some(header)
    }
}
