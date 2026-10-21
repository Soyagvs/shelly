use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExecutionMode {
    Normal,
    DryRun,
}

#[derive(Debug)]
pub(crate) enum AuditAction {
    Path {
        action: &'static str,
        path: String,
    },
    FromTo {
        action: &'static str,
        from: String,
        to: String,
    },
    Archive {
        operation: String,
    },
    Temp {
        kind: &'static str,
        capture: String,
    },
    Process {
        program: String,
        argc: usize,
    },
    Http {
        method: String,
        url: String,
    },
    Shell,
    Wait,
    Watch {
        argc: usize,
    },
    Log {
        identifier: String,
    },
    Input {
        kind: &'static str,
    },
}
impl AuditAction {
    fn encode(self) -> (&'static str, Value, String) {
        match self {
            Self::Path { action, path } => (
                action,
                json!({"path":path.clone()}),
                format!("{action} path={}", human_quote(&path)),
            ),
            Self::FromTo { action, from, to } => (
                action,
                json!({"from":from.clone(),"to":to.clone()}),
                format!(
                    "{action} from={} to={}",
                    human_quote(&from),
                    human_quote(&to)
                ),
            ),
            Self::Archive { operation } => {
                ("archive", json!({"operation":operation}), "archive".into())
            }
            Self::Temp { kind, capture } => (
                "temp",
                json!({"kind":kind,"capture":capture}),
                "temp".into(),
            ),
            Self::Process { program, argc } => (
                "command",
                json!({"program":program,"argc":argc}),
                format!("command program={} argc={argc}", human_quote(&program)),
            ),
            Self::Http { method, url } => (
                "http",
                {
                    let safe_url = redact_url(&url);
                    json!({"method":method.clone(),"url":safe_url.clone()})
                },
                format!(
                    "http method={} url={}",
                    human_quote(&method),
                    human_quote(&redact_url(&url))
                ),
            ),
            Self::Shell => ("shell", json!({"validated":true}), "shell".into()),
            Self::Wait => ("wait", json!({"blocked":true}), "planned blocked".into()),
            Self::Watch { argc } => ("watch", json!({"argc":argc}), format!("watch argc={argc}")),
            Self::Log { identifier } => ("log", json!({"identifier":identifier}), "log".into()),
            Self::Input { kind } => ("input", json!({"kind":kind,"blocked":true}), "input".into()),
        }
    }
}
fn human_quote(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => escaped.push_str(&format!("\\u{{{:04x}}}", c as u32)),
            c => escaped.push(c),
        }
    }
    escaped.push('"');
    escaped
}
fn redact_url(raw: &str) -> String {
    match url::Url::parse(raw) {
        Ok(mut url) => {
            let _ = url.set_username("");
            let _ = url.set_password(None);
            url.set_query(None);
            url.set_fragment(None);
            url.to_string()
        }
        Err(_) => "[invalid-url]".into(),
    }
}
#[derive(Debug)]
pub(crate) struct AuditWriter {
    file: File,
    seq: u64,
    counts: BTreeMap<String, u64>,
}
#[derive(Serialize)]
struct Record {
    schema: &'static str,
    version: u8,
    seq: u64,
    event: &'static str,
    action: String,
    status: &'static str,
    data: Value,
}
impl AuditWriter {
    pub(crate) fn create(path: &Path) -> Result<Self, String> {
        Ok(Self {
            file: OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map_err(|e| format!("could not create audit log '{}': {e}", path.display()))?,
            seq: 0,
            counts: BTreeMap::new(),
        })
    }
    pub(crate) fn record(&mut self, action: AuditAction) -> Result<(), String> {
        let (kind, data, human) = action.encode();
        println!("DRY-RUN action: {kind} {human}");
        let seq = self.seq + 1;
        serde_json::to_writer(
            &mut self.file,
            &Record {
                schema: "shelly.audit.v1",
                version: 1,
                seq,
                event: "action",
                action: kind.into(),
                status: "planned",
                data,
            },
        )
        .map_err(|e| format!("could not write audit log: {e}"))?;
        self.file
            .write_all(b"\n")
            .map_err(|e| format!("could not write audit log: {e}"))?;
        self.seq = seq;
        *self.counts.entry(kind.into()).or_default() += 1;
        Ok(())
    }
    pub(crate) fn error(&mut self, reason: &str) -> Result<(), String> {
        let reason = crate::secret::redact(reason);
        self.write("error", "summary", "error", json!({"reason":reason}))
    }
    pub(crate) fn finish(&mut self) -> Result<(), String> {
        self.write(
            "summary",
            "summary",
            "complete",
            json!({"counts":self.counts}),
        )
    }
    fn write(
        &mut self,
        event: &'static str,
        action: &'static str,
        status: &'static str,
        data: Value,
    ) -> Result<(), String> {
        let seq = self.seq + 1;
        serde_json::to_writer(
            &mut self.file,
            &Record {
                schema: "shelly.audit.v1",
                version: 1,
                seq,
                event,
                action: action.into(),
                status,
                data,
            },
        )
        .map_err(|e| format!("could not write audit log: {e}"))?;
        self.file
            .write_all(b"\n")
            .map_err(|e| format!("could not write audit log: {e}"))?;
        self.seq = seq;
        Ok(())
    }
}
#[derive(Debug)]
pub(crate) struct ExecutionContext {
    pub(crate) mode: ExecutionMode,
    pub(crate) audit: Option<AuditWriter>,
}
impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::Normal,
            audit: None,
        }
    }
}
impl ExecutionContext {
    pub(crate) fn dry_run(audit: Option<AuditWriter>) -> Self {
        Self {
            mode: ExecutionMode::DryRun,
            audit,
        }
    }
    pub(crate) fn is_dry_run(&self) -> bool {
        self.mode == ExecutionMode::DryRun
    }
}
pub(crate) fn record(c: &mut ExecutionContext, action: AuditAction) -> Result<(), String> {
    if let Some(w) = &mut c.audit {
        w.record(action)
    } else {
        let (kind, _, human) = action.encode();
        println!("DRY-RUN action: {kind} {human}");
        Ok(())
    }
}
pub(crate) fn finish(c: &mut ExecutionContext) -> Result<(), String> {
    if let Some(w) = &mut c.audit {
        w.finish()
    } else {
        Ok(())
    }
}
