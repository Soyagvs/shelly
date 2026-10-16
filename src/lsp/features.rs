use super::syntax::{SymbolKind, parse};
use serde_json::{Value, json};

pub fn utf16_pos(text: &str, byte: usize) -> (usize, usize) {
    let mut line = 0;
    let mut start = 0;
    for (i, c) in text.char_indices() {
        if i >= byte {
            break;
        }
        if c == '\n' {
            line += 1;
            start = i + 1
        }
    }
    let part = &text[start..byte.min(text.len())];
    (line, part.encode_utf16().count())
}
pub fn range(text: &str, s: usize, e: usize) -> Value {
    let (l, c) = utf16_pos(text, s);
    let (el, ec) = utf16_pos(text, e);
    json!({"start":{"line":l,"character":c},"end":{"line":el,"character":ec}})
}
pub fn word_at(text: &str, byte: usize) -> Option<(String, usize, usize)> {
    let p = byte.min(text.len());
    let mut s = p;
    while s > 0 {
        let (i, c) = text[..s].char_indices().next_back().unwrap();
        if c.is_alphanumeric() || c == '_' {
            s = i
        } else {
            break;
        }
    }
    let mut e = p;
    while e < text.len() {
        let c = text[e..].chars().next().unwrap();
        if c.is_alphanumeric() || c == '_' {
            e += c.len_utf8()
        } else {
            break;
        }
    }
    if s < e {
        Some((text[s..e].into(), s, e))
    } else {
        None
    }
}
pub fn symbols(text: &str) -> Vec<(String, usize, usize, SymbolKind, bool)> {
    parse(text)
        .symbols
        .into_iter()
        .map(|s| (s.name, s.span.start, s.span.end, s.kind, s.declaration))
        .collect()
}
pub struct HelperMetadata {
    pub name: &'static str,
    pub signature: &'static str,
    pub contract: &'static str,
}

pub const HELPERS: &[HelperMetadata] = &[
    HelperMetadata {
        name: "substring",
        signature: "substring(text, start, length)",
        contract: "Uses one-based Unicode character positions and a length, not an end position.",
    },
    HelperMetadata {
        name: "padLeft",
        signature: "padLeft(text, length, fill)",
        contract: "Pads to length using exactly one Unicode character as fill.",
    },
    HelperMetadata {
        name: "padRight",
        signature: "padRight(text, length, fill)",
        contract: "Pads to length using exactly one Unicode character as fill.",
    },
    HelperMetadata {
        name: "formatDate",
        signature: "formatDate(iso, format)",
        contract: "Accepts strict UTC ISO input and only YYYY/MM/DD/HH/mm/ss tokens.",
    },
];

pub fn helper(name: &str) -> Option<&'static HelperMetadata> {
    HELPERS.iter().find(|h| h.name == name)
}

pub fn kind(k: SymbolKind) -> u8 {
    match k {
        SymbolKind::Function => 12,
        SymbolKind::Constant => 14,
        SymbolKind::Variable => 13,
        _ => 1,
    }
}
