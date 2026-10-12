use crate::value::IdentKind;

pub(crate) fn split_commas(input: &str) -> Result<Vec<&str>, String> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut in_string = false;
    let mut escaped = false;
    for (i, ch) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            ',' if !in_string => {
                parts.push(input[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    if in_string {
        return Err("unterminated string literal".to_string());
    }
    parts.push(input[start..].trim());
    if parts.iter().any(|p| p.is_empty()) {
        return Err("expected value after comma".to_string());
    }
    Ok(parts)
}

pub(crate) fn is_identifier_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_alphabetic())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

pub(crate) fn parse_identifier(input: &str) -> Result<(IdentKind, String, &str), String> {
    let bytes = input.as_bytes();
    if bytes.first() != Some(&b'$') {
        return Err("expected identifier".to_string());
    }
    let (kind, prefix_len) = if bytes.get(1) == Some(&b'$') {
        (IdentKind::Constant, 2)
    } else {
        (IdentKind::Variable, 1)
    };
    let Some(first) = bytes.get(prefix_len).copied() else {
        return Err("expected identifier after '$'".to_string());
    };
    if !first.is_ascii_alphabetic() {
        return Err("variable name/identifier must start with an ASCII letter".to_string());
    }
    let mut end = prefix_len + 1;
    while let Some(byte) = bytes.get(end).copied() {
        if byte.is_ascii_alphanumeric() || byte == b'_' {
            end += 1;
        } else {
            break;
        }
    }
    let raw_name = &input[prefix_len..end];
    let name = match kind {
        IdentKind::Variable => raw_name.to_ascii_lowercase(),
        IdentKind::Constant => {
            if raw_name != raw_name.to_ascii_uppercase() {
                return Err("constant identifier must be uppercase".to_string());
            }
            raw_name.to_string()
        }
    };
    Ok((kind, name, &input[end..]))
}

pub(crate) fn parse_var_ref(input: &str) -> Result<String, String> {
    let (kind, name, rest) = parse_identifier(input)?;
    if kind != IdentKind::Variable {
        return Err("expected variable identifier".to_string());
    }
    if !rest.trim().is_empty() {
        return Err("unexpected text after identifier".to_string());
    }
    Ok(name)
}

pub(crate) fn split_string_literal_tail(input: &str) -> Result<(&str, &str), String> {
    let input = input.trim();
    if !input.starts_with('"') {
        return Err("expected string literal".to_string());
    }
    let mut escaped = false;
    for (offset, ch) in input[1..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => {
                let end = 1 + offset + ch.len_utf8();
                return Ok((&input[..end], &input[end..]));
            }
            _ => {}
        }
    }
    Err("unterminated string literal".to_string())
}

pub(crate) fn parse_string_literal(input: &str) -> Result<String, String> {
    let input = input.trim();
    let (literal, tail) = split_string_literal_tail(input)?;
    if !tail.trim().is_empty() {
        return Err("unexpected text after string literal".to_string());
    }
    let mut chars = literal[1..literal.len() - 1].chars();
    let mut result = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some(other) => return Err(format!("unsupported escape \\{other}")),
                None => return Err("unterminated escape".to_string()),
            },
            other => result.push(other),
        }
    }
    Ok(result)
}

pub(crate) fn is_number_literal(input: &str) -> bool {
    let input = input.trim();
    if input.is_empty() {
        return false;
    }
    let digits = input.strip_prefix('-').unwrap_or(input);
    let mut saw_digit = false;
    let mut saw_dot = false;
    for ch in digits.chars() {
        if ch.is_ascii_digit() {
            saw_digit = true;
        } else if ch == '.' && !saw_dot {
            saw_dot = true;
        } else {
            return false;
        }
    }
    saw_digit
}
