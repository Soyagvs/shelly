use std::env;
use std::sync::{Mutex, OnceLock};

fn values() -> &'static Mutex<Vec<String>> {
    static VALUES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    VALUES.get_or_init(|| Mutex::new(Vec::new()))
}

pub(crate) fn get(name: &str) -> Result<String, String> {
    if !is_environment_name(name) {
        return Err("invalid environment variable name".to_string());
    }
    match env::var(name) {
        Ok(value) if !value.is_empty() => {
            values().lock().unwrap().push(value.clone());
            Ok(value)
        }
        Ok(_) | Err(env::VarError::NotPresent) => {
            Err("environment secret is missing or empty".to_string())
        }
        Err(env::VarError::NotUnicode(_)) => {
            Err("environment secret is not valid text".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_environment_name;

    #[test]
    fn accepts_conservative_environment_names() {
        assert!(is_environment_name("API_KEY_2"));
        assert!(is_environment_name("_TOKEN"));
        assert!(!is_environment_name("2FAST"));
        assert!(!is_environment_name("API-KEY"));
        assert!(!is_environment_name(""));
    }
}

pub(crate) fn redact(text: &str) -> String {
    values()
        .lock()
        .unwrap()
        .iter()
        .fold(text.to_string(), |text, secret| {
            text.replace(secret, "[REDACTED]")
        })
}

pub(crate) fn is_environment_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
