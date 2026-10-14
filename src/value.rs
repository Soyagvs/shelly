use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) enum Value {
    Scalar(String),
    List(Vec<Value>),
    Dict(BTreeMap<String, Value>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum IdentKind {
    Variable,
    Constant,
}

pub(crate) fn display_identifier(kind: IdentKind, name: &str) -> String {
    match kind {
        IdentKind::Variable => format!("${name}"),
        IdentKind::Constant => format!("${name}"),
    }
}

pub(crate) fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

pub(crate) fn format_value(value: &Value) -> String {
    match value {
        Value::Scalar(value) => value.clone(),
        Value::List(values) => format_list(values),
        Value::Dict(values) => format_dict(values),
    }
}

pub(crate) fn format_list(values: &[Value]) -> String {
    let parts: Vec<String> = values.iter().map(format_value).collect();
    format!("[{}]", parts.join(", "))
}

pub(crate) fn format_dict(values: &BTreeMap<String, Value>) -> String {
    let parts: Vec<String> = values
        .iter()
        .map(|(key, value)| format!("{}: {}", format_dict_key(key), format_value(value)))
        .collect();
    format!("{{{}}}", parts.join(", "))
}

fn format_dict_key(key: &str) -> String {
    if is_simple_key(key) {
        key.to_string()
    } else {
        format!("\"{}\"", key.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

pub(crate) fn is_simple_key(key: &str) -> bool {
    let mut chars = key.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_alphabetic())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

pub(crate) fn compare_values(left: &str, right: &str) -> std::cmp::Ordering {
    match (left.parse::<i64>(), right.parse::<i64>()) {
        (Ok(left), Ok(right)) => left.cmp(&right),
        _ => left.cmp(right),
    }
}

pub(crate) fn is_truthy(value: &str) -> bool {
    !(value == "false" || value.is_empty() || value == "0")
}
