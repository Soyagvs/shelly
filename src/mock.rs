use std::collections::HashMap;

#[derive(Clone, Debug)]
pub(crate) enum Mock {
    Read {
        content: String,
    },
    Http {
        method: String,
        url: String,
        status: u16,
        body: String,
    },
    Command {
        program: String,
        args: Vec<String>,
        code: i32,
        stdout: String,
        stderr: String,
    },
}

#[derive(Default)]
pub(crate) struct MockRegistry {
    entries: HashMap<String, Mock>,
}
impl MockRegistry {
    pub(crate) fn add(&mut self, key: String, mock: Mock) -> Result<(), String> {
        if self.entries.insert(key, mock).is_some() {
            return Err("duplicate mock declaration".to_string());
        }
        Ok(())
    }
    pub(crate) fn find(&self, key: &str) -> Option<&Mock> {
        self.entries.get(key).filter(|mock| mock.matches_key(key))
    }
}

impl Mock {
    fn matches_key(&self, key: &str) -> bool {
        match self {
            Self::Read { .. } => true,
            Self::Http { method, url, .. } => key == format!("http:{method}:{url}"),
            Self::Command { program, args, .. } => {
                key == format!(
                    "command:{}",
                    std::iter::once(program.clone())
                        .chain(args.iter().cloned())
                        .collect::<Vec<_>>()
                        .join("\u{1f}")
                )
            }
        }
    }
}
