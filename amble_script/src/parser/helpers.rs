use super::AstError;
use crate::MovabilityAst;

pub(super) fn unquote(s: &str) -> String {
    parse_string(s).unwrap_or_else(|_| s.to_string())
}

/// Parse a string literal (supports "...", r"...", and """..."""). Returns the decoded value.
pub(super) fn parse_string(s: &str) -> Result<String, AstError> {
    let (v, _u) = parse_string_at(s)?;
    Ok(v)
}

/// Parse a string literal starting at the beginning of `s`. Returns (decoded, bytes_consumed).
pub(super) fn parse_string_at(s: &str) -> Result<(String, usize), AstError> {
    let b = s.as_bytes();
    if b.is_empty() {
        return Err(AstError::Shape("empty string"));
    }
    // Triple-quoted
    if s.starts_with("\"\"\"") {
        let mut out = String::new();
        let mut i = 3usize; // after opening """
        let mut escape = false;
        while i < s.len() {
            if !escape && s[i..].starts_with("\"\"\"") {
                return Ok((out, i + 3));
            }
            let ch = s[i..].chars().next().unwrap();
            i += ch.len_utf8();
            if escape {
                match ch {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    other => {
                        out.push('\\');
                        out.push(other);
                    },
                }
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
            } else {
                out.push(ch);
            }
        }
        return Err(AstError::Shape("missing closing triple quote"));
    }
    // Raw r"..." and hashed raw r#"..."#
    if s.starts_with('r') {
        // Count hashes after r
        let mut i = 1usize;
        while i < s.len() && s.as_bytes()[i] as char == '#' {
            i += 1;
        }
        if i < s.len() && s.as_bytes()[i] as char == '"' {
            let num_hashes = i - 1;
            let close_seq = {
                let mut seq = String::from("\"");
                for _ in 0..num_hashes {
                    seq.push('#');
                }
                seq
            };
            let content_start = i + 1;
            let rest = &s[content_start..];
            if let Some(pos) = rest.find(&close_seq) {
                let val = &rest[..pos];
                return Ok((val.to_string(), content_start + pos + close_seq.len()));
            } else {
                return Err(AstError::Shape("missing closing raw quote"));
            }
        }
    }
    // Single-quoted
    if b[0] as char == '\'' {
        let mut out = String::new();
        let mut i = 1usize; // skip opening '
        let mut escape = false;
        while i < s.len() {
            let ch = s[i..].chars().next().unwrap();
            i += ch.len_utf8();
            if escape {
                match ch {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '\'' => out.push('\''),
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    other => {
                        out.push('\\');
                        out.push(other);
                    },
                }
                escape = false;
                continue;
            }
            match ch {
                '\\' => {
                    escape = true;
                },
                '\'' => return Ok((out, i)),
                _ => out.push(ch),
            }
        }
        return Err(AstError::Shape("missing closing single quote"));
    }
    // Regular quoted
    if b[0] as char != '"' {
        return Err(AstError::Shape("missing opening quote"));
    }
    let mut out = String::new();
    let mut i = 1usize; // skip opening quote
    let mut escape = false;
    while i < s.len() {
        let ch = s[i..].chars().next().unwrap();
        i += ch.len_utf8();
        if escape {
            match ch {
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                other => {
                    out.push('\\');
                    out.push(other);
                },
            }
            escape = false;
            continue;
        }
        match ch {
            '\\' => {
                escape = true;
            },
            '"' => return Ok((out, i)),
            _ => out.push(ch),
        }
    }
    Err(AstError::Shape("missing closing quote"))
}
pub(super) fn parse_movability_opt(raw: &str) -> Result<MovabilityAst, AstError> {
    let trimmed = raw.trim();
    if trimmed == "free" {
        return Ok(MovabilityAst::Free);
    }
    if let Some(rest) = trimmed.strip_prefix("fixed") {
        let rest = rest.trim_start();
        let (reason, _) =
            parse_string_at(rest).map_err(|_| AstError::Shape("movability fixed expects a quoted reason"))?;
        return Ok(MovabilityAst::Fixed { reason });
    }
    if let Some(rest) = trimmed.strip_prefix("restricted") {
        let rest = rest.trim_start();
        let (reason, _) =
            parse_string_at(rest).map_err(|_| AstError::Shape("movability restricted expects a quoted reason"))?;
        return Ok(MovabilityAst::Restricted { reason });
    }
    Err(AstError::Shape(
        "movability expects free | fixed \"reason\" | restricted \"reason\"",
    ))
}
pub(super) fn extract_body(src: &str) -> Result<&str, AstError> {
    let bytes = src.as_bytes();
    let mut depth = 0i32;
    let mut start = None;
    let mut end = None;
    let mut i = 0usize;
    let mut in_str = false;
    let mut escape = false;
    let mut in_comment = false;
    let mut at_line_start = true;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if in_comment {
            if c == '\n' {
                in_comment = false;
                at_line_start = true;
            }
            i += 1;
            continue;
        }
        if in_str {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_str = false;
            }

            // inside string, line starts don't apply
            i += 1;
            continue;
        }
        match c {
            '\n' => {
                at_line_start = true;
            },
            ' ' | '\t' | '\r' => {
                // keep at_line_start as-is
            },
            '"' => {
                in_str = true;
                at_line_start = false;
            },
            '#' => {
                // Treat '#' as a comment only if it begins the line (ignoring leading spaces)
                if at_line_start {
                    in_comment = true;
                }
                at_line_start = false;
            },
            '{' => {
                if depth == 0 {
                    start = Some(i + 1);
                }
                depth += 1;
                at_line_start = false;
            },
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(i);
                    break;
                }
                at_line_start = false;
            },
            _ => {
                at_line_start = false;
            },
        }
        i += 1;
    }
    let s = start.ok_or(AstError::Shape("missing '{' body start"))?;
    let e = end.ok_or(AstError::Shape("missing '}' body end"))?;
    Ok(&src[s..e])
}
pub(super) fn split_top_level_commas(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b as char {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                // Only split at commas that separate conditions, not those inside
                // lists like "ambient ... in rooms a,b,c".
                let mut j = i + 1;
                while j < bytes.len() && (bytes[j] as char).is_whitespace() {
                    j += 1;
                }
                // capture next word
                let mut k = j;
                while k < bytes.len() {
                    let ch = bytes[k] as char;
                    if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                        k += 1;
                    } else {
                        break;
                    }
                }
                let next_word = if j < k {
                    s[j..k].to_ascii_lowercase()
                } else {
                    String::new()
                };
                let is_sep = next_word.is_empty()
                    || matches!(
                        next_word.as_str(),
                        "has"
                            | "missing"
                            | "player"
                            | "container"
                            | "ambient"
                            | "in"
                            | "npc"
                            | "with"
                            | "flag"
                            | "chance"
                            | "all"
                            | "any"
                    );
                if is_sep {
                    let part = s[start..i].trim();
                    if !part.is_empty() {
                        out.push(part);
                    }
                    start = i + 1;
                }
            },
            _ => {},
        }
    }
    if start < s.len() {
        let tail = s[start..].trim();
        if !tail.is_empty() {
            out.push(tail);
        }
    }
    out
}
pub(super) fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | ':' | '_' | '#')
}
#[allow(dead_code)]
pub(super) fn strip_leading_ws_and_comments(s: &str) -> &str {
    let mut i = 0usize;
    let bytes = s.as_bytes();
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if bytes[i] as char == '#' {
            while i < bytes.len() && (bytes[i] as char) != '\n' {
                i += 1;
            }
            continue;
        }
        break;
    }
    &s[i..]
}
pub(super) struct SourceMap {
    line_starts: Vec<usize>,
    src: String,
}
impl SourceMap {
    pub(super) fn new(source: &str) -> Self {
        let mut starts = vec![0usize];
        for (i, ch) in source.char_indices() {
            if ch == '\n' {
                starts.push(i + 1);
            }
        }
        Self {
            line_starts: starts,
            src: source.to_string(),
        }
    }
    pub(super) fn line_col(&self, offset: usize) -> (usize, usize) {
        let idx = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let line_start = *self.line_starts.get(idx).unwrap_or(&0);
        let line_no = idx + 1;
        let col = offset.saturating_sub(line_start) + 1;
        (line_no, col)
    }
    pub(super) fn line_snippet(&self, line_no: usize) -> String {
        let start = *self.line_starts.get(line_no - 1).unwrap_or(&0);
        let end = *self.line_starts.get(line_no).unwrap_or(&self.src.len());
        self.src[start..end].trim_end_matches(['\r', '\n']).to_string()
    }
}
pub(super) fn str_offset(full: &str, slice: &str) -> usize {
    (slice.as_ptr() as usize) - (full.as_ptr() as usize)
}
pub(super) fn extract_note(header: &str) -> Option<String> {
    if let Some(idx) = header.find("note ") {
        let after = &header[idx + 5..];
        let trimmed = after.trim_start();
        if let Ok((n, _used)) = parse_string_at(trimmed) {
            return Some(n);
        }
    }
    None
}
