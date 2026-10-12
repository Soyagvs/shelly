#[derive(Clone, Debug)]
pub(crate) enum Value {
    Scalar(String),
    List(Vec<String>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum IdentKind {
    Variable,
    Constant,
}

pub(crate) fn display_identifier(kind: IdentKind, name: &str) -> String {
    match kind {
        IdentKind::Variable => format!("${name}"),
        IdentKind::Constant => format!("$${name}"),
    }
}

pub(crate) fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

pub(crate) fn format_list(values: &[String]) -> String {
    format!("[{}]", values.join(", "))
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
