use std::fs;
use std::path::Path;

const INDENT: &str = "    ";

/// Formats Shelly source without interpreting expressions or rewriting literals.
pub fn format_source(source: &str) -> Result<String, String> {
    let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
    let had_final = normalized.ends_with('\n');
    let mut depth = 0usize;
    let mut collection_depth = 0usize;
    let mut triple = false;
    let mut block_comment = false;
    let mut out = String::new();
    let line_count = normalized.split('\n').count();

    for (number, raw) in normalized.split('\n').enumerate() {
        if raw.is_empty() && number + 1 == line_count {
            continue;
        }
        if block_comment {
            out.push_str(raw.trim_end_matches([' ', '\t']));
            out.push('\n');
            if raw.trim_start().starts_with('#') {
                block_comment = false;
            }
            continue;
        }
        if triple {
            out.push_str(raw);
            out.push('\n');
            if raw.trim() == "```" {
                triple = false;
            }
            continue;
        }
        let (code, opens_triple) =
            scan_line(raw).map_err(|e| format!("line {}: {e}", number + 1))?;
        let trimmed = code.trim();
        if trimmed.starts_with('#') && !trimmed[1..].contains('#') {
            block_comment = true;
        }
        if opens_triple {
            triple = true;
        }
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }
        let (collection_opens, collection_closes) = collection_braces(trimmed);
        let collection_close = collection_depth > 0 && trimmed.starts_with('}');
        if collection_close {
            collection_depth = collection_depth
                .checked_sub(collection_closes)
                .ok_or_else(|| format!("line {}: unmatched collection terminator", number + 1))?;
        }
        let close = is_close(trimmed) && !collection_close;
        if close {
            depth = depth
                .checked_sub(1)
                .ok_or_else(|| format!("line {}: unmatched block terminator", number + 1))?;
        }
        out.push_str(&INDENT.repeat(depth + collection_depth));
        out.push_str(trimmed);
        if close {
            collection_depth += collection_opens;
        } else if !collection_close {
            collection_depth = collection_depth
                .checked_add(collection_opens)
                .and_then(|depth| depth.checked_sub(collection_closes))
                .ok_or_else(|| format!("line {}: unmatched collection terminator", number + 1))?;
        } else {
            collection_depth += collection_opens;
        }
        if is_open(trimmed)
            || trimmed.starts_with("else")
            || trimmed.starts_with("elif")
            || trimmed.starts_with("catch")
        {
            depth += 1;
        }
        out.push('\n');
    }
    if triple {
        return Err("unterminated multiline string".into());
    }
    if block_comment {
        return Err("unterminated comment".into());
    }
    if depth != 0 {
        return Err("unterminated block".into());
    }
    if !had_final && out.ends_with('\n') {
        out.pop();
    }
    if out.is_empty() || !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

fn scan_line(line: &str) -> Result<(String, bool), String> {
    let mut code = String::new();
    let mut quote = false;
    let mut escaped = false;
    let mut chars = line.char_indices().peekable();
    let mut opens_triple = false;
    while let Some((i, ch)) = chars.next() {
        if !quote && line[i..].starts_with("```") {
            code.push_str("```");
            opens_triple = true;
            break;
        }
        code.push(ch);
        if quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quote = false;
            }
        } else if ch == '"' {
            quote = true;
        } else if ch == '#' {
            break;
        }
    }
    if quote {
        return Err("unterminated string literal".into());
    }
    if opens_triple && !code[code.len() - 3..].is_empty() {
        // A delimiter is valid only at the end of its opening line.
        let rest = &line[code.len()..];
        if !rest.trim().is_empty() {
            return Err("unexpected text after multiline string delimiter".into());
        }
    }
    Ok((code, opens_triple))
}

fn collection_braces(line: &str) -> (usize, usize) {
    let mut opens = 0;
    let mut closes = 0;
    let mut quoted = false;
    let mut escaped = false;
    for ch in line.chars() {
        if quoted {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                quoted = false;
            }
        } else if ch == '"' {
            quoted = true;
        } else if ch == '{' {
            opens += 1;
        } else if ch == '}' {
            closes += 1;
        }
    }
    if (line.starts_with("fn ") || line.starts_with("export fn ")) && opens > 0 {
        opens -= 1;
    }
    (opens, closes)
}

fn first_word(line: &str) -> &str {
    line.split_whitespace().next().unwrap_or("")
}
fn is_open(line: &str) -> bool {
    line.starts_with("export fn ")
        || matches!(
            first_word(line),
            "if" | "for"
                | "while"
                | "repeat"
                | "function"
                | "fn"
                | "case"
                | "try"
                | "log"
                | "timeout"
                | "defer"
                | "watch"
        )
}
fn is_close(line: &str) -> bool {
    matches!(first_word(line), "end" | "}" | "else" | "elif" | "catch")
}

pub fn format_file(path: &Path, check: bool) -> Result<bool, String> {
    let source = fs::read_to_string(path)
        .map_err(|e| format!("could not read '{}': {e}", path.display()))?;
    let formatted = format_source(&source)?;
    if check {
        return Ok(source == formatted);
    }
    if source != formatted {
        fs::write(path, formatted)
            .map_err(|e| format!("could not write '{}': {e}", path.display()))?;
    }
    Ok(true)
}
