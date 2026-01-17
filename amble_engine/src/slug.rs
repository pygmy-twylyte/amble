/// Normalize a user-provided identifier into a filesystem-safe slug.
pub fn sanitize_slug(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "world".to_string();
    }

    let mut slug = String::new();
    let mut pending_dash = false;
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            slug.push(ch.to_ascii_lowercase());
            pending_dash = false;
        } else if ch == '-' || ch == '_' {
            if !slug.is_empty() {
                slug.push(ch);
            }
            pending_dash = false;
        } else if ch.is_ascii_whitespace() {
            pending_dash = true;
        } else {
            pending_dash = true;
        }
    }

    let trimmed = slug.trim_matches(&['-', '_'][..]).to_string();
    if trimmed.is_empty() {
        "world".to_string()
    } else {
        trimmed
    }
}
