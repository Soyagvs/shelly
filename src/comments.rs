pub(crate) fn strip_comments(source: &str) -> Result<Vec<(usize, String)>, String> {
    let mut lines = Vec::new();
    let mut comment_start: Option<usize> = None;

    for (idx, raw_line) in source.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw_line.trim();

        if comment_start.is_some() {
            if find_unquoted_hash(trimmed, 0).is_some() {
                comment_start = None;
            }
            continue;
        }

        if trimmed.starts_with('#') {
            if find_unquoted_hash(trimmed, 1).is_none() {
                comment_start = Some(line_no);
            }
            continue;
        }

        lines.push((line_no, trimmed.to_string()));
    }

    if let Some(line_no) = comment_start {
        return Err(format!("line {line_no}: unterminated comment"));
    }

    Ok(lines)
}

fn find_unquoted_hash(input: &str, start: usize) -> Option<usize> {
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in input.char_indices() {
        if idx < start {
            continue;
        }
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '#' if !in_string => return Some(idx),
            _ => {}
        }
    }
    None
}
