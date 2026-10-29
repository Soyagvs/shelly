use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod archive;
mod audit;
mod comments;
pub mod formatter;
#[doc(hidden)]
pub mod http;
pub mod lsp;
mod mock;
mod parser;
mod secret;
mod shell;
pub mod update;
mod value;

use comments::strip_comments;
use parser::{
    is_identifier_name, is_number_literal, parse_identifier, parse_string_literal, parse_var_ref,
    split_commas, split_string_literal_tail,
};
use shell::{read_stdin_line, shell_command};
use value::{
    IdentKind, Value, compare_values, display_identifier, format_number, format_value,
    is_simple_key, is_truthy,
};

const WHILE_ITERATION_LIMIT: usize = 10_000;
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn run_lsp() {
    lsp::run(check_source);
}

pub fn cli_main() {
    let version = env::args_os().skip(1).next();
    if version.as_deref() == Some(std::ffi::OsStr::new("--version")) {
        println!("shelly {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if env::args_os()
        .next()
        .and_then(|path| {
            PathBuf::from(path)
                .file_stem()
                .map(|name| name == "shelly-lsp")
        })
        .unwrap_or(false)
    {
        run_lsp();
        return;
    }
    let mut args = env::args_os().skip(1);
    if args.next().as_deref() == Some(std::ffi::OsStr::new("--shelly-update-helper")) {
        if let Err(error) = update::helper(args) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    let mut raw_args: Vec<_> = env::args_os().skip(1).collect();
    let dry_run = raw_args.first().is_some_and(|arg| arg == "--dry-run");
    if dry_run {
        raw_args.remove(0);
    }
    let audit_count = raw_args.iter().filter(|a| *a == "--audit-log").count();
    if audit_count > 1 {
        eprintln!("--audit-log may be specified only once");
        std::process::exit(1);
    }
    let audit_path = if dry_run && raw_args.first().is_some_and(|a| a == "--audit-log") {
        raw_args.remove(0);
        match raw_args.get(0).cloned() {
            Some(p) if p != "--audit-log" => {
                raw_args.remove(0);
                Some(PathBuf::from(p))
            }
            _ => {
                eprintln!("--audit-log requires a path");
                std::process::exit(1);
            }
        }
    } else if raw_args.iter().any(|a| a == "--audit-log") {
        eprintln!("--audit-log must immediately follow --dry-run and precede the script");
        std::process::exit(1);
    } else {
        None
    };
    if dry_run
        && matches!(
            raw_args.first().and_then(|a| a.to_str()),
            Some("init" | "fmt" | "update" | "test" | "run" | "check")
        )
    {
        eprintln!("--dry-run supports a direct script path, not a top-level command");
        std::process::exit(1);
    }
    if dry_run && raw_args.iter().skip(1).any(|a| a == "--audit-log") {
        eprintln!("--audit-log must precede the script path");
        std::process::exit(1);
    }
    let mut args = raw_args.into_iter();
    let result = match args.next() {
        Some(command) if command == "init" => init_project(args),
        Some(command) if command == "test" => run_tests(args),
        Some(command) if command == "run" => run_project(args),
        Some(command) if command == "update" => update::run(args),
        Some(command) if command == "fmt" => format_command(args),
        Some(command) if command == "check" => {
            let mut format = CheckFormat::Text;
            let next = args.next();
            let script_path = if next.as_deref() == Some(std::ffi::OsStr::new("--format")) {
                match args.next().and_then(|value| value.into_string().ok()) {
                    Some(value) if value == "json" => {
                        format = CheckFormat::Json;
                        Ok(args.next().map(PathBuf::from))
                    }
                    Some(value) => Err(format!("unsupported check format: {value}")),
                    None => Err("missing format after --format".to_string()),
                }
            } else {
                Ok(next.map(PathBuf::from))
            };
            match script_path {
                Ok(Some(path)) => check_script(path, format),
                Ok(None) => check_project(format),
                Err(error) => Err(error),
            }
        }
        Some(path) => {
            let script_args = args.map(|arg| arg.to_string_lossy().into_owned()).collect();
            let context = if dry_run {
                audit::ExecutionContext::dry_run(
                    audit_path
                        .as_deref()
                        .map(audit::AuditWriter::create)
                        .transpose()
                        .unwrap_or_else(|e| {
                            eprintln!("{e}");
                            std::process::exit(1)
                        }),
                )
            } else {
                audit::ExecutionContext::default()
            };
            run_script(PathBuf::from(path), script_args, context)
        }
        None => {
            println!("Shelly is ready. Pass a .shelly script path to run it.");
            println!("Usage: shelly <script.shelly>");
            Ok(())
        }
    };

    if let Err(error) = result {
        if let Some(code) = parse_exit_signal(&error) {
            std::process::exit(code);
        }
        if error != "__shelly_check_failed" {
            eprintln!("{}", secret::redact(&error));
        }
        std::process::exit(1);
    }
}

#[derive(Deserialize)]
struct Manifest {
    project: ProjectManifest,
    #[serde(default)]
    test: TestManifest,
}

#[derive(Deserialize)]
struct ProjectManifest {
    name: String,
    version: String,
    entry: String,
}

#[derive(Deserialize, Default)]
struct TestManifest {
    directory: Option<String>,
    pattern: Option<String>,
}

fn load_manifest(cwd: &Path) -> Result<Manifest, String> {
    let path = cwd.join("Shelly.toml");
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("could not read Shelly.toml: {error}"))?;
    let manifest: Manifest = match toml::from_str(&source) {
        Ok(manifest) => manifest,
        Err(first_error) => {
            let legacy = source.replace("\\n", "\n").replace("\\\"", "\"");
            toml::from_str(&legacy)
                .map_err(|_| format!("could not parse manifest Shelly.toml: {first_error}"))?
        }
    };
    for (field, value) in [
        ("name", &manifest.project.name),
        ("version", &manifest.project.version),
        ("entry", &manifest.project.entry),
    ] {
        if value.trim().is_empty() {
            return Err(format!("project {field} must be a nonempty string"));
        }
    }
    Ok(manifest)
}

fn format_command(mut args: impl Iterator<Item = std::ffi::OsString>) -> Result<(), String> {
    let first = args.next();
    let (check, path) = if first.as_deref() == Some(std::ffi::OsStr::new("--check")) {
        (true, args.next().map(PathBuf::from))
    } else {
        (false, first.map(PathBuf::from))
    };
    if args.next().is_some() || path.is_none() {
        return Err("Usage: shelly fmt [--check] <path>".to_string());
    }
    let path = path.unwrap();
    let clean = formatter::format_file(&path, check)?;
    if check && !clean {
        return Err("formatting differs (run 'shelly fmt <path>')".to_string());
    }
    Ok(())
}

fn check_project(format: CheckFormat) -> Result<(), String> {
    let cwd = env::current_dir()
        .map_err(|error| format!("could not determine current directory: {error}"))?;
    if !cwd.join("Shelly.toml").is_file() {
        return Err("Shelly.toml is required in the current directory".to_string());
    }
    let manifest = load_manifest(&cwd)?;
    let entry = cwd.join(&manifest.project.entry);
    if !entry.is_file() {
        return Err(format!(
            "project entry '{}' does not exist",
            manifest.project.entry
        ));
    }
    check_script(entry, format)
}

fn run_project(args: impl Iterator<Item = std::ffi::OsString>) -> Result<(), String> {
    let mut forwarded = Vec::new();
    let mut separator_seen = false;
    for arg in args {
        if !separator_seen {
            if arg == std::ffi::OsStr::new("--") {
                separator_seen = true;
            } else {
                return Err(format!(
                    "Usage: shelly run [-- <args...>] (unexpected argument '{}')",
                    arg.to_string_lossy()
                ));
            }
        } else {
            forwarded.push(arg.to_string_lossy().into_owned());
        }
    }

    let cwd = env::current_dir()
        .map_err(|error| format!("could not determine current directory: {error}"))?;
    if !cwd.join("Shelly.toml").is_file() {
        return Err("Shelly.toml is required in the current directory".to_string());
    }
    let manifest = load_manifest(&cwd)?;
    let entry = cwd.join(&manifest.project.entry);
    if !entry.is_file() {
        return Err(format!(
            "project entry '{}' does not exist",
            manifest.project.entry
        ));
    }
    run_script(entry, forwarded, audit::ExecutionContext::default())
}

fn run_tests(mut args: impl Iterator<Item = std::ffi::OsString>) -> Result<(), String> {
    if let Some(extra) = args.next() {
        return Err(format!(
            "Usage: shelly test (unexpected extra argument '{}')",
            extra.to_string_lossy()
        ));
    }

    let cwd = env::current_dir()
        .map_err(|error| format!("could not determine current directory: {error}"))?;
    if !cwd.join("Shelly.toml").is_file() {
        return Err("Shelly.toml is required in the current directory".to_string());
    }
    let manifest = load_manifest(&cwd)?;
    let tests_dir = cwd.join(manifest.test.directory.as_deref().unwrap_or("tests"));
    let pattern = manifest.test.pattern.as_deref().unwrap_or("*_test.shelly");
    if !tests_dir.is_dir() {
        return Err("tests directory is required in the current directory".to_string());
    }

    let mut tests = Vec::new();
    collect_test_files(&tests_dir, &cwd, pattern, &mut tests)?;
    tests.sort();
    if tests.is_empty() {
        return Err("tests directory contains no *_test.shelly files".to_string());
    }

    let executable = env::current_exe()
        .map_err(|error| format!("could not determine Shelly executable: {error}"))?;
    let mut passed = 0;
    let mut failed = 0;
    for relative in tests {
        let output = Command::new(&executable)
            .arg(&relative)
            .env("SHELLY_TEST_MODE", "1")
            .current_dir(&cwd)
            .output()
            .map_err(|error| format!("could not run test '{relative}': {error}"))?;
        if output.status.success() {
            println!("PASS {relative}");
            passed += 1;
        } else {
            println!("FAIL {relative}");
            failed += 1;
            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
    }
    println!("{passed} passed, {failed} failed");
    if failed == 0 {
        Ok(())
    } else {
        Err("one or more tests failed".to_string())
    }
}

fn collect_test_files(
    dir: &Path,
    cwd: &Path,
    pattern: &str,
    tests: &mut Vec<String>,
) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|error| {
        format!(
            "could not read tests directory '{}': {error}",
            dir.display()
        )
    })? {
        let entry = entry.map_err(|error| format!("could not read test entry: {error}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_test_files(&path, cwd, pattern, tests)?;
        } else if path.is_file()
            && path
                .file_name()
                .is_some_and(|name| wildcard_match(pattern, &name.to_string_lossy()))
        {
            let relative = path
                .strip_prefix(cwd)
                .map_err(|error| format!("could not normalize test path: {error}"))?;
            tests.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let (mut p, mut v, mut star, mut mark) = (0, 0, None, 0);
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    while v < value.len() {
        if p < pattern.len() && (pattern[p] == value[v] || pattern[p] == b'?') {
            p += 1;
            v += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            mark = v;
            p += 1;
        } else if let Some(s) = star {
            p = s + 1;
            mark += 1;
            v = mark;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
}

fn init_project(mut args: impl Iterator<Item = std::ffi::OsString>) -> Result<(), String> {
    if let Some(extra) = args.next() {
        return Err(format!(
            "Usage: shelly init (unexpected extra argument '{}')",
            extra.to_string_lossy()
        ));
    }

    let cwd = env::current_dir()
        .map_err(|error| format!("could not determine current directory: {error}"))?;
    let project_name = cwd
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "could not represent the current directory name as UTF-8".to_string())?;

    let files = ["Shelly.toml", "main.shelly", "README.md", ".gitignore"];
    let directories = [
        "tests",
        "consts",
        "functions",
        "utils",
        "setup",
        "scripts",
        "modules",
        "domains",
    ];
    let mut targets = Vec::new();
    targets.extend(files.iter().map(|name| cwd.join(name)));
    for directory in directories {
        let path = cwd.join(directory);
        targets.push(path.clone());
        targets.push(path.join(".gitkeep"));
    }
    if let Some(existing) = targets
        .iter()
        .find(|path| fs::symlink_metadata(path).is_ok())
    {
        return Err(format!(
            "cannot initialize project: target already exists: {}",
            existing.display()
        ));
    }

    let toml_name = project_name.replace('\\', "\\\\").replace('"', "\\\"");
    let contents = [
        (
            "Shelly.toml",
            format!(
                "[project]\nname = \"{toml_name}\"\nversion = \"0.1.0\"\nentry = \"main.shelly\"\n"
            ),
        ),
        (
            "main.shelly",
            "# Hello, Shelly #\n\nfn greet($name) {\n    if $name == \"world\"\n        shellout \"Hello, Shelly!\"\n    else\n        shellout \"Hello, $name\"\n    end\n}\n\ncall greet(\"world\")\n"
                .to_string(),
        ),
        ("README.md", format!("# {project_name}\n")),
        (".gitignore", "logs/\n".to_string()),
    ];
    for (name, content) in contents {
        fs::write(cwd.join(name), content)
            .map_err(|error| format!("could not create {name}: {error}"))?;
    }
    for directory in directories {
        let path = cwd.join(directory);
        fs::create_dir(&path)
            .map_err(|error| format!("could not create {}: {error}", path.display()))?;
        fs::write(path.join(".gitkeep"), "")
            .map_err(|error| format!("could not create {directory}/.gitkeep: {error}"))?;
    }
    Ok(())
}

fn exit_signal(code: u8) -> String {
    format!("__shelly_exit:{code}")
}

fn break_signal() -> String {
    "__shelly_break".to_string()
}

fn continue_signal() -> String {
    "__shelly_continue".to_string()
}

fn return_signal(value: String) -> String {
    format!("__shelly_return:{value}")
}

fn parse_exit_signal(error: &str) -> Option<i32> {
    error
        .strip_prefix("__shelly_exit:")
        .and_then(|code| code.parse::<i32>().ok())
}

fn parse_return_signal(error: &str) -> Option<String> {
    error
        .strip_prefix("__shelly_return:")
        .map(ToString::to_string)
}

fn is_break_signal(error: &str) -> bool {
    error == "__shelly_break"
}

fn is_continue_signal(error: &str) -> bool {
    error == "__shelly_continue"
}

fn is_control_signal(error: &str) -> bool {
    parse_exit_signal(error).is_some()
        || parse_return_signal(error).is_some()
        || is_break_signal(error)
        || is_continue_signal(error)
}

fn line_error(line_no: usize, error: String) -> String {
    if is_control_signal(&error) {
        error
    } else {
        format!("line {line_no}: {error}")
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CheckFormat {
    Text,
    Json,
}

#[derive(Clone)]
pub(crate) struct Diagnostic {
    pub(crate) file: String,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) severity: &'static str,
    pub(crate) message: String,
}

fn check_script(path: PathBuf, format: CheckFormat) -> Result<(), String> {
    let path = path
        .canonicalize()
        .map_err(|error| format!("could not read script '{}': {error}", path.display()))?;
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("could not read script '{}': {error}", path.display()))?;
    let diagnostics = check_source(&source, &path.display().to_string());
    if format == CheckFormat::Json {
        println!("{}", format_diagnostics_json(&diagnostics));
    } else {
        for diagnostic in &diagnostics {
            eprintln!(
                "{}: line {}: {}",
                diagnostic.file, diagnostic.line, diagnostic.message
            );
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err("__shelly_check_failed".to_string())
    }
}

fn format_diagnostics_json(diagnostics: &[Diagnostic]) -> String {
    let mut out = String::from("[");
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"file\":\"{}\",\"line\":{},\"column\":{},\"severity\":\"{}\",\"message\":\"{}\"}}",
            json_escape(&diagnostic.file),
            diagnostic.line,
            diagnostic.column,
            diagnostic.severity,
            json_escape(&diagnostic.message)
        ));
    }
    out.push(']');
    out
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn normalize_multiline_strings(source: &str) -> Result<String, String> {
    let raw_lines: Vec<&str> = source.lines().collect();
    let mut out = Vec::new();
    let mut index = 0;
    while index < raw_lines.len() {
        let line_no = index + 1;
        let raw_line = raw_lines[index];
        if let Some(delimiter) = find_unquoted_triple_backtick(raw_line) {
            if !raw_line[delimiter + 3..].trim().is_empty() {
                return Err(format!(
                    "line {line_no}: unexpected text after multiline string delimiter"
                ));
            }
            let prefix = &raw_line[..delimiter];
            let mut body = String::new();
            out.push(String::new());
            index += 1;
            let mut end_index = None;
            while index < raw_lines.len() {
                if raw_lines[index].trim() == "```" {
                    end_index = Some(index);
                    break;
                }
                if !body.is_empty() {
                    body.push('\n');
                }
                body.push_str(raw_lines[index]);
                out.push(String::new());
                index += 1;
            }
            let Some(end_index) = end_index else {
                return Err(format!("line {line_no}: unterminated multiline string"));
            };
            let mut combined = format!("{}{}", prefix, encode_string_literal(&body));
            index = end_index + 1;
            if index < raw_lines.len() && raw_lines[index].trim() == ")" {
                combined.push(')');
                index += 1;
                out.push(String::new());
            }
            out[line_no - 1] = combined;
            out.push(String::new());
            continue;
        }
        out.push(raw_line.to_string());
        index += 1;
    }
    Ok(out.join("\n"))
}

fn combine_dictionary_literals(lines: &[(usize, String)]) -> Result<Vec<(usize, String)>, String> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let (line_no, line) = &lines[index];
        let trimmed = line.trim();
        if trimmed.starts_with('$')
            && trimmed.contains('=')
            && trimmed
                .split_once('=')
                .is_some_and(|(_, rhs)| rhs.trim_start().starts_with('{'))
        {
            let mut combined = line.clone();
            let mut depth = brace_delta(line)?;
            while depth > 0 {
                index += 1;
                if index >= lines.len() {
                    return Err(format!("line {line_no}: malformed dictionary literal"));
                }
                let next = lines[index].1.trim();
                if !combined.trim_end().ends_with('{')
                    && !combined.trim_end().ends_with(',')
                    && next != "}"
                {
                    combined.push(',');
                }
                combined.push(' ');
                combined.push_str(next);
                depth += brace_delta(&lines[index].1)?;
            }
            out.push((*line_no, combined));
        } else {
            out.push((*line_no, line.clone()));
        }
        index += 1;
    }
    Ok(out)
}

fn brace_delta(input: &str) -> Result<i32, String> {
    let mut in_string = false;
    let mut escaped = false;
    let mut delta = 0;
    for ch in input.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '{' if !in_string => delta += 1,
            '}' if !in_string => delta -= 1,
            _ => {}
        }
    }
    if in_string {
        return Err("unterminated string literal".to_string());
    }
    Ok(delta)
}

fn find_unquoted_triple_backtick(input: &str) -> Option<usize> {
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '`' if !in_string && input[idx..].starts_with("```") => return Some(idx),
            _ => {}
        }
    }
    None
}

fn encode_string_literal(input: &str) -> String {
    let mut out = String::from("\"");
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn lexical_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

pub(crate) fn check_source(source: &str, file: &str) -> Vec<Diagnostic> {
    check_source_with_loader(source, file, &|path| fs::read_to_string(path).ok())
}

pub(crate) fn check_source_with_files(
    source: &str,
    file: &str,
    files: &HashMap<PathBuf, String>,
) -> Vec<Diagnostic> {
    check_source_with_loader(source, file, &|path| {
        files
            .get(path)
            .cloned()
            .or_else(|| fs::read_to_string(path).ok())
    })
}

fn check_source_with_loader(
    source: &str,
    file: &str,
    loader: &dyn Fn(&Path) -> Option<String>,
) -> Vec<Diagnostic> {
    let source = match normalize_multiline_strings(source) {
        Ok(source) => source,
        Err(error) => {
            return vec![Diagnostic {
                file: file.to_string(),
                line: 1,
                column: 1,
                severity: "error",
                message: error,
            }];
        }
    };
    match strip_comments(&source) {
        Ok(lines) => {
            let lines = match combine_dictionary_literals(&lines) {
                Ok(lines) => lines,
                Err(error) => {
                    return vec![Diagnostic {
                        file: file.to_string(),
                        line: 1,
                        column: 1,
                        severity: "error",
                        message: error,
                    }];
                }
            };
            let mut checker = Checker::new(file.to_string());
            checker.check_lines(&lines);
            let mut diagnostics = checker.diagnostics;
            let path = lexical_path(Path::new(file));
            {
                let mut stack = vec![path.clone()];
                let mut visited = HashSet::new();
                visited.insert(path);
                check_module_graph(
                    &lines,
                    Path::new(file),
                    &mut stack,
                    &mut visited,
                    &mut diagnostics,
                    loader,
                );
            }
            diagnostics
        }
        Err(error) => vec![Diagnostic {
            file: file.to_string(),
            line: 1,
            column: 1,
            severity: "error",
            message: error,
        }],
    }
}

fn check_module_graph(
    lines: &[(usize, String)],
    current_file: &Path,
    stack: &mut Vec<PathBuf>,
    visited: &mut HashSet<PathBuf>,
    diagnostics: &mut Vec<Diagnostic>,
    loader: &dyn Fn(&Path) -> Option<String>,
) {
    for (line_no, raw) in lines {
        let line = raw.trim();
        let (kind, rest) = if let Some(rest) = line.strip_prefix("import ") {
            ("import", rest)
        } else if let Some(rest) = line.strip_prefix("include ") {
            ("include", rest)
        } else {
            continue;
        };
        let requested = match parse_string_literal(rest.trim())
            .or_else(|_| {
                let cleaned = rest.trim().replace('\\', "");
                parse_string_literal(&cleaned)
            })
            .or_else(|_| {
                let text = rest.trim();
                text.find('"')
                    .and_then(|start| {
                        text[start + 1..].find('"').map(|offset| {
                            text[start + 1..start + 1 + offset]
                                .trim_matches('\\')
                                .to_string()
                        })
                    })
                    .ok_or_else(|| "expected string literal".to_string())
            }) {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(Diagnostic {
                    file: current_file.display().to_string(),
                    line: *line_no,
                    column: 1,
                    severity: "error",
                    message: format!("invalid {kind} dependency: {error}"),
                });
                continue;
            }
        };
        let dependency = match current_file.parent() {
            Some(parent) => parent.join(&requested),
            None => PathBuf::from(&requested),
        };
        let dependency = lexical_path(&dependency);
        if let Some(index) = stack.iter().position(|path| path == &dependency) {
            let chain = stack[index..]
                .iter()
                .chain(std::iter::once(&dependency))
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(" -> ");
            diagnostics.push(Diagnostic {
                file: current_file.display().to_string(),
                line: *line_no,
                column: 1,
                severity: "error",
                message: format!("import/include cycle: {chain}"),
            });
            continue;
        }
        let already_visited = !visited.insert(dependency.clone());
        let dependency_source = match loader(&dependency) {
            Some(source) => source,
            None => {
                diagnostics.push(Diagnostic {
                    file: current_file.display().to_string(),
                    line: *line_no,
                    column: 1,
                    severity: "error",
                    message: format!(
                        "could not read {kind} dependency '{}'",
                        dependency.display()
                    ),
                });
                continue;
            }
        };
        let normalized = match normalize_multiline_strings(&dependency_source) {
            Ok(source) => source,
            Err(error) => {
                diagnostics.push(Diagnostic {
                    file: dependency.display().to_string(),
                    line: 1,
                    column: 1,
                    severity: "error",
                    message: error,
                });
                continue;
            }
        };
        let dependency_lines = match strip_comments(&normalized)
            .and_then(|lines| combine_dictionary_literals(&lines))
        {
            Ok(lines) => lines,
            Err(error) => {
                diagnostics.push(Diagnostic {
                    file: dependency.display().to_string(),
                    line: 1,
                    column: 1,
                    severity: "error",
                    message: error,
                });
                continue;
            }
        };
        let mut checker = Checker::new(dependency.display().to_string());
        checker.check_lines(&dependency_lines);
        diagnostics.extend(checker.diagnostics);
        if !already_visited {
            stack.push(dependency.clone());
            check_module_graph(
                &dependency_lines,
                &dependency,
                stack,
                visited,
                diagnostics,
                loader,
            );
            stack.pop();
        }
    }
}

fn run_script(
    path: PathBuf,
    script_args: Vec<String>,
    context: audit::ExecutionContext,
) -> Result<(), String> {
    let path = path
        .canonicalize()
        .map_err(|error| format!("could not read script '{}': {error}", path.display()))?;
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("could not read script '{}': {error}", path.display()))?;
    let mut interpreter = Interpreter::default();
    interpreter.context = context;
    interpreter.script_args = script_args;
    interpreter.current_dir = path.parent().map(PathBuf::from).unwrap_or_default();
    interpreter.stack.push(path);
    let result = interpreter.run(&source);
    if interpreter.context.is_dry_run() {
        match result {
            Ok(()) => audit::finish(&mut interpreter.context),
            Err(error) => {
                if let Some(writer) = &mut interpreter.context.audit {
                    if let Err(audit_error) = writer.error(&error) {
                        return Err(audit_error);
                    }
                }
                Err(error)
            }
        }
    } else {
        result
    }
}

#[derive(Default)]
struct Interpreter {
    vars: HashMap<String, Value>,
    unavailable: HashSet<String>,
    consts: HashMap<String, Value>,
    functions: HashMap<String, Function>,
    current_dir: PathBuf,
    stack: Vec<PathBuf>,
    imported: HashSet<PathBuf>,
    logs: Vec<LogContext>,
    script_args: Vec<String>,
    pending_process_error: Option<BTreeMap<String, Value>>,
    deadline: Option<Instant>,
    defers: Vec<Vec<Vec<(usize, String)>>>,
    context: audit::ExecutionContext,
    mocks: mock::MockRegistry,
}

#[derive(Clone)]
struct LogContext {
    path: PathBuf,
}

impl Interpreter {
    fn effect(&mut self, action: audit::AuditAction) -> Result<bool, String> {
        if self.context.is_dry_run() {
            audit::record(&mut self.context, action)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn run(&mut self, source: &str) -> Result<(), String> {
        let source = normalize_multiline_strings(source)?;
        let lines = strip_comments(&source)?;
        let lines = combine_dictionary_literals(&lines)?;
        match self.run_lines(&lines) {
            Err(error) if is_break_signal(&error) => Err("break outside loop".to_string()),
            Err(error) if is_continue_signal(&error) => Err("continue outside loop".to_string()),
            Err(error) if parse_return_signal(&error).is_some() => {
                Err("return outside function".to_string())
            }
            result => result,
        }
    }

    fn run_lines(&mut self, lines: &[(usize, String)]) -> Result<(), String> {
        self.defers.push(Vec::new());
        let result = self.run_lines_inner(lines);
        let defers = self.defers.pop().unwrap_or_default();
        let mut cleanup_error = None;
        for body in defers.into_iter().rev() {
            if let Err(error) = self.run_lines(&body) {
                cleanup_error = Some(error);
                break;
            }
        }
        match result {
            Err(error) => Err(error),
            Ok(()) => cleanup_error.map_or(Ok(()), Err),
        }
    }

    fn run_lines_inner(&mut self, lines: &[(usize, String)]) -> Result<(), String> {
        let mut index = 0;
        while index < lines.len() {
            if self
                .deadline
                .is_some_and(|deadline| Instant::now() >= deadline)
            {
                return Err("timeout".to_string());
            }
            let line_no = lines[index].0;
            let line = lines[index].1.trim();
            if line.is_empty() || line.starts_with('#') {
                index += 1;
                continue;
            }
            if line.starts_with("mock ") {
                if env::var_os("SHELLY_TEST_MODE").is_none() {
                    return Err(format!(
                        "line {line_no}: mock declarations are only allowed in shelly test"
                    ));
                }
                let (key, declaration) =
                    parse_mock_declaration(line).map_err(|error| line_error(line_no, error))?;
                self.mocks
                    .add(key, declaration)
                    .map_err(|error| line_error(line_no, error))?;
                index += 1;
                continue;
            }
            if line == "mock" {
                return Err(format!("line {line_no}: mock declaration required"));
            }
            if let Some(rest) = line.strip_prefix("export fn ") {
                let (function, next_index) = parse_function(rest.trim(), lines, index)
                    .map_err(|error| line_error(line_no, error))?;
                self.define_function(function, line_no)?;
                index = next_index;
                continue;
            }
            if let Some(rest) = line.strip_prefix("export ") {
                let (kind, _, _) =
                    parse_identifier(rest).map_err(|error| line_error(line_no, error))?;
                if kind != IdentKind::Constant {
                    return Err(format!(
                        "line {line_no}: only constants and functions can be exported"
                    ));
                }
                self.assignment(rest, line_no)
                    .map_err(|error| line_error(line_no, error))?;
                index += 1;
                continue;
            }
            if let Some(rest) = line.strip_prefix("fn ") {
                let (function, next_index) = parse_function(rest.trim(), lines, index)
                    .map_err(|error| line_error(line_no, error))?;
                self.define_function(function, line_no)?;
                index = next_index;
                continue;
            }
            if line == "case" {
                return Err(format!("line {line_no}: case value required"));
            }
            if line == "when" || line.starts_with("when ") {
                return Err(format!("line {line_no}: when outside case"));
            }
            if let Some(value) = line.strip_prefix("case ") {
                let next_index = self
                    .run_case(value.trim(), lines, index)
                    .map_err(|error| line_error(line_no, error))?;
                index = next_index;
                continue;
            }
            if line == "if" {
                return Err(format!("line {line_no}: condition required"));
            }
            if let Some(condition) = line.strip_prefix("if ") {
                let next_index = self
                    .run_conditional(condition.trim(), lines, index)
                    .map_err(|error| line_error(line_no, error))?;
                index = next_index;
                continue;
            }
            if line == "repeat" {
                return Err(format!("line {line_no}: repeat count required"));
            }
            if let Some(rest) = line.strip_prefix("timeout ") {
                let seconds =
                    parse_timeout(rest.trim()).map_err(|error| line_error(line_no, error))?;
                let (body, next_index) = collect_block(lines, index, "timeout")?;
                let previous = self.deadline;
                let candidate = Instant::now() + seconds;
                self.deadline = Some(previous.map_or(candidate, |parent| parent.min(candidate)));
                let result = self.run_lines(&body);
                let expired = self
                    .deadline
                    .is_some_and(|deadline| Instant::now() >= deadline);
                self.deadline = previous;
                if expired {
                    return Err(line_error(line_no, "timeout".to_string()));
                }
                result.map_err(|error| line_error(line_no, error))?;
                index = next_index;
                continue;
            }
            if line == "defer" || line.starts_with("defer ") {
                if line != "defer" {
                    return Err(format!("line {line_no}: defer takes no arguments"));
                }
                let (body, next_index) = collect_block(lines, index, "defer")?;
                self.defers
                    .last_mut()
                    .expect("run_lines always establishes a scope")
                    .push(body);
                index = next_index;
                continue;
            }
            if line == "timeout" {
                return Err(format!("line {line_no}: timeout value required"));
            }
            if line == "wait" {
                if self.context.is_dry_run() {
                    audit::record(&mut self.context, audit::AuditAction::Wait)?;
                    index += 1;
                    continue;
                }
                loop {
                    thread::park();
                }
            }
            if line.starts_with("wait ") {
                return Err(format!("line {line_no}: wait takes no arguments"));
            }
            if let Some(rest) = line.strip_prefix("watch ") {
                let (command_text, handler_text) = rest.split_once("->").ok_or_else(|| {
                    "watch syntax is watch <program> <args...> -> $line".to_string()
                })?;
                let (program, args, tail) = self.parse_command_atoms(command_text.trim())?;
                if !tail.trim().is_empty() {
                    return Err("watch syntax is watch <program> <args...> -> $line".to_string());
                }
                let destinations = parse_command_capture_destinations(handler_text.trim())?;
                if destinations.len() != 1 {
                    return Err("watch requires exactly one line destination".to_string());
                }
                let (body, next_index) = collect_block(lines, index, "watch")?;
                if self.context.is_dry_run() {
                    audit::record(
                        &mut self.context,
                        audit::AuditAction::Watch { argc: args.len() },
                    )?;
                    index = next_index;
                    continue;
                }
                let mut child = Command::new(&program)
                    .args(&args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .spawn()
                    .map_err(|error| format!("failed to run watch command: {error}"))?;
                let previous = self
                    .vars
                    .insert(destinations[0].clone(), Value::Scalar(String::new()));
                let result = (|| {
                    let stdout = child
                        .stdout
                        .take()
                        .ok_or_else(|| "watch command stdout unavailable".to_string())?;
                    for line_result in io::BufReader::new(stdout).lines() {
                        let event = line_result
                            .map_err(|error| format!("failed to read watch output: {error}"))?;
                        self.vars
                            .insert(destinations[0].clone(), Value::Scalar(event));
                        self.run_lines(&body)?;
                    }
                    let status = child
                        .wait()
                        .map_err(|error| format!("failed to wait for watch command: {error}"))?;
                    if !status.success() {
                        return Err(format!("watch command exited with status {status}"));
                    }
                    Ok(())
                })();
                if result.is_err() {
                    let _ = child.kill();
                }
                let _ = child.wait();
                match previous {
                    Some(value) => {
                        self.vars.insert(destinations[0].clone(), value);
                    }
                    None => {
                        self.vars.remove(&destinations[0]);
                    }
                }
                result.map_err(|error: String| line_error(line_no, error))?;
                index = next_index;
                continue;
            }
            if let Some(condition) = line.strip_prefix("while ") {
                let next_index = self
                    .run_while(condition.trim(), lines, index)
                    .map_err(|error| line_error(line_no, error))?;
                index = next_index;
                continue;
            }
            if line == "while" {
                return Err(format!("line {line_no}: condition required"));
            }
            if let Some(rest) = line.strip_prefix("for ") {
                let next_index = self
                    .run_for(rest.trim(), lines, index)
                    .map_err(|error| line_error(line_no, error))?;
                index = next_index;
                continue;
            }
            if line == "for" {
                return Err(format!("line {line_no}: for syntax is for $item in $list"));
            }
            if let Some(rest) = line.strip_prefix("repeat ") {
                let next_index = self
                    .run_repeat(rest.trim(), lines, index)
                    .map_err(|error| line_error(line_no, error))?;
                index = next_index;
                continue;
            }
            if line == "try" || line.starts_with("try ") {
                let next_index = self
                    .run_try(line, lines, index)
                    .map_err(|error| line_error(line_no, error))?;
                index = next_index;
                continue;
            }
            if line == "log" {
                return Err(format!("line {line_no}: log identifier required"));
            }
            if let Some(rest) = line.strip_prefix("log ") {
                let next_index = self
                    .run_log(rest.trim(), lines, index)
                    .map_err(|error| line_error(line_no, error))?;
                index = next_index;
                continue;
            }
            if matches!(line, "elif" | "else" | "catch" | "end" | "when")
                || line.starts_with("elif ")
                || line.starts_with("when ")
            {
                return Err(format!("line {line_no}: unexpected conditional marker"));
            }
            self.run_line(line, line_no)
                .map_err(|error| line_error(line_no, error))?;
            index += 1;
        }
        Ok(())
    }

    fn run_conditional(
        &mut self,
        first_condition: &str,
        lines: &[(usize, String)],
        start: usize,
    ) -> Result<usize, String> {
        if first_condition.is_empty() {
            return Err("condition required".to_string());
        }
        let mut branches: Vec<(Option<String>, Vec<(usize, String)>)> =
            vec![(Some(first_condition.to_string()), Vec::new())];
        let mut depth = 0;
        let mut saw_else = false;
        let mut index = start + 1;
        while index < lines.len() {
            let line_no = lines[index].0;
            let line = lines[index].1.trim();
            if starts_block(line) {
                depth += 1;
            }
            if line == "}" && depth > 0 {
                depth -= 1;
            }
            if line == "end" {
                if depth == 0 {
                    for (condition, body) in branches {
                        let take = match condition {
                            Some(condition) => self.eval_condition(&condition)?,
                            None => true,
                        };
                        if take {
                            return self.run_lines(&body).map(|_| index + 1);
                        }
                    }
                    return Ok(index + 1);
                }
                depth -= 1;
            }
            if depth == 0 {
                if let Some(condition) = line.strip_prefix("elif ") {
                    if saw_else {
                        return Err(format!("line {line_no}: elif after else"));
                    }
                    if condition.trim().is_empty() {
                        return Err(format!("line {line_no}: condition required"));
                    }
                    branches.push((Some(condition.trim().to_string()), Vec::new()));
                    index += 1;
                    continue;
                }
                if line == "else" {
                    if saw_else {
                        return Err(format!("line {line_no}: duplicate else"));
                    }
                    saw_else = true;
                    branches.push((None, Vec::new()));
                    index += 1;
                    continue;
                }
            }
            branches
                .last_mut()
                .expect("branch exists")
                .1
                .push(lines[index].clone());
            index += 1;
        }
        Err("missing end for if".to_string())
    }

    fn run_case(
        &mut self,
        value_expr: &str,
        lines: &[(usize, String)],
        start: usize,
    ) -> Result<usize, String> {
        if value_expr.is_empty() {
            return Err("case value required".to_string());
        }
        let case_value = self.eval_expr(value_expr)?;
        let (branches, else_body, next_index) = collect_case_block(lines, start)?;
        for (values, body) in branches {
            for value in values {
                if self.eval_expr(&value)? == case_value {
                    self.run_lines(&body)?;
                    return Ok(next_index);
                }
            }
        }
        if let Some(body) = else_body {
            self.run_lines(&body)?;
        }
        Ok(next_index)
    }

    fn run_repeat(
        &mut self,
        rest: &str,
        lines: &[(usize, String)],
        start: usize,
    ) -> Result<usize, String> {
        let (count_expr, index_var) = parse_repeat_header(rest)?;
        let count = self.eval_repeat_count(count_expr)?;
        let (body, next_index) = collect_block(lines, start, "repeat")?;
        let saved_index = index_var.and_then(|name| {
            let previous = self.vars.get(&name).cloned();
            Some((name, previous))
        });

        let mut loop_result = Ok(());
        for iteration in 1..=count {
            if let Some((name, _)) = &saved_index {
                self.vars
                    .insert(name.clone(), Value::Scalar(iteration.to_string()));
            }
            match self.run_lines(&body) {
                Ok(()) => {}
                Err(error) if is_continue_signal(&error) => continue,
                Err(error) if is_break_signal(&error) => break,
                Err(error) => {
                    loop_result = Err(error);
                    break;
                }
            }
        }

        if let Some((name, previous)) = saved_index {
            if let Some(value) = previous {
                self.vars.insert(name, value);
            } else {
                self.vars.remove(&name);
            }
        }
        loop_result?;
        Ok(next_index)
    }

    fn run_while(
        &mut self,
        condition: &str,
        lines: &[(usize, String)],
        start: usize,
    ) -> Result<usize, String> {
        if condition.is_empty() {
            return Err("condition required".to_string());
        }
        let (body, next_index) = collect_block(lines, start, "while")?;
        let mut iterations = 0;
        while self.eval_condition(condition)? {
            if iterations >= WHILE_ITERATION_LIMIT {
                return Err(format!(
                    "while loop exceeded safety limit of {WHILE_ITERATION_LIMIT} iterations"
                ));
            }
            iterations += 1;
            match self.run_lines(&body) {
                Ok(()) => {}
                Err(error) if is_continue_signal(&error) => continue,
                Err(error) if is_break_signal(&error) => break,
                Err(error) => return Err(error),
            }
        }
        Ok(next_index)
    }

    fn run_for(
        &mut self,
        rest: &str,
        lines: &[(usize, String)],
        start: usize,
    ) -> Result<usize, String> {
        let (item_name, iterable) = parse_for_header(rest)?;
        let (body, next_index) = collect_block(lines, start, "for")?;
        let values = match self.eval_expr_value(&iterable)? {
            Value::List(values) => values,
            Value::Scalar(_) | Value::Dict(_) => {
                return Err("for iterable must be a list".to_string());
            }
        };

        for value in values {
            self.vars.insert(item_name.clone(), value);
            match self.run_lines(&body) {
                Ok(()) => {}
                Err(error) if is_continue_signal(&error) => continue,
                Err(error) if is_break_signal(&error) => break,
                Err(error) => return Err(error),
            }
        }

        Ok(next_index)
    }

    fn run_try(
        &mut self,
        line: &str,
        lines: &[(usize, String)],
        start: usize,
    ) -> Result<usize, String> {
        let error_var = parse_try_header(line)?;
        let (try_body, catch_body, next_index) = collect_try_block(lines, start)?;
        match self.run_lines(&try_body) {
            Ok(()) => Ok(next_index),
            Err(error) if is_control_signal(&error) => Err(error),
            Err(error) => {
                if let Some(name) = error_var {
                    let value = self.pending_process_error.take().map_or_else(
                        || {
                            let mut fields = BTreeMap::new();
                            fields.insert("kind".to_string(), Value::Scalar("runtime".to_string()));
                            fields.insert("message".to_string(), Value::Scalar(error.clone()));
                            Value::Dict(fields)
                        },
                        Value::Dict,
                    );
                    self.vars.insert(name, value);
                } else {
                    self.pending_process_error = None;
                }
                self.run_lines(&catch_body)?;
                Ok(next_index)
            }
        }
    }

    fn run_log(
        &mut self,
        identifier_expr: &str,
        lines: &[(usize, String)],
        start: usize,
    ) -> Result<usize, String> {
        if identifier_expr.is_empty() {
            return Err("log identifier required".to_string());
        }
        let identifier = self.eval_expr(identifier_expr)?;
        let (body, next_index) = collect_block(lines, start, "log")?;
        self.log_action(&format!("log: {identifier}"));
        let path = self.log_path(&identifier)?;
        let ctx = LogContext { path };
        self.write_log(&ctx, &format!("--- {identifier} ---"))?;
        self.write_log(&ctx, "ENTER")?;
        self.logs.push(ctx.clone());
        let result = self.run_lines(&body);
        self.logs.pop();
        match &result {
            Ok(()) => self.write_log(&ctx, "EXIT ok")?,
            Err(error) if is_control_signal(error) => self.write_log(
                &ctx,
                &format!("EXIT control {}", control_signal_name(error)),
            )?,
            Err(error) => self.write_log(&ctx, &format!("ERROR {error}"))?,
        }
        result?;
        Ok(next_index)
    }

    fn log_path(&mut self, identifier: &str) -> Result<PathBuf, String> {
        if self.context.is_dry_run() {
            audit::record(
                &mut self.context,
                audit::AuditAction::Log {
                    identifier: identifier.to_string(),
                },
            )?;
            return Err("dry-run unavailable-result: log destination was not created".into());
        }
        let dir = self.current_dir.join("logs");
        fs::create_dir_all(&dir).map_err(|error| {
            format!(
                "could not create log directory '{}': {error}",
                dir.display()
            )
        })?;
        Ok(dir.join(format!(
            "{}-{}.log",
            sanitize_log_identifier(identifier),
            current_utc_date()
        )))
    }

    fn timeout_failure(&mut self, program: &str, args: &[String]) -> String {
        let mut fields = BTreeMap::new();
        fields.insert("kind".into(), Value::Scalar("timeout".into()));
        fields.insert("program".into(), Value::Scalar(program.into()));
        fields.insert(
            "args".into(),
            Value::List(args.iter().cloned().map(Value::Scalar).collect()),
        );
        fields.insert("message".into(), Value::Scalar("command timed out".into()));
        self.pending_process_error = Some(fields);
        "command timed out".into()
    }

    fn log_action(&self, message: &str) {
        let safe = secret::redact(message);
        for ctx in &self.logs {
            let _ = self.write_log(ctx, &safe);
        }
    }

    fn write_log(&self, ctx: &LogContext, message: &str) -> Result<(), String> {
        if self.context.is_dry_run() {
            return Ok(());
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&ctx.path)
            .map_err(|error| format!("could not write log '{}': {error}", ctx.path.display()))?;
        writeln!(file, "{} {message}", current_utc_timestamp())
            .map_err(|error| format!("could not write log '{}': {error}", ctx.path.display()))
    }

    fn eval_repeat_count(&self, expr: &str) -> Result<i64, String> {
        if expr.contains('.') {
            return Err("repeat count must be an integer".to_string());
        }
        let mut parser = NumericParser::new(expr, self);
        let value = parser.parse_expr()?;
        parser.skip_ws();
        if !parser.is_at_end() {
            return Err(format!(
                "unexpected text in numeric expression near '{}'",
                parser.remaining()
            ));
        }
        if value < 0.0 {
            return Err("repeat count cannot be negative".to_string());
        }
        if value.fract() != 0.0 {
            return Err("repeat count must be an integer".to_string());
        }
        Ok(value as i64)
    }

    fn eval_exit_code(&self, expr: &str) -> Result<u8, String> {
        if expr.is_empty() {
            return Ok(0);
        }
        if expr.starts_with('"') || expr.starts_with('[') || matches!(expr, "true" | "false") {
            return Err("exit code must be an integer from 0 to 255".to_string());
        }
        let mut parser = NumericParser::new(expr, self);
        let value = parser
            .parse_expr()
            .map_err(|_| "exit code must be an integer from 0 to 255".to_string())?;
        parser.skip_ws();
        if !parser.is_at_end() {
            return Err(format!(
                "unexpected text in numeric expression near '{}'",
                parser.remaining()
            ));
        }
        if value.fract() != 0.0 {
            return Err("exit code must be an integer".to_string());
        }
        if !(0.0..=255.0).contains(&value) {
            return Err("exit code must be between 0 and 255".to_string());
        }
        Ok(value as u8)
    }

    fn archive_command(&mut self, text: &str) -> Result<(), String> {
        let (op, rest) = text
            .split_once(char::is_whitespace)
            .ok_or("archive syntax invalid")?;
        let (args, dest) = rest
            .trim()
            .split_once("->")
            .ok_or("archive capture required")?;
        if dest.contains("->") {
            return Err("archive capture must be unique".into());
        }
        let name = parse_var_ref(dest.trim())?;
        let args = args.trim();
        if self.context.is_dry_run() {
            self.mark_unavailable(name.clone());
            match op {
                "create" => {
                    let (a, b) = args
                        .split_once(" from ")
                        .ok_or("archive create syntax invalid")?;
                    if b.contains(" from ") {
                        return Err("archive create syntax invalid".into());
                    }
                    self.eval_string(a.trim())?;
                    self.eval_string(b.trim())?;
                }
                "extract" => {
                    let (a, b) = args
                        .split_once(" into ")
                        .ok_or("archive extract syntax invalid")?;
                    if b.contains(" into ") {
                        return Err("archive extract syntax invalid".into());
                    }
                    self.eval_string(a.trim())?;
                    self.eval_string(b.trim())?;
                }
                "checksum" | "check" => {
                    if args.is_empty() {
                        return Err(format!("archive {} operand required", op));
                    }
                    self.eval_string(args)?;
                }
                _ => return Err("unknown archive operation".into()),
            }
            audit::record(
                &mut self.context,
                audit::AuditAction::Archive {
                    operation: op.to_string(),
                },
            )?;
            return Ok(());
        }
        let value = match op {
            "create" => {
                let (a, b) = args
                    .split_once(" from ")
                    .ok_or("archive create syntax invalid")?;
                if b.contains(" from ") {
                    return Err("archive create syntax invalid".into());
                }
                archive::create(
                    Path::new(&self.eval_string(a.trim())?),
                    Path::new(&self.eval_string(b.trim())?),
                )?
            }
            "extract" => {
                let (a, b) = args
                    .split_once(" into ")
                    .ok_or("archive extract syntax invalid")?;
                if b.contains(" into ") {
                    return Err("archive extract syntax invalid".into());
                }
                archive::extract(
                    Path::new(&self.eval_string(a.trim())?),
                    Path::new(&self.eval_string(b.trim())?),
                )?
            }
            "checksum" => {
                if args.is_empty() {
                    return Err("archive checksum operand required".into());
                } else {
                    archive::checksum(Path::new(&self.eval_string(args)?))?
                }
            }
            "check" => {
                if args.is_empty() {
                    return Err("archive check operand required".into());
                } else {
                    archive::check(Path::new(&self.eval_string(args)?))?
                }
            }
            _ => return Err("unknown archive operation".into()),
        };
        self.vars.insert(name, value);
        Ok(())
    }

    fn secret_get(&mut self, text: &str) -> Result<(), String> {
        let Some(index) = find_capture_arrow(text) else {
            return Err("secret get syntax is secret get \"ENV_NAME\" -> $var".to_string());
        };
        let name = parse_string_literal(text[..index].trim())?;
        let destination = parse_capture_destination(&text[index + 2..], "secret get")?;
        let value = secret::get(&name)?;
        self.vars.insert(destination, Value::Scalar(value));
        Ok(())
    }

    fn run_line(&mut self, line: &str, line_no: usize) -> Result<(), String> {
        if let Some(rest) = line.strip_prefix("secret get ") {
            return self.secret_get(rest.trim());
        }
        if let Some(rest) = line.strip_prefix("arg ") {
            return self.capture_arg(rest.trim());
        }
        if let Some(rest) = line.strip_prefix("hasArg ") {
            return self.capture_has_arg(rest.trim());
        }
        if let Some(rest) = line.strip_prefix("args") {
            return self.capture_args(rest.trim());
        }
        if let Some(rest) = line.strip_prefix("import ") {
            return self.import_file(rest.trim(), line_no);
        }
        if let Some(rest) = line.strip_prefix("include ") {
            return self.include_file(rest.trim(), line_no);
        }
        if let Some(rest) = line.strip_prefix("set ") {
            return self.mutate_collection_command(rest.trim(), CollectionMutation::Set);
        }
        if let Some(rest) = line.strip_prefix("delete ") {
            return self.mutate_collection_command(rest.trim(), CollectionMutation::Delete);
        }
        if let Some(rest) = line.strip_prefix("append ") {
            let rest = rest.trim();
            if rest.contains(',') {
                return Err("append uses whitespace-separated arguments; comma-separated mutation syntax is unsupported".to_string());
            }
            if let Some((path_text, _)) = rest.split_once(char::is_whitespace) {
                let is_collection_variable = parse_identifier(path_text)
                    .map(|(kind, _, extra)| {
                        if kind != IdentKind::Variable || !extra.trim().is_empty() {
                            return false;
                        }
                        match self.eval_expr_value(path_text) {
                            Err(_) => true,
                            Ok(Value::Scalar(raw)) => !self.resolve_user_path(&raw).is_file(),
                            Ok(_) => false,
                        }
                    })
                    .unwrap_or(false);
                if !path_text.starts_with('"')
                    && (is_collection_variable
                        || path_text.contains('.')
                        || path_text.contains('[')
                        || matches!(self.eval_expr_value(path_text), Ok(Value::List(_))))
                {
                    return self.mutate_collection_command(rest, CollectionMutation::Append);
                }
            }
            self.log_action(&format!("append: {}", rest));
            return self.append_file_command(rest);
        }
        if let Some(rest) = line.strip_prefix("prepend ") {
            return self.push_list_item(rest.trim(), false);
        }
        if let Some(rest) = line.strip_prefix("archive ") {
            return self.archive_command(rest.trim());
        }
        if let Some(rest) = line.strip_prefix("create ") {
            self.log_action(&format!("create: {}", rest.trim()));
            return self.create_path(rest.trim());
        }
        if let Some(rest) = line.strip_prefix("copy ") {
            self.log_action(&format!("copy: {}", rest.trim()));
            return self.copy_path(rest.trim());
        }
        if let Some(rest) = line.strip_prefix("move ") {
            self.log_action(&format!("move: {}", rest.trim()));
            return self.move_path(rest.trim());
        }
        if let Some(rest) = line.strip_prefix("rename ") {
            self.log_action(&format!("rename: {}", rest.trim()));
            return self.rename_path(rest.trim());
        }
        if let Some(rest) = line.strip_prefix("read ") {
            self.log_action(&format!("read: {}", rest.trim()));
            return self.read_file_command(rest.trim());
        }
        if let Some(rest) = line.strip_prefix("isFile ") {
            return self.capture_path_kind(rest.trim(), true);
        }
        if let Some(rest) = line.strip_prefix("isDir ") {
            return self.capture_path_kind(rest.trim(), false);
        }
        if let Some(rest) = line.strip_prefix("tempFile") {
            return self.create_temp_path(rest.trim(), true);
        }
        if let Some(rest) = line.strip_prefix("tempDir") {
            return self.create_temp_path(rest.trim(), false);
        }
        if let Some(rest) = line.strip_prefix("size ") {
            return self.capture_file_size(rest.trim());
        }
        if let Some(rest) = line.strip_prefix("write ") {
            self.log_action(&format!("write: {}", rest.trim()));
            return self.write_file_command(rest.trim());
        }
        if let Some(rest) = line.strip_prefix("remove ") {
            let rest = rest.trim();
            if rest.trim_start().starts_with('"') {
                self.log_action(&format!("remove: {}", rest));
                return self.remove_path(rest);
            }
            if self.looks_like_dict_remove_target(rest) {
                return self.remove_dict_item(rest);
            }
            if looks_like_list_mutation_target(rest) {
                return self.remove_list_item(rest);
            }
            self.log_action(&format!("remove: {}", rest));
            return self.remove_path(rest);
        }
        if line == "break" {
            return Err(break_signal());
        }
        if line == "continue" {
            return Err(continue_signal());
        }
        if line == "return" {
            return Err("return value required".to_string());
        }
        if let Some(rest) = line.strip_prefix("return ") {
            return Err(return_signal(self.eval_expr(rest.trim())?));
        }
        if line == "exit" {
            return Err(exit_signal(0));
        }
        if let Some(rest) = line.strip_prefix("exit ") {
            let code = self.eval_exit_code(rest.trim())?;
            return Err(exit_signal(code));
        }
        if line.starts_with('$') {
            return self.assignment(line, line_no);
        }
        if let Some(rest) = line.strip_prefix("http ") {
            return self.run_http(rest.trim());
        }
        if let Some(rest) = line.strip_prefix("shellout") {
            let value = self.eval_expr(rest.trim())?;
            self.log_action(&format!("shellout: {value}"));
            println!("{value}");
            return Ok(());
        }
        if let Some(rest) = line.strip_prefix("shellin") {
            let name = parse_var_ref(rest.trim())?;
            if self.effect(audit::AuditAction::Input { kind: "stdin" })? {
                return Ok(());
            }
            let input = read_stdin_line()?;
            self.vars.insert(name, Value::Scalar(input));
            return Ok(());
        }
        if let Some(rest) = line.strip_prefix("ask") {
            if self.effect(audit::AuditAction::Input { kind: "stdin" })? {
                return Ok(());
            }
            let rest = rest.trim();
            if rest.contains(',') && !rest.contains("->") {
                return Err(
                    "old ask comma syntax was removed; use ask \"Question?\" -> $answer"
                        .to_string(),
                );
            }
            let Some((prompt_text, destination)) = rest.split_once("->") else {
                return Err("ask syntax is ask \"Question?\" -> $answer".to_string());
            };
            let prompt = self.eval_string(prompt_text.trim())?;
            let (kind, name, extra) = parse_identifier(destination.trim())
                .map_err(|_| "ask destination must be a variable".to_string())?;
            if kind != IdentKind::Variable || !extra.trim().is_empty() {
                return Err("ask destination must be a variable".to_string());
            }
            print!("{prompt}");
            io::stdout()
                .flush()
                .map_err(|error| format!("failed to flush stdout: {error}"))?;
            let input = read_stdin_line()?;
            self.vars.insert(name, Value::Scalar(input));
            return Ok(());
        }
        if let Some(rest) = line.strip_prefix("choose ") {
            let (question_text, options_text, destination) = parse_choose_capture(rest.trim())?;
            let question = self.eval_string(question_text)?;
            let Value::List(options) = self.eval_expr_value(options_text)? else {
                return Err("choose options must be a list".to_string());
            };
            if options.is_empty() {
                return Err("choose options must not be empty".to_string());
            }
            if self.effect(audit::AuditAction::Input { kind: "stdin" })? {
                return Err(
                    "dry-run unavailable-result: choose requires interactive input".to_string(),
                );
            }
            println!("{question}");
            for (index, option) in options.iter().enumerate() {
                println!("{}) {}", index + 1, format_value(option));
            }
            loop {
                print!("Choose: ");
                io::stdout()
                    .flush()
                    .map_err(|error| format!("failed to flush stdout: {error}"))?;
                let input = read_stdin_line()?;
                if let Ok(index) = input.parse::<usize>() {
                    if (1..=options.len()).contains(&index) {
                        self.vars.insert(destination, options[index - 1].clone());
                        return Ok(());
                    }
                }
                if let Some(option) = options.iter().find(|option| format_value(option) == input) {
                    self.vars.insert(destination, option.clone());
                    return Ok(());
                }
            }
        }
        if let Some(rest) = line.strip_prefix("exists ") {
            return self.capture_predicate(rest.trim(), |this, args| this.eval_exists(args));
        }
        if let Some(rest) = line.strip_prefix("matches ") {
            return self.capture_predicate(rest.trim(), |this, args| this.eval_matches(args));
        }
        if let Some((name, rest)) = split_command_name(line) {
            if matches!(
                name,
                "assertEqual" | "assertTrue" | "assertFalse" | "assertContains"
            ) {
                return self.run_assertion(name, rest.trim());
            }
        }
        if let Some(rest) = line.strip_prefix("contains ") {
            return self.capture_predicate(rest.trim(), |this, args| this.eval_contains(args));
        }
        if let Some((name, rest)) = split_command_name(line) {
            if is_standard_helper(name) {
                return self.run_helper(name, rest.trim());
            }
        }
        if let Some(rest) = line.strip_prefix("files ") {
            return self.collect_files(rest.trim());
        }
        if line == "files" {
            return Err("files syntax is files \"pattern\" -> $items".to_string());
        }
        if let Some(rest) = line.strip_prefix("command") {
            if rest.is_empty() || rest.starts_with(char::is_whitespace) {
                if rest.trim_start().starts_with('"') {
                    return Err("legacy command \"...\" syntax is not supported; use command <program> <args...> or shell \"...\"".to_string());
                }
            }
            let (command_text, timeout) = split_timeout_suffix(rest.trim())?;
            let (program, args, tail) = self.parse_command_atoms(command_text)?;
            if program.starts_with('"') {
                return Err("command requires a bare program followed by argv atoms; use shell \"...\" for shell syntax".to_string());
            }
            let tail = tail.trim();
            let destinations = if tail.is_empty() {
                None
            } else {
                let Some(destination_text) = tail.strip_prefix("->") else {
                    return Err(command_capture_syntax_error());
                };
                Some(parse_command_capture_destinations(destination_text.trim())?)
            };
            if self.context.is_dry_run() {
                audit::record(
                    &mut self.context,
                    audit::AuditAction::Process {
                        program: program.clone(),
                        argc: args.len(),
                    },
                )?;
                if let Some(names) = &destinations {
                    for n in names {
                        self.mark_unavailable(n.clone());
                    }
                }
                return Ok(());
            }
            let mock_key = format!(
                "command:{}",
                std::iter::once(program.clone())
                    .chain(args.iter().cloned())
                    .collect::<Vec<_>>()
                    .join("\u{1f}")
            );
            if env::var_os("SHELLY_TEST_MODE").is_some() {
                let Some(mock::Mock::Command {
                    code,
                    stdout,
                    stderr,
                    ..
                }) = self.mocks.find(&mock_key)
                else {
                    return Err(format!("no exact command mock for {} {:?}", program, args));
                };
                let code = *code;
                let stdout = stdout.clone();
                let stderr = stderr.clone();
                if destinations.is_none() {
                    if code != 0 {
                        return self.mock_process_error(&program, &args, code, stdout, stderr);
                    }
                    return Ok(());
                }
                let destinations = destinations.unwrap();
                if code != 0 {
                    return self.mock_process_error(&program, &args, code, stdout, stderr);
                }
                match destinations.as_slice() {
                    [name] => {
                        self.vars.insert(name.clone(), Value::Scalar(stdout));
                    }
                    [out, err, status] => {
                        self.vars.insert(out.clone(), Value::Scalar(stdout));
                        self.vars.insert(err.clone(), Value::Scalar(stderr));
                        self.vars
                            .insert(status.clone(), Value::Scalar(code.to_string()));
                    }
                    _ => unreachable!(),
                }
                return Ok(());
            }
            let mut process = Command::new(&program);
            process.args(&args);
            let deadline = timeout
                .map(|seconds| Instant::now() + seconds)
                .or(self.deadline);
            self.log_action(&format!("command: {} {:?}", program, args));
            if destinations.is_none() {
                let mut child = process
                    .spawn()
                    .map_err(|error| format!("failed to run command: {error}"))?;
                let status = poll_child(&mut child, deadline)
                    .map_err(|_| self.timeout_failure(&program, &args))?;
                if !status.success() {
                    let mut fields = BTreeMap::new();
                    fields.insert("kind".to_string(), Value::Scalar("process".to_string()));
                    fields.insert("program".to_string(), Value::Scalar(program.clone()));
                    fields.insert(
                        "args".to_string(),
                        Value::List(args.iter().cloned().map(Value::Scalar).collect()),
                    );
                    fields.insert(
                        "code".to_string(),
                        Value::Scalar(status.code().unwrap_or(1).to_string()),
                    );
                    let message = format!("command exited with status {status}");
                    fields.insert("message".to_string(), Value::Scalar(message.clone()));
                    self.pending_process_error = Some(fields);
                    return Err(message);
                }
                return Ok(());
            }
            let output = process
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|error| format!("failed to capture command: {error}"))?;
            let output =
                poll_output(output, deadline).map_err(|_| self.timeout_failure(&program, &args))?;
            let destinations = destinations.unwrap();
            match destinations.as_slice() {
                [stdout_name] => {
                    if !output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
                        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
                        let mut fields = BTreeMap::new();
                        fields.insert("kind".to_string(), Value::Scalar("process".to_string()));
                        fields.insert("program".to_string(), Value::Scalar(program.clone()));
                        fields.insert(
                            "args".to_string(),
                            Value::List(args.iter().cloned().map(Value::Scalar).collect()),
                        );
                        fields.insert(
                            "code".to_string(),
                            Value::Scalar(output.status.code().unwrap_or(1).to_string()),
                        );
                        fields.insert("stdout".to_string(), Value::Scalar(stdout));
                        fields.insert("stderr".to_string(), Value::Scalar(stderr));
                        let message = format!(
                            "command exited with status {}",
                            output.status.code().unwrap_or(1)
                        );
                        fields.insert("message".to_string(), Value::Scalar(message.clone()));
                        self.pending_process_error = Some(fields);
                        return Err(message);
                    }
                    let value = command_output_to_string(output.stdout, "output")?;
                    self.vars.insert(stdout_name.clone(), Value::Scalar(value));
                }
                [stdout_name, stderr_name, code_name] => {
                    let stdout = command_output_to_string(output.stdout, "stdout")?;
                    let stderr = command_output_to_string(output.stderr, "stderr")?;
                    let code = output.status.code().unwrap_or(1).to_string();
                    self.vars.insert(stdout_name.clone(), Value::Scalar(stdout));
                    self.vars.insert(stderr_name.clone(), Value::Scalar(stderr));
                    self.vars.insert(code_name.clone(), Value::Scalar(code));
                }
                _ => unreachable!("command destination count is validated"),
            }
            return Ok(());
        }
        if let Some(rest) = line.strip_prefix("shell ") {
            let (text, tail) = split_string_literal_tail(rest.trim())?;
            let command = self.eval_string(text)?;
            if command.contains("[REDACTED]") {
                return Err("secret values cannot be interpolated into shell source".to_string());
            }
            let tail = tail.trim();
            let destinations = if tail.is_empty() {
                None
            } else {
                Some(parse_command_capture_destinations(
                    tail.strip_prefix("->")
                        .ok_or_else(command_capture_syntax_error)?
                        .trim(),
                )?)
            };
            if self.context.is_dry_run() {
                audit::record(&mut self.context, audit::AuditAction::Shell)?;
                if let Some(names) = &destinations {
                    for n in names {
                        self.mark_unavailable(n.clone());
                    }
                }
                return Ok(());
            }
            if destinations.is_none() {
                let status = shell_command(&command)
                    .status()
                    .map_err(|e| format!("failed to run shell: {e}"))?;
                if !status.success() {
                    return Err(format!("shell exited with status {status}"));
                }
                return Ok(());
            }
            let output = shell_command(&command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| format!("failed to run shell: {e}"))?;
            if let Some(destinations) = destinations {
                if destinations.len() == 1 && !output.status.success() {
                    let mut fields = BTreeMap::new();
                    fields.insert("kind".to_string(), Value::Scalar("process".to_string()));
                    fields.insert("program".to_string(), Value::Scalar("sh".to_string()));
                    fields.insert(
                        "args".to_string(),
                        Value::List(vec![
                            Value::Scalar("-c".to_string()),
                            Value::Scalar(command.clone()),
                        ]),
                    );
                    fields.insert(
                        "code".to_string(),
                        Value::Scalar(output.status.code().unwrap_or(1).to_string()),
                    );
                    fields.insert(
                        "stdout".to_string(),
                        Value::Scalar(String::from_utf8_lossy(&output.stdout).into_owned()),
                    );
                    fields.insert(
                        "stderr".to_string(),
                        Value::Scalar(String::from_utf8_lossy(&output.stderr).into_owned()),
                    );
                    let message = format!("shell exited with status {}", output.status);
                    fields.insert("message".to_string(), Value::Scalar(message.clone()));
                    self.pending_process_error = Some(fields);
                    return Err(message);
                }
                self.capture_process_output(output, destinations)?;
            } else if !output.status.success() {
                let mut fields = BTreeMap::new();
                fields.insert("kind".to_string(), Value::Scalar("process".to_string()));
                fields.insert("program".to_string(), Value::Scalar("sh".to_string()));
                fields.insert(
                    "args".to_string(),
                    Value::List(vec![
                        Value::Scalar("-c".to_string()),
                        Value::Scalar(command.clone()),
                    ]),
                );
                fields.insert(
                    "code".to_string(),
                    Value::Scalar(output.status.code().unwrap_or(1).to_string()),
                );
                let message = format!("shell exited with status {}", output.status);
                fields.insert("message".to_string(), Value::Scalar(message.clone()));
                self.pending_process_error = Some(fields);
                return Err(message);
            }
            return Ok(());
        }
        if line.starts_with("capture") {
            return Err(
                "capture syntax was removed; use command <program> <args...> -> $var".to_string(),
            );
        }
        if let Some(rest) = line.strip_prefix("call ") {
            return self.call_function(rest.trim(), line_no);
        }
        if line == "call" {
            return Err("call requires a function name".to_string());
        }
        if let Some(name) = line.split_whitespace().next() {
            if self.functions.contains_key(name) {
                return Err("function calls must use call".to_string());
            }
        }
        Err(format!("unknown command or invalid syntax: {line}"))
    }

    fn mock_process_error(
        &mut self,
        program: &str,
        args: &[String],
        code: i32,
        stdout: String,
        stderr: String,
    ) -> Result<(), String> {
        let message = format!("command exited with status {code}");
        let mut fields = BTreeMap::new();
        fields.insert("kind".into(), Value::Scalar("process".into()));
        fields.insert("program".into(), Value::Scalar(program.into()));
        fields.insert(
            "args".into(),
            Value::List(args.iter().cloned().map(Value::Scalar).collect()),
        );
        fields.insert("code".into(), Value::Scalar(code.to_string()));
        fields.insert("stdout".into(), Value::Scalar(stdout));
        fields.insert("stderr".into(), Value::Scalar(stderr));
        fields.insert("message".into(), Value::Scalar(message.clone()));
        self.pending_process_error = Some(fields);
        Err(message)
    }

    fn run_http(&mut self, text: &str) -> Result<(), String> {
        let (method, args, tail) = self.parse_command_atoms(text)?;
        if !matches!(
            method.as_str(),
            "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS"
        ) {
            return Err(
                "http method must be GET, POST, PUT, PATCH, DELETE, HEAD, or OPTIONS".into(),
            );
        }
        let url = args.first().ok_or("http requires METHOD URL")?.clone();
        let mut headers = None;
        let mut body = None;
        let mut timeout = None;
        let mut max_bytes = None;
        let mut seen = HashSet::new();
        let mut i = 1;
        while i < args.len() {
            let option = args[i].as_str();
            if !seen.insert(option) {
                return Err(format!("http option repeated: {option}"));
            }
            match option {
                "headers" => {
                    i += 1;
                    headers = Some(
                        self.eval_expr_value(args.get(i).ok_or("headers requires a dictionary")?)?,
                    );
                }
                "body" => {
                    i += 1;
                    body = Some(self.eval_string(args.get(i).ok_or("body requires text")?)?);
                }
                "timeout" => {
                    i += 1;
                    timeout = Some(
                        self.eval_expr(args.get(i).ok_or("timeout requires a number")?)?
                            .parse()
                            .map_err(|_| "timeout must be a number")?,
                    );
                }
                "maxBytes" => {
                    i += 1;
                    max_bytes = Some(
                        self.eval_expr(args.get(i).ok_or("maxBytes requires an integer")?)?
                            .parse()
                            .map_err(|_| "maxBytes must be an integer")?,
                    );
                }
                other => return Err(format!("unexpected http option: {other}")),
            }
            i += 1;
        }
        let dest = tail
            .trim()
            .strip_prefix("->")
            .ok_or("http requires exactly one capture: -> $response")?;
        let names = parse_command_capture_destinations(dest.trim())?;
        if names.len() != 1 {
            return Err("http requires exactly one capture".into());
        }
        if self.context.is_dry_run() {
            audit::record(
                &mut self.context,
                audit::AuditAction::Http {
                    method: method.clone(),
                    url: url.clone(),
                },
            )?;
            self.mark_unavailable(names[0].clone());
            return Ok(());
        }
        let mock_key = format!("http:{method}:{url}");
        if env::var_os("SHELLY_TEST_MODE").is_some() {
            let Some(mock::Mock::Http { status, body, .. }) = self.mocks.find(&mock_key) else {
                return Err(format!("no exact HTTP mock for {method} {url}"));
            };
            let mut response = BTreeMap::new();
            response.insert("status".into(), Value::Scalar(status.to_string()));
            response.insert("body".into(), Value::Scalar(body.clone()));
            response.insert("url".into(), Value::Scalar(url));
            response.insert("headers".into(), Value::Dict(BTreeMap::new()));
            if *status >= 400 {
                return Err(format!("HTTP request returned a non-success status"));
            }
            self.vars.insert(names[0].clone(), Value::Dict(response));
            return Ok(());
        }
        match http::request(&method, &url, headers, body, timeout, max_bytes) {
            Ok(value) => {
                self.vars.insert(names[0].clone(), value);
                Ok(())
            }
            Err((message, fields)) => {
                self.pending_process_error = Some(fields);
                Err(message)
            }
        }
    }

    fn parse_command_atoms<'a>(
        &self,
        mut text: &'a str,
    ) -> Result<(String, Vec<String>, &'a str), String> {
        let mut atoms = Vec::new();
        while !text.trim_start().is_empty() && !text.trim_start().starts_with("->") {
            text = text.trim_start();
            if text.starts_with('"') {
                let (literal, rest) = split_string_literal_tail(text)?;
                atoms.push(self.eval_string(literal)?);
                text = rest;
            } else if text.starts_with('$') {
                let (kind, name, rest) = parse_identifier(text)?;
                let value = self.lookup_value(kind, &name)?;
                atoms.push(match value {
                    Value::Scalar(secret) if secret::redact(secret) != *secret => secret.clone(),
                    value => format_value(value),
                });
                text = rest;
            } else {
                let end = text.find(char::is_whitespace).unwrap_or(text.len());
                atoms.push(text[..end].to_string());
                text = &text[end..];
            }
        }
        if atoms.is_empty() {
            return Err("command requires a program and argv atoms".to_string());
        }
        let program = atoms.remove(0);
        Ok((program, atoms, text))
    }

    fn capture_process_output(
        &mut self,
        output: std::process::Output,
        destinations: Vec<String>,
    ) -> Result<(), String> {
        match destinations.as_slice() {
            [stdout_name] => {
                if !output.status.success() {
                    return Err(format!("shell exited with status {}", output.status));
                }
                self.vars.insert(
                    stdout_name.clone(),
                    Value::Scalar(command_output_to_string(output.stdout, "output")?),
                );
            }
            [stdout_name, stderr_name, code_name] => {
                self.vars.insert(
                    stdout_name.clone(),
                    Value::Scalar(command_output_to_string(output.stdout, "stdout")?),
                );
                self.vars.insert(
                    stderr_name.clone(),
                    Value::Scalar(command_output_to_string(output.stderr, "stderr")?),
                );
                self.vars.insert(
                    code_name.clone(),
                    Value::Scalar(output.status.code().unwrap_or(1).to_string()),
                );
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    fn capture_arg(&mut self, text: &str) -> Result<(), String> {
        let (index_text, destination) = parse_arg_capture(text)?;
        let index = self.eval_arg_index(index_text)?;
        let value = self.script_args.get(index - 1).cloned().unwrap_or_default();
        self.vars.insert(destination, Value::Scalar(value));
        Ok(())
    }

    fn capture_has_arg(&mut self, text: &str) -> Result<(), String> {
        let (value_text, destination) = parse_has_arg_capture(text)?;
        let needle = self.eval_value(value_text)?;
        let found = self.script_args.iter().any(|arg| arg == &needle);
        self.vars
            .insert(destination, Value::Scalar(found.to_string()));
        Ok(())
    }

    fn capture_args(&mut self, text: &str) -> Result<(), String> {
        let destination = parse_args_capture(text)?;
        self.vars.insert(
            destination,
            Value::List(
                self.script_args
                    .iter()
                    .cloned()
                    .map(Value::Scalar)
                    .collect(),
            ),
        );
        Ok(())
    }

    fn eval_arg_index(&self, expr: &str) -> Result<usize, String> {
        let expr = expr.trim();
        if expr.is_empty() {
            return Err("arg index must be a positive integer".to_string());
        }
        if expr.starts_with('"') || expr.starts_with('[') || matches!(expr, "true" | "false") {
            return Err("arg index must be a positive integer".to_string());
        }
        let mut parser = NumericParser::new(expr, self);
        let value = parser
            .parse_expr()
            .map_err(|_| "arg index must be a positive integer".to_string())?;
        parser.skip_ws();
        if !parser.is_at_end() {
            return Err("arg index must be a positive integer".to_string());
        }
        if value.fract() != 0.0 || value < 1.0 {
            return Err("arg index must be a positive integer".to_string());
        }
        Ok(value as usize)
    }

    fn resolve_user_path(&self, raw: &str) -> PathBuf {
        self.current_dir.join(raw)
    }

    fn eval_path_expr(&self, text: &str, error: &str) -> Result<String, String> {
        let text = text.trim();
        if text.is_empty() {
            return Err(error.to_string());
        }
        if text.starts_with('$') {
            return match self.eval_expr_value(text).map_err(|_| error.to_string())? {
                Value::Scalar(value) => Ok(value),
                Value::List(_) | Value::Dict(_) => Err(error.to_string()),
            };
        }
        self.eval_expr(text).map_err(|_| error.to_string())
    }

    fn create_path(&mut self, text: &str) -> Result<(), String> {
        let raw = self.eval_path_expr(text, "create requires a string path")?;
        let path = self.resolve_user_path(&raw);
        if self.context.is_dry_run() {
            audit::record(
                &mut self.context,
                audit::AuditAction::Path {
                    action: "create",
                    path: path.display().to_string(),
                },
            )?;
            return Ok(());
        }
        if raw.ends_with('/') {
            fs::create_dir_all(&path).map_err(|error| {
                format!("could not create directory '{}': {error}", path.display())
            })?;
            return Ok(());
        }
        if path.exists() {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "could not create parent directory '{}': {error}",
                    parent.display()
                )
            })?;
        }
        fs::File::create(&path)
            .map_err(|error| format!("could not create file '{}': {error}", path.display()))?;
        Ok(())
    }

    fn copy_path(&mut self, text: &str) -> Result<(), String> {
        let (from, to) = parse_two_string_paths(text, "copy")?;
        let from = self.resolve_user_path(&from);
        let to = self.resolve_user_path(&to);
        if self.context.is_dry_run() {
            audit::record(
                &mut self.context,
                audit::AuditAction::FromTo {
                    action: "copy",
                    from: from.display().to_string(),
                    to: to.display().to_string(),
                },
            )?;
            return Ok(());
        }
        if to.exists() {
            return Err(format!("copy destination already exists: {}", to.display()));
        }
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "could not create parent directory '{}': {error}",
                    parent.display()
                )
            })?;
        }
        copy_recursive(&from, &to)
    }

    fn move_path(&mut self, text: &str) -> Result<(), String> {
        let (from, to) = parse_two_string_paths(text, "move")?;
        let from = self.resolve_user_path(&from);
        let to = self.resolve_user_path(&to);
        if self.context.is_dry_run() {
            audit::record(
                &mut self.context,
                audit::AuditAction::FromTo {
                    action: "move",
                    from: from.display().to_string(),
                    to: to.display().to_string(),
                },
            )?;
            return Ok(());
        }
        if to.exists() {
            return Err(format!("move destination already exists: {}", to.display()));
        }
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "could not create parent directory '{}': {error}",
                    parent.display()
                )
            })?;
        }
        fs::rename(&from, &to)
            .or_else(|_| {
                copy_recursive(&from, &to)?;
                if from.is_dir() {
                    fs::remove_dir_all(&from)
                } else {
                    fs::remove_file(&from)
                }
                .map_err(|error| {
                    format!(
                        "could not remove moved source '{}': {error}",
                        from.display()
                    )
                })
            })
            .map_err(|error| {
                format!(
                    "could not move '{}' to '{}': {error}",
                    from.display(),
                    to.display()
                )
            })
    }

    fn rename_path(&mut self, text: &str) -> Result<(), String> {
        let (from_raw, name) = parse_two_string_paths(text, "rename")?;
        if name.contains('/')
            || name.contains('\\')
            || name == "."
            || name == ".."
            || name.is_empty()
        {
            return Err("rename new-name must be a plain name, not a path".to_string());
        }
        let from = self.resolve_user_path(&from_raw);
        let to = from.parent().unwrap_or_else(|| Path::new("")).join(name);
        if self.context.is_dry_run() {
            audit::record(
                &mut self.context,
                audit::AuditAction::FromTo {
                    action: "rename",
                    from: from.display().to_string(),
                    to: to.display().to_string(),
                },
            )?;
            return Ok(());
        }
        if to.exists() {
            return Err(format!(
                "rename destination already exists: {}",
                to.display()
            ));
        }
        fs::rename(&from, &to).map_err(|error| {
            format!(
                "could not rename '{}' to '{}': {error}",
                from.display(),
                to.display()
            )
        })
    }

    fn remove_path(&mut self, text: &str) -> Result<(), String> {
        let raw = self.eval_path_expr(text, "remove requires a string path")?;
        let path = self.resolve_user_path(&raw);
        if self.context.is_dry_run() {
            audit::record(
                &mut self.context,
                audit::AuditAction::Path {
                    action: "remove",
                    path: path.display().to_string(),
                },
            )?;
            return Ok(());
        }
        if !path.exists() {
            return Ok(());
        }
        if path.is_dir() {
            if !raw.ends_with('/') {
                return Err(
                    "remove directory paths must end with '/' to delete recursively".to_string(),
                );
            }
            fs::remove_dir_all(&path).map_err(|error| {
                format!("could not remove directory '{}': {error}", path.display())
            })?;
        } else {
            fs::remove_file(&path)
                .map_err(|error| format!("could not remove file '{}': {error}", path.display()))?;
        }
        Ok(())
    }

    fn read_file_command(&mut self, text: &str) -> Result<(), String> {
        let (file_text, destination) = parse_read_capture(text)?;
        let raw = self.eval_path_expr(file_text, "read requires a string file path")?;
        let mock_key = format!("read:{}", normalize_mock_path(&raw));
        if env::var_os("SHELLY_TEST_MODE").is_some() {
            let Some(mock::Mock::Read { content }) = self.mocks.find(&mock_key) else {
                return Err(format!(
                    "no exact read mock for '{}'; filesystem access is disabled",
                    raw
                ));
            };
            self.vars
                .insert(destination, Value::Scalar(content.clone()));
            return Ok(());
        }
        let path = self.resolve_user_path(&raw);
        if path.is_dir() {
            return Err(format!("read path is a directory: {}", path.display()));
        }
        let content = fs::read_to_string(&path)
            .map_err(|error| format!("could not read file '{}': {error}", path.display()))?;
        self.vars.insert(destination, Value::Scalar(content));
        Ok(())
    }

    fn append_file_command(&mut self, text: &str) -> Result<(), String> {
        let (file_text, value_text) = split_path_expr_tail(text, "append")?;
        if self.effect(audit::AuditAction::Input { kind: "filesystem" })? {
            return Ok(());
        }
        if value_text.trim().is_empty() {
            return Err("append syntax is append \"file\" $text".to_string());
        }
        let raw = self.eval_path_expr(file_text, "append requires a string file path")?;
        let path = self.resolve_user_path(&raw);
        if path.is_dir() {
            return Err(format!("append path is a directory: {}", path.display()));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "could not create parent directory '{}': {error}",
                    parent.display()
                )
            })?;
        }
        let value = self.eval_expr(value_text.trim())?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|error| format!("could not append file '{}': {error}", path.display()))?;
        writeln!(file, "{value}")
            .map_err(|error| format!("could not append file '{}': {error}", path.display()))
    }

    fn capture_path_kind(&mut self, text: &str, file: bool) -> Result<(), String> {
        let (path_text, destination) =
            parse_path_capture(text, if file { "isFile" } else { "isDir" })?;
        let raw = self.eval_expr(path_text.trim())?;
        let path = self.resolve_user_path(&raw);
        let ok = if file { path.is_file() } else { path.is_dir() };
        self.vars.insert(destination, Value::Scalar(ok.to_string()));
        Ok(())
    }

    fn create_temp_path(&mut self, text: &str, file: bool) -> Result<(), String> {
        let Some(destination) = text.strip_prefix("->") else {
            return Err(format!(
                "{} syntax is {} -> $path",
                if file { "tempFile" } else { "tempDir" },
                if file { "tempFile" } else { "tempDir" }
            ));
        };
        let (kind, name, extra) = parse_identifier(destination.trim())
            .map_err(|_| "temp destination must be a variable".to_string())?;
        if kind != IdentKind::Variable || !extra.trim().is_empty() {
            return Err("temp destination must be a variable".to_string());
        }
        if self.context.is_dry_run() {
            self.mark_unavailable(name.to_string());
            audit::record(
                &mut self.context,
                audit::AuditAction::Temp {
                    kind: if file { "file" } else { "dir" },
                    capture: name.to_string(),
                },
            )?;
            return Ok(());
        }
        let path = unique_temp_path(if file { "file" } else { "dir" });
        if file {
            fs::File::create(&path).map_err(|error| {
                format!("could not create temp file '{}': {error}", path.display())
            })?;
        } else {
            fs::create_dir(&path).map_err(|error| {
                format!(
                    "could not create temp directory '{}': {error}",
                    path.display()
                )
            })?;
        }
        self.vars
            .insert(name, Value::Scalar(path.display().to_string()));
        Ok(())
    }

    fn capture_file_size(&mut self, text: &str) -> Result<(), String> {
        let (file_text, destination) = parse_path_capture(text, "size")?;
        let raw = self.eval_expr(file_text.trim())?;
        let path = self.resolve_user_path(&raw);
        let metadata = fs::metadata(&path)
            .map_err(|error| format!("could not read file size '{}': {error}", path.display()))?;
        if !metadata.is_file() {
            return Err(format!("size path is not a file: {}", path.display()));
        }
        self.vars
            .insert(destination, Value::Scalar(metadata.len().to_string()));
        Ok(())
    }

    fn write_file_command(&mut self, text: &str) -> Result<(), String> {
        let (file_text, value_text) = split_path_expr_tail(text, "write")?;
        if value_text.trim().is_empty() {
            return Err("write syntax is write \"file\" $text".to_string());
        }
        let raw = self.eval_path_expr(file_text, "write requires a string file path")?;
        let path = self.resolve_user_path(&raw);
        if self.context.is_dry_run() {
            audit::record(
                &mut self.context,
                audit::AuditAction::Path {
                    action: "write",
                    path: path.display().to_string(),
                },
            )?;
            return Ok(());
        }
        if path.is_dir() {
            return Err(format!("write path is a directory: {}", path.display()));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "could not create parent directory '{}': {error}",
                    parent.display()
                )
            })?;
        }
        let value = self.eval_expr(value_text.trim())?;
        fs::write(&path, value)
            .map_err(|error| format!("could not write file '{}': {error}", path.display()))
    }

    fn define_function(&mut self, function: Function, line_no: usize) -> Result<(), String> {
        if self.functions.contains_key(&function.name) {
            return Err(format!(
                "line {line_no}: duplicate function {}",
                function.name
            ));
        }
        self.functions.insert(function.name.clone(), function);
        Ok(())
    }

    fn import_file(&mut self, path_text: &str, line_no: usize) -> Result<(), String> {
        let path = self.resolve_path(path_text)?;
        if self.stack.contains(&path) {
            return Err(format!(
                "line {line_no}: import/include cycle involving {}",
                path.display()
            ));
        }
        if !self.imported.insert(path.clone()) {
            return Ok(());
        }
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("could not read import '{}': {error}", path.display()))?;
        let source = normalize_multiline_strings(&source)?;
        let lines = strip_comments(&source)?;
        self.stack.push(path.clone());
        let previous_dir = std::mem::replace(
            &mut self.current_dir,
            path.parent().map(PathBuf::from).unwrap_or_default(),
        );
        let result = self.load_exports(&lines);
        self.current_dir = previous_dir;
        self.stack.pop();
        result
    }

    fn include_file(&mut self, path_text: &str, line_no: usize) -> Result<(), String> {
        let path = self.resolve_path(path_text)?;
        if self.stack.contains(&path) {
            return Err(format!(
                "line {line_no}: import/include cycle involving {}",
                path.display()
            ));
        }
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("could not read include '{}': {error}", path.display()))?;
        let source = normalize_multiline_strings(&source)?;
        let lines = strip_comments(&source)?;
        self.stack.push(path.clone());
        let previous_dir = std::mem::replace(
            &mut self.current_dir,
            path.parent().map(PathBuf::from).unwrap_or_default(),
        );
        let result = self.run_lines(&lines);
        self.current_dir = previous_dir;
        self.stack.pop();
        result
    }

    fn resolve_path(&self, path_text: &str) -> Result<PathBuf, String> {
        let raw = parse_string_literal(path_text)?;
        let path = self.current_dir.join(raw);
        path.canonicalize()
            .map_err(|error| format!("could not read file '{}': {error}", path.display()))
    }

    fn load_exports(&mut self, lines: &[(usize, String)]) -> Result<(), String> {
        let mut index = 0;
        while index < lines.len() {
            let line_no = lines[index].0;
            let line = lines[index].1.trim();
            if let Some(rest) = line.strip_prefix("import ") {
                self.import_file(rest.trim(), line_no)
                    .map_err(|error| line_error(line_no, error))?;
            } else if let Some(rest) = line.strip_prefix("include ") {
                self.include_file(rest.trim(), line_no)
                    .map_err(|error| line_error(line_no, error))?;
            } else if let Some(rest) = line.strip_prefix("export fn ") {
                let (function, next_index) = parse_function(rest.trim(), lines, index)
                    .map_err(|error| line_error(line_no, error))?;
                if !self.functions.contains_key(&function.name) {
                    self.functions.insert(function.name.clone(), function);
                }
                index = next_index;
                continue;
            }
            if let Some(rest) = line.strip_prefix("export ") {
                let (kind, _name, _) = parse_identifier(rest)?;
                if kind != IdentKind::Constant {
                    return Err(format!(
                        "line {line_no}: only constants and functions can be exported"
                    ));
                }
                self.assignment(rest, line_no)
                    .map_err(|error| line_error(line_no, error))?;
            }
            index += 1;
        }
        Ok(())
    }

    fn call_function(&mut self, line: &str, _line_no: usize) -> Result<(), String> {
        let (call_text, destination) = match line.split_once("->") {
            Some((call_text, destination)) => {
                let (kind, name, extra) = parse_identifier(destination.trim())
                    .map_err(|_| "function return destination must be a variable".to_string())?;
                if kind != IdentKind::Variable || !extra.trim().is_empty() {
                    return Err("function return destination must be a variable".to_string());
                }
                (call_text.trim(), Some(name))
            }
            None => (line.trim(), None),
        };
        let (name, arg_texts) = parse_call_syntax(call_text)?;
        let function = self
            .functions
            .get(name)
            .cloned()
            .ok_or_else(|| format!("unknown command or function: {name}"))?;
        let required = function
            .params
            .iter()
            .filter(|p| p.default.is_none())
            .count();
        if arg_texts.len() < required || arg_texts.len() > function.params.len() {
            return Err(format!("wrong argument count for {name}"));
        }
        let mut child = Interpreter::default();
        child.consts = HashMap::new();
        child.functions = self.functions.clone();
        child.current_dir = self.current_dir.clone();
        child.stack = self.stack.clone();
        child.imported = self.imported.clone();
        child.deadline = self.deadline;
        for (idx, param) in function.params.iter().enumerate() {
            let value = if let Some(text) = arg_texts.get(idx) {
                self.eval_expr(text)?
            } else {
                param.default.clone().expect("default exists")
            };
            child.vars.insert(param.name.clone(), Value::Scalar(value));
        }
        let mut executable_body = Vec::new();
        for (line_no, body_line) in function.body {
            if let Some(rest) = body_line.strip_prefix("global") {
                child
                    .declare_globals(rest.trim(), self)
                    .map_err(|error| line_error(line_no, error))?;
                continue;
            }
            executable_body.push((line_no, body_line));
        }
        match child.run_lines(&executable_body) {
            Ok(()) => {
                if destination.is_some() {
                    return Err(format!("function {name} did not return a value"));
                }
                Ok(())
            }
            Err(error) if let Some(value) = parse_return_signal(&error) => {
                if let Some(destination) = destination {
                    self.vars.insert(destination, Value::Scalar(value));
                }
                Ok(())
            }
            Err(error) if is_break_signal(&error) => Err("break outside loop".to_string()),
            Err(error) if is_continue_signal(&error) => Err("continue outside loop".to_string()),
            Err(error) => Err(error),
        }
    }

    fn declare_globals(&mut self, rest: &str, parent: &Interpreter) -> Result<(), String> {
        if rest.is_empty() {
            return Err("global requires at least one identifier".to_string());
        }
        for item in split_commas(rest)? {
            let (kind, name, extra) = parse_identifier(item.trim())?;
            if !extra.trim().is_empty() {
                return Err("unexpected text after identifier".to_string());
            }
            match kind {
                IdentKind::Variable => {
                    let value = parent
                        .vars
                        .get(&name)
                        .cloned()
                        .ok_or_else(|| format!("undefined global variable ${name}"))?;
                    self.vars.insert(name, value);
                }
                IdentKind::Constant => {
                    let value = parent
                        .consts
                        .get(&name)
                        .cloned()
                        .ok_or_else(|| format!("undefined global constant ${name}"))?;
                    self.consts.insert(name, value);
                }
            }
        }
        Ok(())
    }

    fn assignment(&mut self, line: &str, _line_no: usize) -> Result<(), String> {
        let (kind, name, rest) = parse_identifier(line)?;
        let rest = rest.trim_start();
        if rest.starts_with('.') {
            return self.replace_dict_item(kind, &name, rest);
        }
        if rest.starts_with('[') {
            if matches!(self.lookup_value(kind, &name), Ok(Value::Dict(_))) {
                return self.replace_dict_item(kind, &name, rest);
            }
            return self.replace_list_item(kind, &name, rest);
        }
        if rest.is_empty() {
            match kind {
                IdentKind::Variable => {
                    self.vars.insert(name, Value::Scalar(String::new()));
                    return Ok(());
                }
                IdentKind::Constant => {
                    return Err("constant must have an initial value".to_string());
                }
            }
        }
        let rest = rest
            .strip_prefix('=')
            .ok_or_else(|| "unexpected text after identifier".to_string())?
            .trim_start();
        if rest.is_empty() {
            return Err("expected value after '='".to_string());
        }
        if kind == IdentKind::Constant && self.consts.contains_key(&name) {
            return Err(format!("constant ${name} cannot be reassigned"));
        }
        if rest.starts_with("capture") {
            return Err("capture syntax was removed; use command \"...\" -> $var".to_string());
        }
        let value = self.eval_value_kind(rest)?;
        match kind {
            IdentKind::Variable => {
                self.unavailable.remove(&name);
                self.vars.insert(name, value);
            }
            IdentKind::Constant => {
                self.consts.insert(name, value);
            }
        }
        Ok(())
    }

    fn mutate_collection_command(
        &mut self,
        text: &str,
        mutation: CollectionMutation,
    ) -> Result<(), String> {
        if text.contains(',') {
            return Err(format!(
                "{} uses whitespace-separated arguments; comma-separated mutation syntax is unsupported",
                mutation.name()
            ));
        }
        let (path_text, value_text) = if mutation == CollectionMutation::Delete {
            if text.split_whitespace().count() != 1 {
                return Err("delete requires exactly one argument".to_string());
            }
            (text, None)
        } else {
            let (path, value) = text
                .trim()
                .split_once(char::is_whitespace)
                .ok_or_else(|| format!("{} requires exactly two arguments", mutation.name()))?;
            if value.trim().is_empty() {
                return Err(format!(
                    "{} requires exactly two arguments",
                    mutation.name()
                ));
            }
            (path, Some(value))
        };
        if mutation == CollectionMutation::Append
            && parse_identifier(path_text.trim())
                .map(|(kind, _, extra)| kind == IdentKind::Variable && extra.trim().is_empty())
                .unwrap_or(false)
        {
            let (_, name, _) = parse_identifier(path_text.trim()).unwrap();
            let value = self.eval_value_kind(value_text.unwrap().trim())?;
            if matches!(value, Value::List(_)) {
                return Err("nested lists are not supported yet".to_string());
            }
            let target = self
                .vars
                .get_mut(&name)
                .ok_or_else(|| format!("undefined variable ${name}"))?;
            return match target {
                Value::List(values) => {
                    values.push(value);
                    Ok(())
                }
                _ => Err("mutation target must be a list".to_string()),
            };
        }
        let (kind, name, segments) = parse_collection_path(path_text.trim())?;
        if kind != IdentKind::Variable {
            return Err("mutation target must be a variable collection".to_string());
        }
        let value = value_text
            .map(|text| self.eval_value_kind(text.trim()))
            .transpose()?;
        let target = self
            .vars
            .get_mut(&name)
            .ok_or_else(|| format!("undefined variable ${name}"))?;
        apply_collection_mutation(target, &segments, mutation, value, &format!("${name}"))
    }

    fn push_list_item(&mut self, rest: &str, append: bool) -> Result<(), String> {
        let verb = if append { "append" } else { "prepend" };
        if rest.contains(',') {
            return Err(format!(
                "{verb} uses whitespace-separated arguments; comma-separated mutation syntax is unsupported"
            ));
        }
        let (path_text, value_text) = rest
            .split_once(char::is_whitespace)
            .ok_or_else(|| format!("{verb} requires exactly two arguments"))?;
        let (kind, name, extra) = parse_identifier(path_text)
            .map_err(|_| "mutation target must be a variable list".to_string())?;
        if kind != IdentKind::Variable || !extra.trim().is_empty() {
            return Err("mutation target must be a variable list".to_string());
        }
        let value = match self.eval_value_kind(value_text.trim())? {
            Value::Scalar(value) => Value::Scalar(value),
            Value::List(_) | Value::Dict(_) => {
                return Err("nested lists are not supported yet".to_string());
            }
        };
        let target = self
            .vars
            .get_mut(&name)
            .ok_or_else(|| format!("undefined variable ${name}"))?;
        match target {
            Value::List(values) => {
                if append {
                    values.push(value);
                } else {
                    values.insert(0, value);
                }
                Ok(())
            }
            Value::Scalar(_) | Value::Dict(_) => Err("mutation target must be a list".to_string()),
        }
    }

    fn remove_list_item(&mut self, target: &str) -> Result<(), String> {
        let (kind, name, selector) = self.parse_mutation_target(target)?;
        if kind != IdentKind::Variable {
            return Err("mutation target must be a variable list".to_string());
        }
        let len = match self.vars.get(&name) {
            Some(Value::List(values)) => values.len(),
            Some(Value::Scalar(_)) | Some(Value::Dict(_)) => {
                return Err("mutation target must be a list".to_string());
            }
            None => return Err(format!("undefined variable ${name}")),
        };
        let index = self.parse_mutation_index(len, selector.trim(), &format!("${name}"))?;
        let Value::List(values) = self.vars.get_mut(&name).expect("checked list") else {
            unreachable!();
        };
        values.remove(index - 1);
        Ok(())
    }

    fn replace_list_item(&mut self, kind: IdentKind, name: &str, rest: &str) -> Result<(), String> {
        if kind != IdentKind::Variable {
            return Err("mutation target must be a variable list".to_string());
        }
        let close = rest
            .find(']')
            .ok_or_else(|| "malformed list index syntax".to_string())?;
        let selector = rest[..=close].trim();
        let value_text = rest[close + 1..]
            .trim_start()
            .strip_prefix('=')
            .ok_or_else(|| "unexpected text after identifier".to_string())?;
        if !selector.starts_with('[') || !selector.ends_with(']') {
            return Err("malformed list index syntax".to_string());
        }
        let selector = &selector[1..selector.len() - 1];
        let len = match self.vars.get(name) {
            Some(Value::List(values)) => values.len(),
            Some(Value::Scalar(_)) | Some(Value::Dict(_)) => {
                return Err("mutation target must be a list".to_string());
            }
            None => return Err(format!("undefined variable ${name}")),
        };
        let index = self.parse_mutation_index(len, selector.trim(), &format!("${name}"))?;
        let value = match self.eval_value_kind(value_text.trim())? {
            Value::Scalar(value) => Value::Scalar(value),
            Value::List(_) | Value::Dict(_) => {
                return Err("nested lists are not supported yet".to_string());
            }
        };
        let Value::List(values) = self.vars.get_mut(name).expect("checked list") else {
            unreachable!();
        };
        values[index - 1] = value;
        Ok(())
    }

    fn replace_dict_item(&mut self, kind: IdentKind, name: &str, rest: &str) -> Result<(), String> {
        if kind != IdentKind::Variable {
            return Err("mutation target must be a variable dictionary".to_string());
        }
        let (path_text, value_text) = rest
            .rsplit_once('=')
            .ok_or_else(|| "unexpected text after identifier".to_string())?;
        let keys = self.resolve_dict_path(path_text.trim())?;
        let value = self.eval_value_kind(value_text.trim())?;
        let target = self
            .vars
            .get_mut(name)
            .ok_or_else(|| format!("undefined variable ${name}"))?;
        replace_nested_dict_value(target, &keys, value, &format!("${name}"))
    }

    fn remove_dict_item(&mut self, target: &str) -> Result<(), String> {
        let (kind, name, rest) = parse_identifier(target)?;
        if kind != IdentKind::Variable {
            return Err("mutation target must be a variable dictionary".to_string());
        }
        let keys = self.resolve_dict_path(rest.trim())?;
        let target = self
            .vars
            .get_mut(&name)
            .ok_or_else(|| format!("undefined variable ${name}"))?;
        remove_nested_dict_value(target, &keys, &format!("${name}"))
    }

    fn looks_like_dict_remove_target(&self, target: &str) -> bool {
        let Ok((kind, name, rest)) = parse_identifier(target) else {
            return false;
        };
        if kind != IdentKind::Variable || rest.trim().is_empty() {
            return false;
        }
        rest.trim().starts_with('.') || matches!(self.lookup_value(kind, &name), Ok(Value::Dict(_)))
    }

    fn parse_mutation_target<'a>(
        &self,
        target: &'a str,
    ) -> Result<(IdentKind, String, &'a str), String> {
        let (kind, name, rest) = parse_identifier(target)?;
        let rest = rest.trim();
        if !rest.starts_with('[') || !rest.ends_with(']') {
            return Err("malformed list index syntax".to_string());
        }
        Ok((kind, name, &rest[1..rest.len() - 1]))
    }

    fn parse_mutation_index(
        &self,
        len: usize,
        selector: &str,
        target: &str,
    ) -> Result<usize, String> {
        if selector.is_empty() {
            return Err("malformed list index syntax".to_string());
        }
        if selector.contains(',') || selector.contains("..") || parse_filter(selector).is_some() {
            return Err("mutation index must select exactly one item".to_string());
        }
        if len == 0 && matches!(selector, "first" | "last") {
            return Err("list is empty".to_string());
        }
        let index = self.parse_list_position(len, selector)?;
        if index == 0 || index > len {
            return Err(format!(
                "list index {index} out of range for {target} (length {len})"
            ));
        }
        Ok(index)
    }

    fn capture_predicate<F>(&mut self, text: &str, eval: F) -> Result<(), String>
    where
        F: FnOnce(&Self, &str) -> Result<bool, String>,
    {
        let Some((args, destination)) = split_capture_arrow(text) else {
            return Err("predicate capture syntax is exists/contains ... -> $var".to_string());
        };
        let (kind, name, extra) = parse_identifier(destination.trim())
            .map_err(|_| "predicate capture destination must be a variable".to_string())?;
        if kind != IdentKind::Variable || !extra.trim().is_empty() {
            return Err("predicate capture destination must be a variable".to_string());
        }
        let value = eval(self, args.trim())?;
        self.vars.insert(name, Value::Scalar(value.to_string()));
        Ok(())
    }

    fn eval_exists(&self, args: &str) -> Result<bool, String> {
        let raw =
            parse_string_literal(args).map_err(|_| "exists requires a string path".to_string())?;
        Ok(self.current_dir.join(raw).exists())
    }

    fn run_assertion(&self, name: &str, text: &str) -> Result<(), String> {
        let args = split_assertion_args(text)?;
        let expected = assertion_arity(name);
        if args.len() != expected {
            return Err(format!(
                "{name} expects {expected} argument{}",
                if expected == 1 { "" } else { "s" }
            ));
        }
        let values: Result<Vec<_>, _> = args.iter().map(|arg| self.eval_expr_value(arg)).collect();
        let values = values?;
        let failed = match name {
            "assertEqual" => {
                compare_values(&format_value(&values[0]), &format_value(&values[1]))
                    != std::cmp::Ordering::Equal
            }
            "assertTrue" => !is_truthy(&format_value(&values[0])),
            "assertFalse" => is_truthy(&format_value(&values[0])),
            "assertContains" => !self.eval_contains(&format!("{} {}", args[0], args[1]))?,
            _ => unreachable!(),
        };
        if !failed {
            return Ok(());
        }
        let message = match name {
            "assertEqual" => format!(
                "{name} failed: expected {}, actual {}",
                format_value(&values[1]),
                format_value(&values[0])
            ),
            "assertTrue" => format!(
                "{name} failed: expected true, actual {}",
                format_value(&values[0])
            ),
            "assertFalse" => format!(
                "{name} failed: expected false, actual {}",
                format_value(&values[0])
            ),
            "assertContains" => format!(
                "{name} failed: expected {} to contain {}",
                format_value(&values[0]),
                format_value(&values[1])
            ),
            _ => unreachable!(),
        };
        Err(message)
    }

    fn eval_matches(&self, args: &str) -> Result<bool, String> {
        let (value_text, pattern) = split_matches_args(args)?;
        let regex = regex::Regex::new(&pattern)
            .map_err(|error| format!("invalid regex /{pattern}/: {error}"))?;
        match self.eval_expr_value(value_text)? {
            Value::Scalar(value) => Ok(regex.is_match(&value)),
            Value::List(values) => Ok(values
                .iter()
                .any(|value| regex.is_match(&format_value(value)))),
            Value::Dict(_) => Err("matches does not support dictionary values".to_string()),
        }
    }

    fn eval_contains(&self, args: &str) -> Result<bool, String> {
        let (value_text, search_text) = split_predicate_two_args(args)
            .ok_or_else(|| "contains requires a value and search value".to_string())?;
        let search = self.eval_value(search_text)?;
        match self.eval_expr_value(value_text)? {
            Value::Scalar(value) => Ok(value.contains(&search)),
            Value::List(values) => Ok(values.iter().any(|value| format_value(value) == search)),
            Value::Dict(values) => Ok(values.contains_key(&search)),
        }
    }

    fn run_helper(&mut self, name: &str, text: &str) -> Result<(), String> {
        let (args_text, destination) = parse_helper_capture(name, text)?;
        let args = split_space_args(args_text)?;
        let value = match name {
            "trim" => Value::Scalar(
                expect_arg_count(name, &args, 1)
                    .and_then(|_| self.eval_value(args[0]))?
                    .trim()
                    .to_string(),
            ),
            "lower" => Value::Scalar(
                expect_arg_count(name, &args, 1)
                    .and_then(|_| self.eval_value(args[0]))?
                    .to_lowercase(),
            ),
            "upper" => Value::Scalar(
                expect_arg_count(name, &args, 1)
                    .and_then(|_| self.eval_value(args[0]))?
                    .to_uppercase(),
            ),
            "replace" => {
                expect_arg_count(name, &args, 3)?;
                let text = self.eval_value(args[0])?;
                let old = self.eval_value(args[1])?;
                let new = self.eval_value(args[2])?;
                Value::Scalar(text.replace(&old, &new))
            }
            "substring" => {
                expect_arg_count(name, &args, 3)?;
                let text = self.eval_value(args[0])?;
                let start = parse_positive_integer_arg(
                    &self.eval_value(args[1])?,
                    "substring start must be a positive integer",
                )?;
                let length = parse_positive_integer_arg(
                    &self.eval_value(args[2])?,
                    "substring length must be a positive integer",
                )?;
                let chars: Vec<char> = text.chars().collect();
                let begin = start - 1;
                if begin >= chars.len() {
                    Value::Scalar(String::new())
                } else {
                    Value::Scalar(
                        chars[begin..(begin + length).min(chars.len())]
                            .iter()
                            .collect(),
                    )
                }
            }
            "padLeft" | "padRight" => {
                expect_arg_count(name, &args, 3)?;
                let text = self.eval_value(args[0])?;
                let width = parse_nonnegative_integer_arg(
                    &self.eval_value(args[1])?,
                    "padding width must be a non-negative integer",
                )?;
                let fill = self.eval_value(args[2])?;
                let mut chars = fill.chars();
                let fill_char = chars
                    .next()
                    .ok_or_else(|| format!("{name} fill must be exactly one Unicode character"))?;
                if chars.next().is_some() {
                    return Err(format!("{name} fill must be exactly one Unicode character"));
                }
                let needed = width.saturating_sub(text.chars().count());
                let padding: String = std::iter::repeat_n(fill_char, needed).collect();
                Value::Scalar(if name == "padLeft" {
                    format!("{padding}{text}")
                } else {
                    format!("{text}{padding}")
                })
            }
            "formatDate" => {
                expect_arg_count(name, &args, 2)?;
                let timestamp = self.eval_value(args[0])?;
                let pattern = self.eval_value(args[1])?;
                Value::Scalar(format_date_helper(&timestamp, &pattern)?)
            }
            "split" => {
                expect_arg_count(name, &args, 2)?;
                let text = self.eval_value(args[0])?;
                let sep = self.eval_value(args[1])?;
                if sep.is_empty() {
                    return Err("split separator cannot be empty".to_string());
                }
                let items = if text.is_empty() {
                    Vec::new()
                } else {
                    text.split(&sep)
                        .map(|item| Value::Scalar(item.to_string()))
                        .collect()
                };
                Value::List(items)
            }
            "startsWith" | "endsWith" => {
                expect_arg_count(name, &args, 2)?;
                let text = self.eval_value(args[0])?;
                let needle = self.eval_value(args[1])?;
                let ok = if name == "startsWith" {
                    text.starts_with(&needle)
                } else {
                    text.ends_with(&needle)
                };
                Value::Scalar(ok.to_string())
            }
            "len" => {
                expect_arg_count(name, &args, 1)?;
                let len = match self.eval_expr_value(args[0])? {
                    Value::Scalar(v) => v.chars().count(),
                    Value::List(v) => v.len(),
                    Value::Dict(v) => v.len(),
                };
                Value::Scalar(len.to_string())
            }
            "join" => {
                expect_arg_count(name, &args, 2)?;
                let Value::List(items) = self.eval_expr_value(args[0])? else {
                    return Err("join requires a list".to_string());
                };
                let items = scalar_list(name, &items)?;
                Value::Scalar(items.join(&self.eval_value(args[1])?))
            }
            "sort" | "reverse" | "unique" => {
                expect_arg_count(name, &args, 1)?;
                let Value::List(items) = self.eval_expr_value(args[0])? else {
                    return Err(format!("{name} requires a list"));
                };
                let mut scalars = scalar_list(name, &items)?;
                match name {
                    "sort" => scalars.sort(),
                    "reverse" => scalars.reverse(),
                    "unique" => {
                        let mut seen = HashSet::new();
                        scalars.retain(|item| seen.insert(item.clone()));
                    }
                    _ => {}
                }
                Value::List(scalars.into_iter().map(Value::Scalar).collect())
            }
            "toNumber" => {
                expect_arg_count(name, &args, 1)?;
                let raw = self.eval_value(args[0])?;
                let n: f64 = raw
                    .parse()
                    .map_err(|_| "toNumber requires numeric input".to_string())?;
                Value::Scalar(format_number(n))
            }
            "toString" => {
                expect_arg_count(name, &args, 1)?;
                Value::Scalar(self.eval_expr(args[0])?)
            }
            "isEmpty" => {
                expect_arg_count(name, &args, 1)?;
                Value::Scalar(
                    match self.eval_expr_value(args[0])? {
                        Value::Scalar(v) => v.is_empty(),
                        Value::List(v) => v.is_empty(),
                        Value::Dict(v) => v.is_empty(),
                    }
                    .to_string(),
                )
            }
            "isNumber" | "isText" | "isList" | "isDict" => {
                expect_arg_count(name, &args, 1)?;
                let value = self.eval_expr_value(args[0])?;
                let ok = match (name, value) {
                    ("isList", Value::List(_)) => true,
                    ("isDict", Value::Dict(_)) => true,
                    ("isNumber", Value::Scalar(v)) => v.parse::<f64>().is_ok(),
                    ("isText", Value::Scalar(v)) => {
                        v.parse::<f64>().is_err() && v != "true" && v != "false"
                    }
                    _ => false,
                };
                Value::Scalar(ok.to_string())
            }
            "keys" => {
                expect_arg_count(name, &args, 1)?;
                let Value::Dict(values) = self.eval_expr_value(args[0])? else {
                    return Err("keys requires a dictionary".to_string());
                };
                Value::List(values.keys().cloned().map(Value::Scalar).collect())
            }
            "values" => {
                expect_arg_count(name, &args, 1)?;
                let Value::Dict(values) = self.eval_expr_value(args[0])? else {
                    return Err("values requires a dictionary".to_string());
                };
                Value::List(values.values().cloned().collect())
            }
            "jsonParse" => {
                expect_arg_count(name, &args, 1)?;
                let raw = self.eval_value(args[0])?;
                json_to_value(
                    serde_json::from_str(&raw)
                        .map_err(|error| format!("jsonParse failed: {error}"))?,
                )
            }
            "jsonStringify" => {
                expect_arg_count(name, &args, 1)?;
                Value::Scalar(value_to_json(&self.eval_expr_value(args[0])?).to_string())
            }
            "pathJoin" => {
                expect_min_arg_count(name, &args, 2)?;
                let parts = args
                    .iter()
                    .map(|arg| self.eval_value(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                Value::Scalar(path_join(&parts))
            }
            "basename" => {
                if args.len() != 1 && args.len() != 2 {
                    return Err("basename expects 1 or 2 argument(s)".to_string());
                }
                let raw = self.eval_value(args[0])?;
                let extension = if args.len() == 2 {
                    Some(self.eval_value(args[1])?)
                } else {
                    None
                };
                Value::Scalar(basename_helper(&raw, extension.as_deref()))
            }
            "dirname" | "extension" | "withoutExtension" => {
                expect_arg_count(name, &args, 1)?;
                let raw = self.eval_value(args[0])?;
                Value::Scalar(path_helper(name, &raw))
            }
            "now" | "timestamp" => {
                expect_arg_count(name, &args, 0)?;
                Value::Scalar(current_utc_timestamp())
            }
            "today" => {
                expect_arg_count(name, &args, 0)?;
                Value::Scalar(current_utc_date())
            }
            "commandExists" => {
                expect_arg_count(name, &args, 1)?;
                let command = self.eval_value(args[0])?;
                Value::Scalar(find_command(&command).is_some().to_string())
            }
            "which" => {
                expect_arg_count(name, &args, 1)?;
                let command = self.eval_value(args[0])?;
                Value::Scalar(
                    find_command(&command)
                        .map(|path| path.to_string_lossy().to_string())
                        .unwrap_or_default(),
                )
            }
            "random" => {
                expect_arg_count(name, &args, 0)?;
                Value::Scalar(random_u64().to_string())
            }
            "randomInt" => {
                expect_arg_count(name, &args, 2)?;
                let min = parse_integer_arg(
                    &self.eval_value(args[0])?,
                    "randomInt min must be an integer",
                )?;
                let max = parse_integer_arg(
                    &self.eval_value(args[1])?,
                    "randomInt max must be an integer",
                )?;
                if min > max {
                    return Err("randomInt min cannot be greater than max".to_string());
                }
                let span = (max - min + 1) as u64;
                Value::Scalar((min + (random_u64() % span) as i64).to_string())
            }
            _ => unreachable!(),
        };
        self.vars.insert(destination, value);
        Ok(())
    }

    fn collect_files(&mut self, text: &str) -> Result<(), String> {
        let (pattern_text, destination) = parse_files_capture(text)?;
        let pattern = self.eval_string(pattern_text)?;
        let matches = glob_files(&self.current_dir, &pattern)?;
        self.vars.insert(
            destination,
            Value::List(matches.into_iter().map(Value::Scalar).collect()),
        );
        Ok(())
    }

    fn eval_condition(&self, condition: &str) -> Result<bool, String> {
        let condition = condition.trim();
        if condition.is_empty() {
            return Err("condition required".to_string());
        }
        validate_condition_parentheses(condition)?;
        self.eval_or_condition(condition)
    }

    fn eval_or_condition(&self, condition: &str) -> Result<bool, String> {
        if let Some((left, right)) = split_condition_keyword(condition, "or") {
            return Ok(self.eval_or_condition(left)? || self.eval_and_condition(right)?);
        }
        self.eval_and_condition(condition)
    }

    fn eval_and_condition(&self, condition: &str) -> Result<bool, String> {
        if let Some((left, right)) = split_condition_keyword(condition, "and") {
            return Ok(self.eval_and_condition(left)? && self.eval_not_condition(right)?);
        }
        self.eval_not_condition(condition)
    }

    fn eval_not_condition(&self, condition: &str) -> Result<bool, String> {
        let condition = condition.trim();
        if let Some(rest) = condition.strip_prefix("not ") {
            return Ok(!self.eval_not_condition(rest)?);
        }
        self.eval_simple_condition(condition)
    }

    fn eval_simple_condition(&self, condition: &str) -> Result<bool, String> {
        let condition = condition.trim();
        if let Some(inner) = strip_grouped_condition(condition)? {
            return self.eval_condition(inner);
        }
        if condition.starts_with(')') {
            return Err("unexpected ')' in condition".to_string());
        }
        if condition.starts_with('(') {
            return Err("missing ')' in condition".to_string());
        }
        if let Some(args) = condition.strip_prefix("exists ") {
            return self.eval_exists(args.trim());
        }
        if condition == "exists" {
            return Err("exists requires a string path".to_string());
        }
        if let Some(args) = condition.strip_prefix("contains ") {
            return self.eval_contains(args.trim());
        }
        if let Some(args) = condition.strip_prefix("matches ") {
            return self.eval_matches(args.trim());
        }
        if condition == "contains" {
            return Err("contains requires a value and search value".to_string());
        }
        if let Some((left, operator, right)) = split_condition_operator(condition) {
            let left = self.eval_expr(left.trim())?;
            let right = self.eval_expr(right.trim())?;
            return match operator {
                "==" => Ok(left == right),
                "!=" => Ok(left != right),
                ">" => Ok(compare_values(&left, &right).is_gt()),
                ">=" => Ok(!compare_values(&left, &right).is_lt()),
                "<" => Ok(compare_values(&left, &right).is_lt()),
                "<=" => Ok(!compare_values(&left, &right).is_gt()),
                _ => unreachable!(),
            };
        }
        match self.eval_expr_value(condition)? {
            Value::Scalar(value) => Ok(is_truthy(&value)),
            Value::List(values) => Ok(!values.is_empty()),
            Value::Dict(values) => Ok(!values.is_empty()),
        }
    }

    fn eval_expr_value(&self, expr: &str) -> Result<Value, String> {
        let expr = expr.trim();
        if expr.starts_with('$') {
            let (kind, name, rest) = parse_identifier(expr)?;
            let rest = rest.trim();
            if rest.is_empty() {
                return Ok(self.lookup_value(kind, &name)?.clone());
            }
            if rest.starts_with('.') || rest.starts_with('[') {
                return self.eval_access_value(kind, &name, rest);
            }
            if looks_like_numeric_expr(expr) {
                return Ok(Value::Scalar(self.eval_numeric_expr(expr)?));
            }
            return Err("unexpected text after identifier".to_string());
        }
        self.eval_value_kind(expr)
    }

    fn eval_expr(&self, expr: &str) -> Result<String, String> {
        let expr = expr.trim();
        if expr.starts_with('$') {
            let (kind, name, rest) = parse_identifier(expr)?;
            let rest = rest.trim();
            if rest.is_empty() {
                return Ok(format_value(self.lookup_value(kind, &name)?));
            }
            if rest.starts_with('.') || rest.starts_with('[') {
                return Ok(format_value(&self.eval_access_value(kind, &name, rest)?));
            }
            if looks_like_numeric_expr(expr) {
                return self.eval_numeric_expr(expr);
            }
            return Err("unexpected text after identifier".to_string());
        }
        self.eval_value(expr)
    }

    fn eval_value(&self, text: &str) -> Result<String, String> {
        match self.eval_value_kind(text)? {
            Value::Scalar(value) => Ok(value),
            Value::List(_) => Err("cannot use list as scalar value".to_string()),
            Value::Dict(_) => Err("cannot use dictionary as scalar value".to_string()),
        }
    }

    fn eval_value_kind(&self, text: &str) -> Result<Value, String> {
        let text = text.trim();
        if text.starts_with('[') {
            return self.eval_list(text);
        }
        if text.starts_with('{') {
            return self.eval_dict(text);
        }
        if text.starts_with('"') {
            return Ok(Value::Scalar(self.eval_string(text)?));
        }
        if text.starts_with('$') {
            return self.eval_expr_value(text);
        }
        if text == "true" || text == "false" {
            return Ok(Value::Scalar(text.to_string()));
        }
        if is_number_literal(text) {
            return Ok(Value::Scalar(format_number(
                text.parse::<f64>().expect("number literal parses"),
            )));
        }
        if looks_like_numeric_expr(text) {
            return Ok(Value::Scalar(self.eval_numeric_expr(text)?));
        }
        Err("expected string, number, boolean, or numeric expression".to_string())
    }

    fn eval_list(&self, text: &str) -> Result<Value, String> {
        let text = text.trim();
        if !text.ends_with(']') {
            return Err("malformed list literal".to_string());
        }
        let inner = &text[1..text.len() - 1];
        if inner.trim().is_empty() {
            return Ok(Value::List(Vec::new()));
        }
        let mut values = Vec::new();
        for item in split_top_level_commas(inner)? {
            values.push(self.eval_value_kind(item)?);
        }
        Ok(Value::List(values))
    }

    fn eval_dict(&self, text: &str) -> Result<Value, String> {
        let text = text.trim();
        if !text.ends_with('}') {
            return Err("malformed dictionary literal".to_string());
        }
        let inner = &text[1..text.len() - 1];
        let mut values = std::collections::BTreeMap::new();
        if inner.trim().is_empty() {
            return Ok(Value::Dict(values));
        }
        for item in split_top_level_commas(inner)? {
            let (key_text, value_text) = split_dict_entry(item)?;
            let key = if key_text.trim().starts_with('"') {
                parse_string_literal(key_text.trim())?
            } else {
                let key = key_text.trim();
                if !is_simple_key(key) {
                    return Err("dictionary key must be an identifier or quoted string".to_string());
                }
                key.to_string()
            };
            values.insert(key, self.eval_value_kind(value_text.trim())?);
        }
        Ok(Value::Dict(values))
    }

    fn eval_access_value(&self, kind: IdentKind, name: &str, rest: &str) -> Result<Value, String> {
        let mut current = self.lookup_value(kind, name)?.clone();
        let mut path = display_identifier(kind, name);
        let mut remaining = rest.trim();
        while !remaining.is_empty() {
            if let Some(after_dot) = remaining.strip_prefix('.') {
                let key_len = after_dot
                    .chars()
                    .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                    .map(char::len_utf8)
                    .sum::<usize>();
                if key_len == 0 {
                    return Err(
                        "invalid dot key; use bracket access for non-identifier keys".to_string(),
                    );
                }
                let key = &after_dot[..key_len];
                match current {
                    Value::Dict(values) => {
                        current = values
                            .get(key)
                            .cloned()
                            .ok_or_else(|| format!("missing key '{key}' in {path}"))?;
                        path.push_str(&format!(".{key}"));
                    }
                    Value::Scalar(_) | Value::List(_) => {
                        return Err(format!("cannot access key on non-dictionary value {path}"));
                    }
                }
                remaining = after_dot[key_len..].trim_start();
            } else if remaining.starts_with('[') {
                let end = matching_bracket_end(remaining)
                    .map_err(|_| "malformed list index syntax".to_string())?;
                let selector = remaining[1..end].trim();
                if selector.is_empty() {
                    return Err("malformed list index syntax".to_string());
                }
                current = match current {
                    Value::List(values) => {
                        self.eval_list_selector_value(kind, name, &values, selector)?
                    }
                    Value::Dict(values) => {
                        let key = self.eval_value(selector)?;
                        values
                            .get(&key)
                            .cloned()
                            .ok_or_else(|| format!("missing key '{key}' in {path}"))?
                    }
                    Value::Scalar(_) => return Err(format!("cannot index non-list value {path}")),
                };
                path.push_str(&remaining[..=end]);
                remaining = remaining[end + 1..].trim_start();
            } else {
                return Err("unexpected text after identifier".to_string());
            }
        }
        Ok(current)
    }

    fn resolve_dict_path(&self, rest: &str) -> Result<Vec<String>, String> {
        let parts = parse_dict_path(rest)?;
        let mut keys = Vec::new();
        for part in parts {
            match part {
                DictPathPart::Dot(key) => keys.push(key),
                DictPathPart::Bracket(expr) => keys.push(self.eval_value(&expr)?),
            }
        }
        Ok(keys)
    }

    fn eval_list_selector_value(
        &self,
        kind: IdentKind,
        name: &str,
        values: &[Value],
        selector: &str,
    ) -> Result<Value, String> {
        if let Some((op, rhs)) = parse_filter(selector) {
            return self.eval_list_filter(values, op, rhs).map(Value::List);
        }
        if selector.contains(',') {
            let mut selected = Vec::new();
            for part in split_commas(selector)? {
                let index = self.resolve_list_position(kind, name, values.len(), part)?;
                selected.push(values[index - 1].clone());
            }
            return Ok(Value::List(selected));
        }
        if let Some((start, end)) = selector.split_once("..") {
            let start = if start.trim().is_empty() {
                1
            } else {
                self.parse_list_position(values.len(), start.trim())?
            };
            let end = if end.trim().is_empty() {
                values.len()
            } else {
                self.parse_list_position(values.len(), end.trim())?
            };
            if start > end {
                return Err("reverse list ranges are not supported".to_string());
            }
            self.ensure_list_position_in_range(kind, name, values.len(), start)?;
            self.ensure_list_position_in_range(kind, name, values.len(), end)?;
            return Ok(Value::List(values[start - 1..end].to_vec()));
        }
        let index = self.resolve_list_position(kind, name, values.len(), selector)?;
        Ok(values[index - 1].clone())
    }

    fn resolve_list_position(
        &self,
        kind: IdentKind,
        name: &str,
        len: usize,
        expr: &str,
    ) -> Result<usize, String> {
        let index = self.parse_list_position(len, expr)?;
        self.ensure_list_position_in_range(kind, name, len, index)?;
        Ok(index)
    }

    fn parse_list_position(&self, len: usize, expr: &str) -> Result<usize, String> {
        match expr {
            "first" => Ok(1),
            "last" => Ok(len),
            _ => {
                if expr.starts_with('"') {
                    return Err("list index must be numeric".to_string());
                }
                let value = self
                    .eval_numeric_expr(expr)
                    .map_err(|_| "list index must be numeric".to_string())?;
                let number = value
                    .parse::<f64>()
                    .map_err(|_| "list index must be numeric".to_string())?;
                if number.fract() != 0.0 {
                    return Err("list index must be an integer".to_string());
                }
                if number < 1.0 {
                    return Err("list index must be 1 or greater".to_string());
                }
                Ok(number as usize)
            }
        }
    }

    fn ensure_list_position_in_range(
        &self,
        kind: IdentKind,
        name: &str,
        len: usize,
        index: usize,
    ) -> Result<(), String> {
        if index == 0 || index > len {
            return Err(format!(
                "list index {index} out of range for {} (length {len})",
                display_identifier(kind, name)
            ));
        }
        Ok(())
    }

    fn eval_list_filter(
        &self,
        values: &[Value],
        op: &str,
        rhs: &str,
    ) -> Result<Vec<Value>, String> {
        let target = self.eval_value(rhs)?;
        if matches!(op, ">" | ">=" | "<" | "<=") {
            if rhs.trim().starts_with('"') {
                return Err("ordered string filters are not supported".to_string());
            }
            let target_num = target
                .parse::<f64>()
                .map_err(|_| "ordered list filters require numeric comparison value".to_string())?;
            let mut selected = Vec::new();
            for item in values {
                let Value::Scalar(item_text) = item else {
                    return Err("list filters require scalar items".to_string());
                };
                let item_num = item_text
                    .parse::<f64>()
                    .map_err(|_| "ordered list filters require numeric items".to_string())?;
                let keep = match op {
                    ">" => item_num > target_num,
                    ">=" => item_num >= target_num,
                    "<" => item_num < target_num,
                    "<=" => item_num <= target_num,
                    _ => unreachable!(),
                };
                if keep {
                    selected.push(item.clone());
                }
            }
            return Ok(selected);
        }
        let selected: Vec<Value> = values
            .iter()
            .filter(|item| match item {
                Value::Scalar(item) => match op {
                    "==" => item == &target,
                    "!=" => item != &target,
                    _ => false,
                },
                _ => false,
            })
            .cloned()
            .collect();
        Ok(selected)
    }

    fn eval_string(&self, text: &str) -> Result<String, String> {
        let raw = parse_string_literal(text)?;
        self.interpolate(&raw)
    }

    fn interpolate(&self, input: &str) -> Result<String, String> {
        if input.contains("$$") {
            return Err("legacy '$$' identifier prefix is not supported; use '$'".to_string());
        }
        let bytes = input.as_bytes();
        let mut output = String::new();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'$' {
                let tail = &input[index..];
                let (kind, name, rest) = parse_identifier(tail)?;
                let consumed = tail.len() - rest.len();
                let rest_trim = rest.trim_start();
                if rest_trim.starts_with('[') {
                    let leading_ws = rest.len() - rest_trim.len();
                    let index_tail = &rest[leading_ws..];
                    let end = access_prefix_len(index_tail)?;
                    output.push_str(&format_value(&self.eval_access_value(
                        kind,
                        &name,
                        &index_tail[..end],
                    )?));
                    index += consumed + leading_ws + end;
                } else if rest_trim.starts_with('.') {
                    let leading_ws = rest.len() - rest_trim.len();
                    let key_len = access_prefix_len(rest_trim)? - 1;
                    if key_len == 0 {
                        return Err(
                            "invalid dot key; use bracket access for non-identifier keys"
                                .to_string(),
                        );
                    }
                    let access = &rest[leading_ws..leading_ws + 1 + key_len];
                    output.push_str(&format_value(&self.eval_access_value(kind, &name, access)?));
                    index += consumed + leading_ws + 1 + key_len;
                } else {
                    output.push_str(&self.lookup(kind, &name)?);
                    index = input.len() - rest.len();
                }
            } else {
                let ch = input[index..].chars().next().expect("valid char boundary");
                output.push(ch);
                index += ch.len_utf8();
            }
        }
        Ok(output)
    }

    fn eval_numeric_expr(&self, expr: &str) -> Result<String, String> {
        let mut parser = NumericParser::new(expr, self);
        let value = parser.parse_expr()?;
        parser.skip_ws();
        if !parser.is_at_end() {
            return Err(format!(
                "unexpected text in numeric expression near '{}'",
                parser.remaining()
            ));
        }
        Ok(format_number(value))
    }

    fn mark_unavailable(&mut self, name: String) {
        self.vars.remove(&name);
        self.unavailable.insert(name);
    }

    fn lookup_value(&self, kind: IdentKind, name: &str) -> Result<&Value, String> {
        match kind {
            IdentKind::Variable => {
                if self.unavailable.contains(name) {
                    return Err(format!(
                        "dry-run unavailable-result: ${name} was not created"
                    ));
                }
                self.vars
                    .get(name)
                    .ok_or_else(|| format!("undefined variable ${name}"))
            }
            IdentKind::Constant => self
                .consts
                .get(name)
                .ok_or_else(|| format!("undefined constant ${name}")),
        }
    }

    fn lookup(&self, kind: IdentKind, name: &str) -> Result<String, String> {
        match self.lookup_value(kind, name)? {
            Value::Scalar(value) => Ok(value.clone()),
            Value::List(_) | Value::Dict(_) => Err(format!(
                "cannot use list/dictionary {} as scalar value",
                display_identifier(kind, name)
            )),
        }
    }
}

enum DictPathPart {
    Dot(String),
    Bracket(String),
}

fn parse_dict_path(rest: &str) -> Result<Vec<DictPathPart>, String> {
    let mut parts = Vec::new();
    let mut index = 0;
    let rest = rest.trim();
    while index < rest.len() {
        let tail = &rest[index..];
        if let Some(after_dot) = tail.strip_prefix('.') {
            let key_len = after_dot
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                .map(char::len_utf8)
                .sum::<usize>();
            if key_len == 0 {
                return Err(
                    "invalid dot key; use bracket access for non-identifier keys".to_string(),
                );
            }
            let key = &after_dot[..key_len];
            if !is_simple_key(key) {
                return Err(
                    "invalid dot key; use bracket access for non-identifier keys".to_string(),
                );
            }
            parts.push(DictPathPart::Dot(key.to_string()));
            index += 1 + key_len;
        } else if tail.starts_with('[') {
            let end = matching_bracket_end(tail)?;
            let expr = tail[1..end].trim();
            if expr.is_empty() {
                return Err("malformed dictionary key syntax".to_string());
            }
            parts.push(DictPathPart::Bracket(expr.to_string()));
            index += end + 1;
        } else {
            return Err("malformed dictionary key syntax".to_string());
        }
    }
    if parts.is_empty() {
        return Err("malformed dictionary key syntax".to_string());
    }
    Ok(parts)
}

fn matching_bracket_end(text: &str) -> Result<usize, String> {
    let mut in_string = false;
    let mut escaped = false;
    for (idx, ch) in text.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            ']' if !in_string => return Ok(idx),
            _ => {}
        }
    }
    Err("malformed dictionary key syntax".to_string())
}

fn access_prefix_len(text: &str) -> Result<usize, String> {
    let mut index = 0;
    while index < text.len() {
        let tail = &text[index..];
        if let Some(after_dot) = tail.strip_prefix('.') {
            let key_len = after_dot
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                .map(char::len_utf8)
                .sum::<usize>();
            if key_len == 0 {
                return Err(
                    "invalid dot key; use bracket access for non-identifier keys".to_string(),
                );
            }
            index += 1 + key_len;
        } else if tail.starts_with('[') {
            index += matching_bracket_end(tail)? + 1;
        } else {
            break;
        }
    }
    Ok(index)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CollectionMutation {
    Set,
    Append,
    Delete,
}
impl CollectionMutation {
    fn name(self) -> &'static str {
        match self {
            Self::Set => "set",
            Self::Append => "append",
            Self::Delete => "delete",
        }
    }
}

#[derive(Clone)]
enum CollectionSegment {
    Key(String),
    Index(String),
}

fn parse_collection_path(
    text: &str,
) -> Result<(IdentKind, String, Vec<CollectionSegment>), String> {
    let (kind, name, mut rest) = parse_identifier(text)?;
    let mut segments = Vec::new();
    while !rest.trim().is_empty() {
        rest = rest.trim_start();
        if let Some(tail) = rest.strip_prefix('.') {
            let end = tail
                .find(|c: char| c == '.' || c == '[')
                .unwrap_or(tail.len());
            let key = &tail[..end];
            if !is_simple_key(key) {
                return Err("invalid collection path key".to_string());
            }
            segments.push(CollectionSegment::Key(key.to_string()));
            rest = &tail[end..];
        } else if rest.starts_with('[') {
            let end = matching_bracket_end(rest)?;
            let inner = rest[1..end].trim();
            if inner.starts_with('"') {
                segments.push(CollectionSegment::Key(parse_string_literal(inner)?));
            } else {
                segments.push(CollectionSegment::Index(inner.to_string()));
            }
            rest = &rest[end + 1..];
        } else {
            return Err("malformed collection path".to_string());
        }
    }
    if segments.is_empty() {
        return Err("collection path requires a nested segment".to_string());
    }
    Ok((kind, name, segments))
}

fn apply_collection_mutation(
    target: &mut Value,
    segments: &[CollectionSegment],
    mutation: CollectionMutation,
    value: Option<Value>,
    base: &str,
) -> Result<(), String> {
    let (last, parents) = segments.split_last().unwrap();
    let mut current = target;
    for segment in parents {
        current = match segment {
            CollectionSegment::Key(key) => match current {
                Value::Dict(map) => map
                    .get_mut(key)
                    .ok_or_else(|| format!("missing key '{key}' in {base}"))?,
                _ => return Err(format!("cannot access key on non-dictionary value {base}")),
            },
            CollectionSegment::Index(raw) => match current {
                Value::List(items) => {
                    let i = strict_collection_index(raw, items.len(), base)?;
                    &mut items[i]
                }
                _ => return Err(format!("cannot index non-list value {base}")),
            },
        };
    }
    match (last, mutation, current) {
        (CollectionSegment::Key(key), CollectionMutation::Set, Value::Dict(map)) => {
            map.insert(key.clone(), value.unwrap());
            Ok(())
        }
        (CollectionSegment::Key(key), CollectionMutation::Append, Value::Dict(map)) => {
            match map.get_mut(key) {
                Some(Value::List(items)) => {
                    items.push(value.unwrap());
                    Ok(())
                }
                Some(_) => Err("append target must be a list".to_string()),
                None => Err(format!("missing key '{key}' in {base}")),
            }
        }
        (CollectionSegment::Key(key), CollectionMutation::Delete, Value::Dict(map)) => map
            .remove(key)
            .map(|_| ())
            .ok_or_else(|| format!("missing key '{key}' in {base}")),
        (CollectionSegment::Index(raw), CollectionMutation::Set, Value::List(items)) => {
            let i = strict_collection_index(raw, items.len(), base)?;
            items[i] = value.unwrap();
            Ok(())
        }
        (CollectionSegment::Index(raw), CollectionMutation::Delete, Value::List(items)) => {
            let i = strict_collection_index(raw, items.len(), base)?;
            items.remove(i);
            Ok(())
        }
        (CollectionSegment::Index(raw), CollectionMutation::Append, Value::List(items)) => {
            let i = strict_collection_index(raw, items.len(), base)?;
            match &mut items[i] {
                Value::List(nested) => {
                    nested.push(value.unwrap());
                    Ok(())
                }
                _ => Err("append target must be a list".to_string()),
            }
        }
        _ => Err("mutation target has the wrong collection type".to_string()),
    }
}

fn strict_collection_index(raw: &str, len: usize, base: &str) -> Result<usize, String> {
    let index = raw
        .parse::<usize>()
        .map_err(|_| "list index must be a positive integer".to_string())?;
    if index == 0 || index > len {
        return Err(format!(
            "list index {index} out of range for {base} (length {len})"
        ));
    }
    Ok(index - 1)
}

fn replace_nested_dict_value(
    target: &mut Value,
    keys: &[String],
    value: Value,
    base: &str,
) -> Result<(), String> {
    let (last, parents) = keys.split_last().expect("path has at least one key");
    let mut current = target;
    let mut path = base.to_string();
    for key in parents {
        match current {
            Value::Dict(values) => {
                current = values
                    .get_mut(key)
                    .ok_or_else(|| format!("missing key '{key}' in {path}"))?;
                path.push_str(&format!(".{key}"));
            }
            Value::Scalar(_) | Value::List(_) => {
                return Err(format!("cannot access key on non-dictionary value {path}"));
            }
        }
    }
    match current {
        Value::Dict(values) => {
            if !values.contains_key(last) {
                return Err(format!("missing key '{last}' in {path}"));
            }
            values.insert(last.clone(), value);
            Ok(())
        }
        Value::Scalar(_) | Value::List(_) => {
            Err(format!("cannot access key on non-dictionary value {path}"))
        }
    }
}

fn remove_nested_dict_value(target: &mut Value, keys: &[String], base: &str) -> Result<(), String> {
    let (last, parents) = keys.split_last().expect("path has at least one key");
    let mut current = target;
    let mut path = base.to_string();
    for key in parents {
        match current {
            Value::Dict(values) => {
                current = values
                    .get_mut(key)
                    .ok_or_else(|| format!("missing key '{key}' in {path}"))?;
                path.push_str(&format!(".{key}"));
            }
            Value::Scalar(_) | Value::List(_) => {
                return Err(format!("cannot access key on non-dictionary value {path}"));
            }
        }
    }
    match current {
        Value::Dict(values) => values
            .remove(last)
            .map(|_| ())
            .ok_or_else(|| format!("missing key '{last}' in {path}")),
        Value::Scalar(_) | Value::List(_) => {
            Err(format!("cannot access key on non-dictionary value {path}"))
        }
    }
}

struct Checker {
    file: String,
    diagnostics: Vec<Diagnostic>,
    blocks: Vec<BlockKind>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    If,
    Case { saw_when: bool, saw_else: bool },
    Loop,
    Function,
    Try { saw_catch: bool },
    Log,
    Timeout,
    Defer,
    Watch,
}

impl Checker {
    fn new(file: String) -> Self {
        Self {
            file,
            diagnostics: Vec::new(),
            blocks: Vec::new(),
        }
    }

    fn check_lines(&mut self, lines: &[(usize, String)]) {
        for (line_no, raw) in lines {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            if let Err(message) = self.check_line(line) {
                self.error(*line_no, message);
            }
        }
        if let Some(kind) = self.blocks.last().copied() {
            let name = match kind {
                BlockKind::If => "if",
                BlockKind::Case { .. } => "case",
                BlockKind::Loop => "loop",
                BlockKind::Function => "function",
                BlockKind::Try { saw_catch } => {
                    if !saw_catch {
                        self.error(
                            lines.last().map(|(n, _)| *n).unwrap_or(1),
                            "missing catch for try".to_string(),
                        );
                    }
                    "try"
                }
                BlockKind::Log => "log",
                BlockKind::Timeout => "timeout",
                BlockKind::Defer => "defer",
                BlockKind::Watch => "watch",
            };
            if matches!(
                kind,
                BlockKind::Case {
                    saw_when: false,
                    ..
                }
            ) {
                self.error(
                    lines.last().map(|(n, _)| *n).unwrap_or(1),
                    "case requires at least one when".to_string(),
                );
            }
            self.error(
                lines.last().map(|(n, _)| *n).unwrap_or(1),
                format!("missing end for {name}"),
            );
        }
    }

    fn check_line(&mut self, line: &str) -> Result<(), String> {
        if let Some(rest) = line.strip_prefix("archive ") {
            let (op, tail) = rest
                .split_once(char::is_whitespace)
                .ok_or("archive operation required")?;
            if !matches!(op, "create" | "extract" | "checksum" | "check") {
                return Err("unknown archive operation".into());
            }
            let (args, dst) = tail
                .split_once("->")
                .ok_or("archive syntax requires -> $result")?;
            if args.trim().is_empty() || dst.trim().is_empty() || dst.contains("->") {
                return Err("archive syntax requires one operand and one result".into());
            }
            parse_var_ref(dst.trim())?;
            match op {
                "create" if args.matches(" from ").count() == 1 => {
                    let mut p = args.split(" from ");
                    if p.next().unwrap().trim().is_empty() || p.next().unwrap().trim().is_empty() {
                        return Err("archive create operands required".into());
                    }
                }
                "extract" if args.matches(" into ").count() == 1 => {
                    let mut p = args.split(" into ");
                    if p.next().unwrap().trim().is_empty() || p.next().unwrap().trim().is_empty() {
                        return Err("archive extract operands required".into());
                    }
                }
                "checksum" | "check" => {}
                _ => return Err("archive syntax invalid".into()),
            }
            return Ok(());
        }
        if line.starts_with("export $") {
            return Ok(());
        }
        if let Some(rest) = line
            .strip_prefix("export fn ")
            .or_else(|| line.strip_prefix("fn "))
        {
            let rest = rest.trim();
            if rest.ends_with("{}") {
                parse_function_signature(&format!("{} {{", rest.trim_end_matches("{}")))?;
                return Ok(());
            }
            parse_function_signature(rest)?;
            self.blocks.push(BlockKind::Function);
            return Ok(());
        }
        if let Some(rest) = line.strip_prefix("case ") {
            validate_case_value(rest.trim())?;
            self.blocks.push(BlockKind::Case {
                saw_when: false,
                saw_else: false,
            });
            return Ok(());
        }
        if line == "case" {
            return Err("case value required".to_string());
        }
        if let Some(rest) = line.strip_prefix("when ") {
            match self.blocks.last_mut() {
                Some(BlockKind::Case { saw_when, saw_else }) if !*saw_else => {
                    validate_when_values(rest.trim())?;
                    *saw_when = true;
                    return Ok(());
                }
                Some(BlockKind::Case { .. }) => return Err("when after else".to_string()),
                _ => return Err("when outside case".to_string()),
            }
        }
        if line == "when" {
            if matches!(self.blocks.last(), Some(BlockKind::Case { .. })) {
                return Err("when value required".to_string());
            }
            return Err("when outside case".to_string());
        }
        if let Some(rest) = line.strip_prefix("if ") {
            validate_condition_form(rest.trim())?;
            self.blocks.push(BlockKind::If);
            return Ok(());
        }
        if line == "if" {
            return Err("condition required".to_string());
        }
        if let Some(rest) = line.strip_prefix("elif ") {
            if !matches!(self.blocks.last(), Some(BlockKind::If)) {
                return Err("unexpected conditional marker".to_string());
            }
            validate_condition_form(rest.trim())?;
            return Ok(());
        }
        if line == "else" {
            match self.blocks.last_mut() {
                Some(BlockKind::If) => return Ok(()),
                Some(BlockKind::Case { saw_else, .. }) if !*saw_else => {
                    *saw_else = true;
                    return Ok(());
                }
                Some(BlockKind::Case { .. }) => return Err("duplicate else".to_string()),
                _ => return Err("unexpected conditional marker".to_string()),
            }
        }
        if line == "defer" {
            self.blocks.push(BlockKind::Defer);
            return Ok(());
        }
        if line.starts_with("defer ") {
            return Err("defer takes no arguments".to_string());
        }
        if let Some(rest) = line.strip_prefix("timeout ") {
            parse_timeout(rest.trim())?;
            self.blocks.push(BlockKind::Timeout);
            return Ok(());
        }
        if line == "timeout" {
            return Err("timeout value required".to_string());
        }
        if line == "wait" {
            return Ok(());
        }
        if line.starts_with("wait ") {
            return Err("wait takes no arguments".to_string());
        }
        if let Some(rest) = line.strip_prefix("watch ") {
            let (command_text, handler_text) = rest
                .split_once("->")
                .ok_or("watch syntax is watch <program> <args...> -> $line")?;
            let (_, _, tail) = validate_command_atoms(command_text.trim())?;
            if !tail.trim().is_empty() {
                return Err("watch syntax is watch <program> <args...> -> $line".to_string());
            }
            if !handler_text.trim().starts_with('$') || handler_text.trim() == "$" {
                return Err("watch destination must be a variable".to_string());
            }
            let destinations =
                parse_command_capture_destinations(handler_text.trim()).map_err(|e| {
                    if e.contains("destination") {
                        e
                    } else {
                        format!("watch {e}")
                    }
                })?;
            if destinations.len() != 1 {
                return Err("watch requires exactly one line destination".to_string());
            }
            self.blocks.push(BlockKind::Watch);
            return Ok(());
        }
        if let Some(rest) = line.strip_prefix("while ") {
            validate_condition_form(rest.trim())?;
            self.blocks.push(BlockKind::Loop);
            return Ok(());
        }
        if line == "while" {
            return Err("condition required".to_string());
        }
        if let Some(rest) = line.strip_prefix("for ") {
            parse_for_header(rest.trim())?;
            self.blocks.push(BlockKind::Loop);
            return Ok(());
        }
        if line == "for" {
            return Err("for syntax is for $item in $list".to_string());
        }
        if let Some(rest) = line.strip_prefix("repeat ") {
            parse_repeat_header(rest.trim())?;
            self.blocks.push(BlockKind::Loop);
            return Ok(());
        }
        if line == "repeat" {
            return Err("repeat count required".to_string());
        }
        if line == "try" || line.starts_with("try ") {
            parse_try_header(line)?;
            self.blocks.push(BlockKind::Try { saw_catch: false });
            return Ok(());
        }
        if line == "log" {
            return Err("log identifier required".to_string());
        }
        if let Some(rest) = line.strip_prefix("log ") {
            validate_log_header(rest.trim())?;
            self.blocks.push(BlockKind::Log);
            return Ok(());
        }
        if line == "catch" {
            match self.blocks.last_mut() {
                Some(BlockKind::Try { saw_catch }) if !*saw_catch => {
                    *saw_catch = true;
                    return Ok(());
                }
                Some(BlockKind::Try { .. }) => return Err("duplicate catch".to_string()),
                _ => return Err("catch outside try".to_string()),
            }
        }
        if line == "}" {
            match self.blocks.pop() {
                Some(BlockKind::Function) => {}
                Some(kind) => {
                    self.blocks.push(kind);
                    return Err("unexpected '}' outside function".to_string());
                }
                None => return Err("unexpected '}' outside function".to_string()),
            }
            return Ok(());
        }
        if line == "end" {
            match self.blocks.pop() {
                Some(BlockKind::Function) => {
                    self.blocks.push(BlockKind::Function);
                    return Err(
                        "old function syntax was removed; close functions with }".to_string()
                    );
                }
                Some(BlockKind::Try { saw_catch: false }) => {
                    return Err("missing catch for try".to_string());
                }
                Some(BlockKind::Case {
                    saw_when: false, ..
                }) => {
                    return Err("case requires at least one when".to_string());
                }
                Some(_) => {}
                None => return Err("unexpected conditional marker".to_string()),
            }
            return Ok(());
        }
        if line == "break" || line == "continue" {
            let function_boundary = self
                .blocks
                .iter()
                .rposition(|kind| *kind == BlockKind::Function);
            let in_loop = self.blocks.iter().enumerate().any(|(index, kind)| {
                *kind == BlockKind::Loop
                    && function_boundary.map_or(true, |boundary| index > boundary)
            });
            if !in_loop {
                return Err(format!("{line} outside loop"));
            }
            return Ok(());
        }
        if line == "return" || line.starts_with("return ") {
            if !self.blocks.iter().any(|kind| *kind == BlockKind::Function) {
                return Err("return outside function".to_string());
            }
            if line == "return" {
                return Err("return value required".to_string());
            }
            return Ok(());
        }
        validate_statement_form(line)
    }

    fn error(&mut self, line: usize, message: String) {
        self.diagnostics.push(Diagnostic {
            file: self.file.clone(),
            line,
            column: 1,
            severity: "error",
            message,
        });
    }
}

fn parse_capture_destination(destination: &str, command: &str) -> Result<String, String> {
    if destination.trim().is_empty() {
        return Err(format!("{command} destination must be a variable"));
    }
    let (kind, name, extra) = parse_identifier(destination.trim())
        .map_err(|_| format!("{command} destination must be a variable"))?;
    if kind != IdentKind::Variable || !extra.trim().is_empty() {
        return Err(format!("{command} destination must be a variable"));
    }
    Ok(name)
}

fn parse_arg_capture(text: &str) -> Result<(&str, String), String> {
    let Some(index) = find_capture_arrow(text) else {
        return Err("arg syntax is arg N -> $var".to_string());
    };
    let index_text = text[..index].trim();
    if index_text.is_empty() {
        return Err("arg index must be a positive integer".to_string());
    }
    let destination = parse_capture_destination(&text[index + 2..], "arg")?;
    Ok((index_text, destination))
}

fn parse_has_arg_capture(text: &str) -> Result<(&str, String), String> {
    let Some(index) = find_capture_arrow(text) else {
        return Err("hasArg syntax is hasArg VALUE -> $var".to_string());
    };
    let value_text = text[..index].trim();
    if value_text.is_empty() {
        return Err("hasArg value required".to_string());
    }
    let destination = parse_capture_destination(&text[index + 2..], "hasArg")?;
    Ok((value_text, destination))
}

fn parse_args_capture(text: &str) -> Result<String, String> {
    let Some(destination) = text.trim().strip_prefix("->") else {
        return Err("args syntax is args -> $var".to_string());
    };
    parse_capture_destination(destination, "args")
}

fn validate_arg_index_form(index_text: &str) -> Result<(), String> {
    let index_text = index_text.trim();
    if index_text.is_empty() || index_text == "0" || index_text.starts_with('-') {
        return Err("arg index must be a positive integer".to_string());
    }
    if index_text.chars().all(|ch| ch.is_ascii_digit()) {
        return Ok(());
    }
    if index_text.starts_with('"') || matches!(index_text, "true" | "false") {
        return Err("arg index must be a positive integer".to_string());
    }
    Ok(())
}

fn validate_temp_capture(text: &str, command: &str) -> Result<(), String> {
    let Some(destination) = text.strip_prefix("->") else {
        return Err(format!("{command} syntax is {command} -> $path"));
    };
    let (kind, _, extra) = parse_identifier(destination.trim())
        .map_err(|_| "temp destination must be a variable".to_string())?;
    if kind != IdentKind::Variable || !extra.trim().is_empty() {
        return Err("temp destination must be a variable".to_string());
    }
    Ok(())
}

fn parse_function_signature(rest: &str) -> Result<(), String> {
    parse_function_header(rest).map(|_| ())
}

fn validate_assignment_form(line: &str) -> Result<(), String> {
    let (kind, name, rest) = parse_identifier(line)?;
    let rest = rest.trim_start();
    if rest.is_empty() {
        if kind == IdentKind::Constant {
            return Err("constant must have an initial value".to_string());
        }
        return Ok(());
    }
    if rest.starts_with('.') || rest.starts_with('[') {
        let (path_text, value_text) = rest
            .rsplit_once('=')
            .ok_or_else(|| "unexpected text after identifier".to_string())?;
        if rest.starts_with('.') || path_text.contains('.') || path_text.matches('[').count() > 1 {
            parse_dict_path(path_text.trim())?;
        } else if !path_text.starts_with('[') || !path_text.ends_with(']') {
            return Err("malformed list index syntax".to_string());
        }
        if value_text.trim().is_empty() {
            return Err("expected value after '='".to_string());
        }
        let _ = name;
        return Ok(());
    }
    let value_text = rest
        .strip_prefix('=')
        .ok_or_else(|| "unexpected text after identifier".to_string())?
        .trim_start();
    if value_text.is_empty() {
        return Err("expected value after '='".to_string());
    }
    if value_text.starts_with("capture") {
        return Err("capture syntax was removed; use command \"...\" -> $var".to_string());
    }
    if value_text.starts_with('{') || value_text.starts_with('[') || value_text.starts_with('"') {
        Interpreter::default().eval_value_kind(value_text)?;
    }
    Ok(())
}

fn validate_statement_form(line: &str) -> Result<(), String> {
    if let Some(rest) = line.strip_prefix("http ") {
        let (method, args, tail) = validate_command_atoms(rest.trim())?;
        if !matches!(
            method.as_str(),
            "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS"
        ) {
            return Err(
                "http method must be GET, POST, PUT, PATCH, DELETE, HEAD, or OPTIONS".into(),
            );
        }
        if args.is_empty() {
            return Err("http requires METHOD URL".into());
        }
        let mut seen = HashSet::new();
        let mut i = 1;
        while i < args.len() {
            let option = &args[i];
            if !matches!(option.as_str(), "headers" | "body" | "timeout" | "maxBytes") {
                return Err(format!("unexpected http option: {option}"));
            }
            if !seen.insert(option) {
                return Err(format!("http option repeated: {option}"));
            }
            let value = args
                .get(i + 1)
                .ok_or_else(|| format!("{option} requires a value"))?;
            match option.as_str() {
                "headers" if value != "$" && !value.starts_with('{') => {
                    return Err("headers must be a dictionary".into());
                }
                "body" if value != "$" && !value.starts_with('"') => {
                    return Err("body must be text".into());
                }
                "timeout" if value != "$" && value.parse::<f64>().is_err() => {
                    return Err("timeout must be a number".into());
                }
                "maxBytes" if value != "$" && value.parse::<usize>().is_err() => {
                    return Err("maxBytes must be an integer".into());
                }
                _ => {}
            }
            i += 2;
        }
        let destination = tail
            .trim()
            .strip_prefix("->")
            .ok_or("http requires exactly one capture: -> $response")?;
        if parse_command_capture_destinations(destination.trim())?.len() != 1 {
            return Err("http requires exactly one capture".into());
        }
        return Ok(());
    }
    if line.starts_with('$') {
        return validate_assignment_form(line);
    }
    if let Some(rest) = line.strip_prefix("arg ") {
        let (index_text, _) = parse_arg_capture(rest.trim())?;
        validate_arg_index_form(index_text)?;
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("hasArg ") {
        parse_has_arg_capture(rest.trim())?;
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("args") {
        parse_args_capture(rest.trim())?;
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("ask") {
        let rest = rest.trim();
        if rest.contains(',') && !rest.contains("->") {
            return Err(
                "old ask comma syntax was removed; use ask \"Question?\" -> $answer".to_string(),
            );
        }
        let Some((prompt_text, destination)) = rest.split_once("->") else {
            return Err("ask syntax is ask \"Question?\" -> $answer".to_string());
        };
        parse_string_literal(prompt_text.trim())?;
        let (kind, _, extra) = parse_identifier(destination.trim())
            .map_err(|_| "ask destination must be a variable".to_string())?;
        if kind != IdentKind::Variable || !extra.trim().is_empty() {
            return Err("ask destination must be a variable".to_string());
        }
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("choose ") {
        let (_, options_text, _) = parse_choose_capture(rest.trim())?;
        if options_text.trim() == "[]" {
            return Err("choose options must not be empty".to_string());
        }
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("exists ") {
        let Some((args, destination)) = split_capture_arrow(rest.trim()) else {
            return Err("predicate capture syntax is exists/contains ... -> $var".to_string());
        };
        parse_string_literal(args.trim())
            .map_err(|_| "exists requires a string path".to_string())?;
        let (kind, _, extra) = parse_identifier(destination.trim())
            .map_err(|_| "predicate capture destination must be a variable".to_string())?;
        if kind != IdentKind::Variable || !extra.trim().is_empty() {
            return Err("predicate capture destination must be a variable".to_string());
        }
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("contains ") {
        let Some((args, destination)) = split_capture_arrow(rest.trim()) else {
            return Err("predicate capture syntax is exists/contains ... -> $var".to_string());
        };
        split_predicate_two_args(args.trim())
            .ok_or_else(|| "contains requires a value and search value".to_string())?;
        let (kind, _, extra) = parse_identifier(destination.trim())
            .map_err(|_| "predicate capture destination must be a variable".to_string())?;
        if kind != IdentKind::Variable || !extra.trim().is_empty() {
            return Err("predicate capture destination must be a variable".to_string());
        }
        return Ok(());
    }
    if let Some((name, rest)) = split_command_name(line) {
        if matches!(
            name,
            "assertEqual" | "assertTrue" | "assertFalse" | "assertContains"
        ) {
            validate_assertion_form(name, rest.trim())?;
            return Ok(());
        }
        if is_standard_helper(name) {
            validate_helper_form(name, rest.trim())?;
            return Ok(());
        }
    }
    if let Some(rest) = line.strip_prefix("files ") {
        parse_files_capture(rest.trim())?;
        return Ok(());
    }
    if line == "files" {
        return Err("files syntax is files \"pattern\" -> $items".to_string());
    }
    if let Some(rest) = line.strip_prefix("create ") {
        validate_path_expr_form(rest.trim(), "create requires a string path")?;
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("copy ") {
        parse_two_string_paths(rest.trim(), "copy")?;
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("move ") {
        parse_two_string_paths(rest.trim(), "move")?;
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("rename ") {
        let (_, name) = parse_two_string_paths(rest.trim(), "rename")?;
        if name.contains('/')
            || name.contains('\\')
            || name == "."
            || name == ".."
            || name.is_empty()
        {
            return Err("rename new-name must be a plain name, not a path".to_string());
        }
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("read ") {
        parse_read_capture(rest.trim())?;
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("isFile ") {
        parse_path_capture(rest.trim(), "isFile")?;
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("isDir ") {
        parse_path_capture(rest.trim(), "isDir")?;
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("tempFile") {
        validate_temp_capture(rest.trim(), "tempFile")?;
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("tempDir") {
        validate_temp_capture(rest.trim(), "tempDir")?;
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("size ") {
        parse_path_capture(rest.trim(), "size")?;
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("append ") {
        let rest = rest.trim();
        if rest.trim_start().starts_with('"') || !rest.contains(',') {
            let (_, value) = split_path_expr_tail(rest, "append")?;
            if value.trim().is_empty() {
                return Err("append syntax is append \"file\" $text".to_string());
            }
            return Ok(());
        }
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("write ") {
        let (_, value) = split_path_expr_tail(rest.trim(), "write")?;
        if value.trim().is_empty() {
            return Err("write syntax is write \"file\" $text".to_string());
        }
        return Ok(());
    }
    if let Some(rest) = line.strip_prefix("remove ") {
        let rest = rest.trim();
        if rest.trim_start().starts_with('"') || !looks_like_list_mutation_target(rest) {
            validate_path_expr_form(rest, "remove requires a string path")?;
            return Ok(());
        }
    }
    if let Some((name, rest)) = split_command_name(line) {
        if name != "command" {
            // Continue to the handlers for unrelated statements such as helpers and shellout.
        } else {
            if rest.trim_start().starts_with('"') {
                return Err("legacy command syntax is not supported; use command <program> <args...> or shell \"...\"".to_string());
            }
            let (command_text, _) = split_timeout_suffix(rest.trim())?;
            let (_, _, tail) = validate_command_atoms(command_text)?;
            let tail = tail.trim();
            if tail.is_empty() {
                return Ok(());
            }
            let Some(destination) = tail.strip_prefix("->") else {
                return Err(command_capture_syntax_error());
            };
            parse_command_capture_destinations(destination.trim())?;
            return Ok(());
        }
    }
    if let Some(rest) = line.strip_prefix("call ") {
        validate_call_syntax(rest.trim())?;
        return Ok(());
    }
    if line == "call" {
        return Err("call requires a function name".to_string());
    }
    if let Some(rest) = line.strip_prefix("shellout") {
        if rest.trim().starts_with("(\"") {
            return Err("expected string, number, boolean, or numeric expression".to_string());
        }
        return Ok(());
    }
    if line.strip_prefix("shellin").is_some()
        || line.starts_with("import ")
        || line.starts_with("include ")
        || line.starts_with("exit")
        || line.starts_with("prepend ")
        || line.starts_with("remove ")
        || line.starts_with("global")
    {
        return Ok(());
    }
    Err(format!("unknown command or invalid syntax: {line}"))
}

fn validate_command_atoms(mut text: &str) -> Result<(String, Vec<String>, &str), String> {
    let mut atoms = Vec::new();
    while !text.trim_start().is_empty() && !text.trim_start().starts_with("->") {
        text = text.trim_start();
        if text.starts_with('"') {
            let (literal, rest) = split_string_literal_tail(text)?;
            atoms.push(literal.to_string());
            text = rest;
        } else if text.starts_with('$') {
            let (_, _, rest) = parse_identifier(text)?;
            atoms.push("$".to_string());
            text = rest;
        } else {
            let end = text.find(char::is_whitespace).unwrap_or(text.len());
            atoms.push(text[..end].to_string());
            text = &text[end..];
        }
    }
    if atoms.is_empty() {
        return Err("command requires a program and argv atoms".to_string());
    }
    Ok((atoms[0].clone(), atoms[1..].to_vec(), text))
}

fn parse_timeout(text: &str) -> Result<Duration, String> {
    let seconds: f64 = text
        .parse()
        .map_err(|_| "timeout must be a positive number".to_string())?;
    if !seconds.is_finite() || seconds <= 0.0 {
        return Err("timeout must be a positive number".to_string());
    }
    Ok(Duration::from_secs_f64(seconds))
}

fn split_timeout_suffix(text: &str) -> Result<(&str, Option<Duration>), String> {
    if let Some(pos) = text.rfind(" timeout ") {
        let command = &text[..pos];
        let value = &text[pos + 9..];
        if !command.is_empty() {
            return Ok((command, Some(parse_timeout(value.trim())?)));
        }
    }
    Ok((text, None))
}

fn poll_child(
    child: &mut std::process::Child,
    deadline: Option<Instant>,
) -> Result<std::process::ExitStatus, ()> {
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            return Ok(status);
        }
        if deadline.is_some_and(|d| Instant::now() >= d) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(());
        }
        thread::sleep(Duration::from_millis(5));
    }
}

fn poll_output(
    mut child: std::process::Child,
    deadline: Option<Instant>,
) -> Result<std::process::Output, ()> {
    poll_child(&mut child, deadline).map(|status| {
        let output = child.wait_with_output().expect("process wait");
        std::process::Output {
            status,
            stdout: output.stdout,
            stderr: output.stderr,
        }
    })
}

fn command_capture_syntax_error() -> String {
    "command capture syntax is command <program> <args...> -> $var or command <program> <args...> -> $stdout, $stderr, $code; use shell \"...\" for shell text"
        .to_string()
}

fn parse_command_capture_destinations(destination_text: &str) -> Result<Vec<String>, String> {
    let destinations =
        split_commas(destination_text).map_err(|_| command_capture_syntax_error())?;
    match destinations.len() {
        1 | 3 => {}
        2 => {
            return Err(
                "command capture requires either 1 or 3 destinations; 2 destinations are not supported"
                    .to_string(),
            );
        }
        count if count > 3 => {
            return Err("command capture supports at most 3 destinations".to_string());
        }
        _ => return Err(command_capture_syntax_error()),
    }

    destinations
        .into_iter()
        .map(|destination| {
            let (kind, name, extra) = parse_identifier(destination.trim())
                .map_err(|_| "command capture destination must be a variable".to_string())?;
            if kind != IdentKind::Variable || !extra.trim().is_empty() {
                return Err("command capture destination must be a variable".to_string());
            }
            Ok(name)
        })
        .collect()
}

fn command_output_to_string(bytes: Vec<u8>, stream_name: &str) -> Result<String, String> {
    Ok(String::from_utf8(bytes)
        .map_err(|error| format!("command {stream_name} was not UTF-8: {error}"))?
        .trim_end_matches(['\r', '\n'])
        .to_string())
}

#[derive(Clone)]
struct Function {
    name: String,
    params: Vec<Param>,
    body: Vec<(usize, String)>,
}

#[derive(Clone)]
struct Param {
    name: String,
    default: Option<String>,
}

fn split_assertion_args(text: &str) -> Result<Vec<&str>, String> {
    let mut args = Vec::new();
    let mut start = None;
    let mut quote = false;
    let mut depth = 0;
    for (index, ch) in text.char_indices() {
        if !quote && ch.is_whitespace() && depth == 0 {
            if let Some(begin) = start.take() {
                args.push(text[begin..index].trim());
            }
            continue;
        }
        if start.is_none() {
            start = Some(index);
        }
        if ch == '"' {
            quote = !quote;
        }
        if !quote {
            match ch {
                '[' | '{' | '(' => depth += 1,
                ']' | '}' | ')' => depth -= 1,
                _ => {}
            }
        }
    }
    if quote || depth != 0 {
        return Err("malformed assertion arguments".to_string());
    }
    if let Some(begin) = start {
        args.push(text[begin..].trim());
    }
    Ok(args)
}

fn assertion_arity(name: &str) -> usize {
    if name == "assertTrue" || name == "assertFalse" {
        1
    } else {
        2
    }
}

fn validate_assertion_form(name: &str, text: &str) -> Result<(), String> {
    let args = split_assertion_args(text)?;
    let expected = assertion_arity(name);
    if args.len() != expected {
        return Err(format!(
            "{name} expects {expected} argument{}",
            if expected == 1 { "" } else { "s" }
        ));
    }
    for arg in args {
        if arg.starts_with('$') {
            continue;
        }
        Interpreter::default()
            .eval_expr_value(arg)
            .map(|_| ())
            .map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}

fn validate_condition_form(condition: &str) -> Result<(), String> {
    let condition = condition.trim();
    if condition.is_empty() {
        return Err("condition required".to_string());
    }
    validate_condition_parentheses(condition)?;
    if let Some((left, right)) = split_condition_keyword(condition, "or") {
        validate_condition_form(left)?;
        return validate_condition_form(right);
    }
    if let Some((left, right)) = split_condition_keyword(condition, "and") {
        validate_condition_form(left)?;
        return validate_condition_form(right);
    }
    if let Some(rest) = condition.strip_prefix("not ") {
        return validate_condition_form(rest);
    }
    if let Some(inner) = strip_grouped_condition(condition)? {
        return validate_condition_form(inner);
    }
    if condition.starts_with(')') {
        return Err("unexpected ')' in condition".to_string());
    }
    if condition.starts_with('(') {
        return Err("missing ')' in condition".to_string());
    }
    if let Some(args) = condition.strip_prefix("exists ") {
        parse_string_literal(args).map_err(|_| "exists requires a string path".to_string())?;
    } else if condition == "exists" {
        return Err("exists requires a string path".to_string());
    } else if let Some(args) = condition.strip_prefix("contains ") {
        split_predicate_two_args(args)
            .ok_or_else(|| "contains requires a value and search value".to_string())?;
    } else if let Some(args) = condition.strip_prefix("matches ") {
        split_matches_args(args).map(|_| ())?;
    } else if condition == "contains" {
        return Err("contains requires a value and search value".to_string());
    }
    Ok(())
}

fn split_command_name(line: &str) -> Option<(&str, &str)> {
    Some(line.split_once(char::is_whitespace).unwrap_or((line, "")))
}

fn is_standard_helper(name: &str) -> bool {
    matches!(
        name,
        "trim"
            | "lower"
            | "upper"
            | "replace"
            | "substring"
            | "padLeft"
            | "padRight"
            | "formatDate"
            | "split"
            | "startsWith"
            | "endsWith"
            | "len"
            | "join"
            | "sort"
            | "reverse"
            | "unique"
            | "toNumber"
            | "toString"
            | "isEmpty"
            | "isNumber"
            | "isText"
            | "isList"
            | "isDict"
            | "keys"
            | "values"
            | "jsonParse"
            | "jsonStringify"
            | "pathJoin"
            | "basename"
            | "dirname"
            | "extension"
            | "withoutExtension"
            | "now"
            | "timestamp"
            | "today"
            | "commandExists"
            | "which"
            | "random"
            | "randomInt"
    )
}

fn parse_helper_capture<'a>(name: &str, text: &'a str) -> Result<(&'a str, String), String> {
    let Some(index) = find_capture_arrow(text) else {
        return Err(format!("{name} syntax requires -> $result"));
    };
    let args = text[..index].trim();
    let destination = text[index + 2..].trim();
    if destination.is_empty() {
        return Err(format!("{name} destination must be a variable"));
    }
    let (kind, dest, extra) = parse_identifier(destination)
        .map_err(|_| format!("{name} destination must be a variable"))?;
    if kind != IdentKind::Variable || !extra.trim().is_empty() {
        return Err(format!("{name} destination must be a variable"));
    }
    Ok((args, dest))
}

fn find_capture_arrow(text: &str) -> Option<usize> {
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
            continue;
        }
        if text[index..].starts_with("->") {
            return Some(index);
        }
    }
    None
}

fn validate_helper_form(name: &str, text: &str) -> Result<(), String> {
    let (args_text, _) = parse_helper_capture(name, text)?;
    let args = split_space_args(args_text)?;
    let n = match name {
        "trim" | "lower" | "upper" | "len" | "sort" | "reverse" | "unique" | "toNumber"
        | "toString" | "isEmpty" | "isNumber" | "isText" | "isList" | "isDict" | "keys"
        | "values" | "jsonParse" | "jsonStringify" | "dirname" | "extension"
        | "withoutExtension" | "commandExists" | "which" => 1,
        "split" | "startsWith" | "endsWith" | "join" | "randomInt" => 2,
        "replace" | "substring" => 3,
        "padLeft" | "padRight" => 3,
        "formatDate" => 2,
        "now" | "timestamp" | "today" | "random" => 0,
        "pathJoin" => {
            expect_min_arg_count(name, &args, 2)?;
            return Ok(());
        }
        "basename" => {
            if args.len() == 1 || args.len() == 2 {
                return Ok(());
            }
            return Err("basename expects 1 or 2 argument(s)".to_string());
        }
        _ => unreachable!(),
    };
    expect_arg_count(name, &args, n)
}

fn json_to_value(value: serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::Scalar(String::new()),
        serde_json::Value::Bool(value) => Value::Scalar(value.to_string()),
        serde_json::Value::Number(value) => Value::Scalar(value.to_string()),
        serde_json::Value::String(value) => Value::Scalar(value),
        serde_json::Value::Array(values) => {
            Value::List(values.into_iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(values) => Value::Dict(
            values
                .into_iter()
                .map(|(key, value)| (key, json_to_value(value)))
                .collect(),
        ),
    }
}

fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Scalar(value) if value == "true" => serde_json::Value::Bool(true),
        Value::Scalar(value) if value == "false" => serde_json::Value::Bool(false),
        Value::Scalar(value) if value.is_empty() => serde_json::Value::Null,
        Value::Scalar(value) => value
            .parse::<serde_json::Number>()
            .map(serde_json::Value::Number)
            .unwrap_or_else(|_| serde_json::Value::String(value.clone())),
        Value::List(values) => serde_json::Value::Array(values.iter().map(value_to_json).collect()),
        Value::Dict(values) => serde_json::Value::Object(
            values
                .iter()
                .map(|(key, value)| (key.clone(), value_to_json(value)))
                .collect(),
        ),
    }
}

fn scalar_list(name: &str, items: &[Value]) -> Result<Vec<String>, String> {
    items
        .iter()
        .map(|item| match item {
            Value::Scalar(value) => Ok(value.clone()),
            Value::List(_) | Value::Dict(_) => Err(format!("{name} requires a scalar list")),
        })
        .collect()
}

fn expect_arg_count(name: &str, args: &[&str], expected: usize) -> Result<(), String> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(format!("{name} expects {expected} argument(s)"))
    }
}

fn expect_min_arg_count(name: &str, args: &[&str], min: usize) -> Result<(), String> {
    if args.len() >= min {
        Ok(())
    } else {
        Err(format!("{name} expects at least {min} argument(s)"))
    }
}

fn split_space_args(text: &str) -> Result<Vec<&str>, String> {
    let mut args = Vec::new();
    let mut start = None;
    let mut bracket = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (i, ch) in text.char_indices() {
        if start.is_none() && !ch.is_whitespace() {
            start = Some(i);
        }
        if in_string {
            if escaped {
                escaped = false
            } else if ch == '\\' {
                escaped = true
            } else if ch == '"' {
                in_string = false
            };
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' => bracket += 1,
            ']' => bracket = bracket.saturating_sub(1),
            c if c.is_whitespace() && bracket == 0 => {
                if let Some(s) = start.take() {
                    args.push(&text[s..i]);
                }
            }
            _ => {}
        }
    }
    if in_string {
        return Err("unterminated string literal".to_string());
    }
    if let Some(s) = start {
        args.push(&text[s..]);
    }
    Ok(args)
}

fn split_top_level_commas(input: &str) -> Result<Vec<&str>, String> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut in_string = false;
    let mut escaped = false;
    let mut square_depth = 0usize;
    let mut curly_depth = 0usize;
    for (i, ch) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '[' if !in_string => square_depth += 1,
            ']' if !in_string => square_depth = square_depth.saturating_sub(1),
            '{' if !in_string => curly_depth += 1,
            '}' if !in_string => curly_depth = curly_depth.saturating_sub(1),
            ',' if !in_string && square_depth == 0 && curly_depth == 0 => {
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

fn split_dict_entry(input: &str) -> Result<(&str, &str), String> {
    let mut in_string = false;
    let mut escaped = false;
    let mut square_depth = 0usize;
    let mut curly_depth = 0usize;
    for (i, ch) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '[' if !in_string => square_depth += 1,
            ']' if !in_string => square_depth = square_depth.saturating_sub(1),
            '{' if !in_string => curly_depth += 1,
            '}' if !in_string => curly_depth = curly_depth.saturating_sub(1),
            ':' if !in_string && square_depth == 0 && curly_depth == 0 => {
                let key = input[..i].trim();
                let value = input[i + 1..].trim();
                if key.is_empty() || value.is_empty() {
                    return Err("malformed dictionary entry".to_string());
                }
                return Ok((key, value));
            }
            _ => {}
        }
    }
    Err("malformed dictionary entry; expected ':'".to_string())
}

fn parse_positive_integer_arg(text: &str, error: &str) -> Result<usize, String> {
    let value = parse_integer_arg(text, error)?;
    usize::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error.to_string())
}

fn parse_nonnegative_integer_arg(text: &str, error: &str) -> Result<usize, String> {
    let value = parse_integer_arg(text, error)?;
    usize::try_from(value).map_err(|_| error.to_string())
}

fn parse_integer_arg(text: &str, error: &str) -> Result<i64, String> {
    if text.contains('.') {
        return Err(error.to_string());
    }
    text.parse::<i64>().map_err(|_| error.to_string())
}

fn path_join(parts: &[String]) -> String {
    let mut path = PathBuf::new();
    for part in parts {
        path.push(part);
    }
    path.to_string_lossy().to_string()
}

fn basename_helper(raw: &str, extension: Option<&str>) -> String {
    let name = Path::new(raw)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let Some(extension) = extension else {
        return name;
    };
    let extension = extension.strip_prefix('.').unwrap_or(extension);
    let suffix = format!(".{extension}");
    name.strip_suffix(&suffix).unwrap_or(&name).to_string()
}

fn path_helper(name: &str, raw: &str) -> String {
    let p = Path::new(raw);
    match name {
        "dirname" => p
            .parent()
            .map(|s| {
                let v = s.to_string_lossy().to_string();
                if v.is_empty() { ".".to_string() } else { v }
            })
            .unwrap_or_else(|| ".".to_string()),
        "extension" => p
            .extension()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
        "withoutExtension" => p
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
        _ => unreachable!(),
    }
}

fn find_command(command: &str) -> Option<PathBuf> {
    if command.trim().is_empty() {
        return None;
    }

    let command_path = Path::new(command);
    if command_path.components().count() > 1 {
        return executable_path(command_path);
    }

    for directory in env::split_paths(&env::var_os("PATH")?) {
        for candidate in command_candidates(&directory, command) {
            if let Some(path) = executable_path(&candidate) {
                return Some(path);
            }
        }
    }
    None
}

fn command_candidates(directory: &Path, command: &str) -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let command_path = Path::new(command);
        if command_path.extension().is_some() {
            return vec![directory.join(command)];
        }
        let pathext = env::var_os("PATHEXT")
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".to_string());
        pathext
            .split(';')
            .filter(|ext| !ext.is_empty())
            .map(|ext| directory.join(format!("{command}{ext}")))
            .collect()
    }
    #[cfg(not(windows))]
    {
        vec![directory.join(command)]
    }
}

fn executable_path(path: &Path) -> Option<PathBuf> {
    let metadata = fs::metadata(path).ok()?;
    if !metadata.is_file() {
        return None;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return None;
        }
    }

    Some(path.to_path_buf())
}

fn random_u64() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or_default();
    let count = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    nanos ^ count.rotate_left(13) ^ (std::process::id() as u64)
}

fn parse_two_string_paths(text: &str, command: &str) -> Result<(String, String), String> {
    let (first_text, tail) = split_string_literal_tail(text.trim())
        .map_err(|_| format!("{command} syntax is {command} \"from\" \"to\""))?;
    let (second_text, extra) = split_string_literal_tail(tail.trim())
        .map_err(|_| format!("{command} syntax is {command} \"from\" \"to\""))?;
    if !extra.trim().is_empty() {
        return Err(format!("{command} syntax is {command} \"from\" \"to\""));
    }
    Ok((
        parse_string_literal(first_text)?,
        parse_string_literal(second_text)?,
    ))
}

fn split_path_expr_tail<'a>(text: &'a str, command: &str) -> Result<(&'a str, &'a str), String> {
    let text = text.trim();
    if text.starts_with('"') {
        return split_string_literal_tail(text);
    }
    let Some((path, tail)) = text.split_once(char::is_whitespace) else {
        return Err(format!("{command} syntax is {command} \"file\" $text"));
    };
    validate_path_expr_form(path, &format!("{command} requires a string file path"))?;
    Ok((path, tail))
}

fn validate_path_expr_form(text: &str, error: &str) -> Result<(), String> {
    let text = text.trim();
    if text.starts_with('"') {
        parse_string_literal(text).map_err(|_| error.to_string())?;
        return Ok(());
    }
    if text.starts_with('$') {
        let (_, _, extra) = parse_identifier(text).map_err(|_| error.to_string())?;
        if extra.trim().is_empty() {
            return Ok(());
        }
        return Err(error.to_string());
    }
    Err(error.to_string())
}

fn looks_like_list_mutation_target(text: &str) -> bool {
    let text = text.trim();
    text.starts_with('$') && text.contains('[') && text.ends_with(']')
}

fn parse_read_capture(text: &str) -> Result<(&str, String), String> {
    let Some((file, destination)) = split_capture_arrow(text) else {
        return Err("read syntax is read \"file\" -> $text".to_string());
    };
    validate_path_expr_form(file, "read requires a string file path")?;
    let (kind, name, extra) = parse_identifier(destination.trim())
        .map_err(|_| "read destination must be a variable".to_string())?;
    if kind != IdentKind::Variable || !extra.trim().is_empty() {
        return Err("read destination must be a variable".to_string());
    }
    Ok((file, name))
}

fn parse_path_capture<'a>(text: &'a str, command: &str) -> Result<(&'a str, String), String> {
    let Some((path, destination)) = split_capture_arrow(text) else {
        return Err(format!("{command} syntax is {command} \"path\" -> $ok"));
    };
    validate_path_expr_form(path, &format!("{command} requires a string path"))?;
    let (kind, name, extra) = parse_identifier(destination.trim())
        .map_err(|_| format!("{command} destination must be a variable"))?;
    if kind != IdentKind::Variable || !extra.trim().is_empty() {
        return Err(format!("{command} destination must be a variable"));
    }
    Ok((path, name))
}

fn unique_temp_path(kind: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let count = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    env::temp_dir().join(format!(
        "shelly-{kind}-{}-{nonce}-{count}",
        std::process::id()
    ))
}

fn copy_recursive(from: &Path, to: &Path) -> Result<(), String> {
    if from.is_dir() {
        fs::create_dir_all(to)
            .map_err(|error| format!("could not create directory '{}': {error}", to.display()))?;
        for entry in fs::read_dir(from)
            .map_err(|error| format!("could not read directory '{}': {error}", from.display()))?
        {
            let entry =
                entry.map_err(|error| format!("could not read directory entry: {error}"))?;
            copy_recursive(&entry.path(), &to.join(entry.file_name()))?;
        }
        Ok(())
    } else if from.is_file() {
        fs::copy(from, to).map(|_| ()).map_err(|error| {
            format!(
                "could not copy '{}' to '{}': {error}",
                from.display(),
                to.display()
            )
        })
    } else {
        Err(format!("source path does not exist: {}", from.display()))
    }
}

fn parse_choose_capture(text: &str) -> Result<(&str, &str, String), String> {
    let (question, tail) = split_string_literal_tail(text).map_err(|_| {
        "choose syntax is choose \"Question\" [\"A\", \"B\"] -> $choice".to_string()
    })?;
    let Some((options_text, destination)) = split_capture_arrow(tail.trim()) else {
        return Err("choose syntax is choose \"Question\" [\"A\", \"B\"] -> $choice".to_string());
    };
    if !options_text.trim_start().starts_with('[') && !options_text.trim_start().starts_with('$') {
        return Err("choose options must be a list".to_string());
    }
    if options_text.trim_start().starts_with('[') {
        Interpreter::default().eval_list(options_text.trim())?;
    }
    let (kind, name, extra) = parse_identifier(destination.trim())
        .map_err(|_| "choose destination must be a variable".to_string())?;
    if kind != IdentKind::Variable || !extra.trim().is_empty() {
        return Err("choose destination must be a variable".to_string());
    }
    Ok((question, options_text, name))
}

fn parse_files_capture(text: &str) -> Result<(&str, String), String> {
    let Some((pattern, destination)) = split_capture_arrow(text) else {
        return Err("files syntax is files \"pattern\" -> $items".to_string());
    };
    parse_string_literal(pattern).map_err(|_| "files requires a string pattern".to_string())?;
    let (kind, name, extra) = parse_identifier(destination.trim())
        .map_err(|_| "files destination must be a variable".to_string())?;
    if kind != IdentKind::Variable || !extra.trim().is_empty() {
        return Err("files destination must be a variable".to_string());
    }
    Ok((pattern, name))
}

fn glob_files(base_dir: &std::path::Path, pattern: &str) -> Result<Vec<String>, String> {
    let segments: Vec<&str> = pattern
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.is_empty() {
        return Ok(Vec::new());
    }
    let mut matches = Vec::new();
    collect_glob_matches(base_dir, &segments, String::new(), &mut matches)?;
    matches.sort();
    Ok(matches)
}

fn collect_glob_matches(
    dir: &std::path::Path,
    segments: &[&str],
    prefix: String,
    matches: &mut Vec<String>,
) -> Result<(), String> {
    let Some((segment, rest)) = segments.split_first() else {
        return Ok(());
    };
    if *segment == "**" {
        // First consume zero directory segments, then recurse into real directories.
        collect_glob_matches(dir, rest, prefix.clone(), matches)?;
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(()),
        };
        for entry in entries {
            let entry =
                entry.map_err(|error| format!("could not read directory entry: {error}"))?;
            let file_type = entry
                .file_type()
                .map_err(|error| format!("could not inspect directory entry: {error}"))?;
            if !file_type.is_dir() {
                continue;
            }
            collect_glob_matches(
                &entry.path(),
                segments,
                join_relative(&prefix, &entry.file_name().to_string_lossy()),
                matches,
            )?;
        }
        return Ok(());
    }
    if segment.contains('*') {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(()),
        };
        for entry in entries {
            let entry =
                entry.map_err(|error| format!("could not read directory entry: {error}"))?;
            let name = entry.file_name().to_string_lossy().to_string();
            if !glob_segment_matches(segment, &name) {
                continue;
            }
            let next_prefix = join_relative(&prefix, &name);
            if rest.is_empty() {
                if entry.path().is_file() {
                    matches.push(next_prefix);
                }
            } else if entry
                .file_type()
                .map(|file_type| file_type.is_dir())
                .unwrap_or(false)
            {
                collect_glob_matches(&entry.path(), rest, next_prefix, matches)?;
            }
        }
        return Ok(());
    }

    let path = dir.join(segment);
    let next_prefix = join_relative(&prefix, segment);
    if rest.is_empty() {
        if path.is_file() {
            matches.push(next_prefix);
        }
    } else if path.is_dir()
        && !path
            .symlink_metadata()
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
    {
        collect_glob_matches(&path, rest, next_prefix, matches)?;
    }
    Ok(())
}

fn join_relative(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}/{name}")
    }
}

fn glob_segment_matches(pattern: &str, text: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == text;
    }
    if pattern == "*" {
        return true;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if !pattern.starts_with('*') && !text.starts_with(parts[0]) {
        return false;
    }
    if !pattern.ends_with('*') && !text.ends_with(parts[parts.len() - 1]) {
        return false;
    }
    let mut rest = text;
    for part in parts.iter().filter(|part| !part.is_empty()) {
        let Some(position) = rest.find(part) else {
            return false;
        };
        rest = &rest[position + part.len()..];
    }
    true
}

fn split_capture_arrow(text: &str) -> Option<(&str, &str)> {
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in text.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
            continue;
        }
        if text[index..].starts_with("->") {
            let left = text[..index].trim();
            let right = text[index + 2..].trim();
            if !left.is_empty() && !right.is_empty() {
                return Some((left, right));
            }
        }
    }
    None
}

fn split_matches_args(args: &str) -> Result<(&str, String), String> {
    let args = args.trim();
    let Some((value, literal)) = args.split_once(char::is_whitespace) else {
        return Err("matches requires a value and /pattern/".to_string());
    };
    let literal = literal.trim();
    if !literal.starts_with('/') {
        return Err("matches requires a slash-delimited /pattern/".to_string());
    }
    let bytes = literal.as_bytes();
    let mut escaped = false;
    for i in 1..bytes.len() {
        if escaped {
            escaped = false;
            continue;
        }
        if bytes[i] == b'\\' {
            escaped = true;
            continue;
        }
        if bytes[i] == b'/' {
            if !literal[i + 1..].trim().is_empty() {
                return Err("unexpected text after regex literal".to_string());
            }
            return Ok((value, literal[1..i].to_string()));
        }
    }
    Err("unterminated regex literal; expected closing '/'".to_string())
}

fn mask_regex_literals(text: &str) -> String {
    let mut out = text.as_bytes().to_vec();
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut escaped = false;
    while i < bytes.len() {
        if bytes[i] == b'/' && !escaped {
            let start = i;
            i += 1;
            let mut e = false;
            while i < bytes.len() {
                if !e && bytes[i] == b'/' {
                    i += 1;
                    break;
                }
                e = !e && bytes[i] == b'\\';
                if e && i + 1 < bytes.len() {
                    i += 1;
                    e = false;
                } else {
                    i += 1;
                }
            }
            for byte in &mut out[start..i] {
                if *byte != b' ' {
                    *byte = b' ';
                }
            }
            escaped = false;
            continue;
        }
        escaped = bytes[i] == b'\\' && !escaped;
        i += 1;
    }
    String::from_utf8(out).expect("mask preserves UTF-8")
}

fn split_predicate_two_args(args: &str) -> Option<(&str, &str)> {
    let mut bracket_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in args.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ch if bracket_depth == 0 && ch.is_whitespace() => {
                let left = args[..index].trim();
                let right = args[index..].trim();
                if !left.is_empty() && !right.is_empty() {
                    return Some((left, right));
                }
            }
            _ => {}
        }
    }
    None
}

fn validate_condition_parentheses(condition: &str) -> Result<(), String> {
    let scan = mask_regex_literals(condition);
    let condition = scan.as_str();
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut previous_open: Option<usize> = None;
    for (index, ch) in condition.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '(' if bracket_depth == 0 => {
                paren_depth += 1;
                previous_open = Some(index);
            }
            ')' if bracket_depth == 0 => {
                if paren_depth == 0 {
                    return Err("unexpected ')' in condition".to_string());
                }
                if let Some(open) = previous_open {
                    if condition[open + 1..index].trim().is_empty() {
                        return Err("empty parenthesized condition".to_string());
                    }
                }
                paren_depth -= 1;
                previous_open = None;
            }
            _ => {}
        }
    }
    if paren_depth > 0 {
        return Err("missing ')' in condition".to_string());
    }
    Ok(())
}

fn strip_grouped_condition(condition: &str) -> Result<Option<&str>, String> {
    let condition = condition.trim();
    if !condition.starts_with('(') {
        return Ok(None);
    }
    let scan = mask_regex_literals(condition);
    let condition_scan = scan.as_str();
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in condition_scan.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '(' if bracket_depth == 0 => paren_depth += 1,
            ')' if bracket_depth == 0 => {
                if paren_depth == 0 {
                    return Err("unexpected ')' in condition".to_string());
                }
                paren_depth -= 1;
                if paren_depth == 0 {
                    if !condition[index + 1..].trim().is_empty() {
                        return Ok(None);
                    }
                    let inner = condition[1..index].trim();
                    if inner.is_empty() {
                        return Err("empty parenthesized condition".to_string());
                    }
                    return Ok(Some(inner));
                }
            }
            _ => {}
        }
    }
    Err("missing ')' in condition".to_string())
}

fn split_condition_keyword<'a>(condition: &'a str, keyword: &str) -> Option<(&'a str, &'a str)> {
    let scan = mask_regex_literals(condition);
    let condition_scan = scan.as_str();
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in condition_scan.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '(' if bracket_depth == 0 => paren_depth += 1,
            ')' if bracket_depth == 0 => paren_depth = paren_depth.saturating_sub(1),
            _ if bracket_depth == 0 && paren_depth == 0 => {
                let tail = &condition_scan[index..];
                if tail.starts_with(keyword) {
                    let before = condition_scan[..index].chars().next_back();
                    let after = condition_scan[index + keyword.len()..].chars().next();
                    let before_ok = before.is_none_or(char::is_whitespace);
                    let after_ok = after.is_none_or(char::is_whitespace);
                    if before_ok && after_ok {
                        return Some((
                            condition[..index].trim(),
                            condition[index + keyword.len()..].trim(),
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    None
}

fn split_condition_operator(condition: &str) -> Option<(&str, &str, &str)> {
    let scan = mask_regex_literals(condition);
    let condition_scan = scan.as_str();
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in condition_scan.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '(' if bracket_depth == 0 => paren_depth += 1,
            ')' if bracket_depth == 0 => paren_depth = paren_depth.saturating_sub(1),
            _ if bracket_depth == 0 && paren_depth == 0 => {
                let tail = &condition_scan[index..];
                for operator in ["==", "!=", ">=", "<=", ">", "<"] {
                    if tail.starts_with(operator) {
                        let right_start = index + operator.len();
                        return Some((&condition[..index], operator, &condition[right_start..]));
                    }
                }
            }
            _ => {}
        }
    }

    None
}

fn parse_for_header(rest: &str) -> Result<(String, String), String> {
    let Some((destination, iterable)) = rest.split_once(" in ") else {
        return Err("for syntax is for $item in $list".to_string());
    };
    let (kind, item_name, extra) = parse_identifier(destination.trim())
        .map_err(|_| "for item destination must be a variable".to_string())?;
    if kind != IdentKind::Variable || !extra.trim().is_empty() {
        return Err("for item destination must be a variable".to_string());
    }
    let iterable = iterable.trim();
    if iterable.is_empty() || !iterable.starts_with('$') {
        return Err("for syntax is for $item in $list".to_string());
    }
    Ok((item_name, iterable.to_string()))
}

fn parse_repeat_header(rest: &str) -> Result<(&str, Option<String>), String> {
    let (count_expr, destination) = match rest.split_once("->") {
        Some((count_expr, destination)) => (count_expr.trim(), Some(destination.trim())),
        None => (rest.trim(), None),
    };
    if count_expr.is_empty() {
        return Err("repeat count required".to_string());
    }
    let index_var = if let Some(destination) = destination {
        let (kind, name, extra) = parse_identifier(destination)
            .map_err(|_| "repeat index destination must be a variable".to_string())?;
        if kind != IdentKind::Variable || !extra.trim().is_empty() {
            return Err("repeat index destination must be a variable".to_string());
        }
        Some(name)
    } else {
        None
    };
    Ok((count_expr, index_var))
}

fn validate_log_header(rest: &str) -> Result<(), String> {
    if rest.trim().is_empty() {
        return Err("log identifier required".to_string());
    }
    if rest.trim().starts_with('"') {
        parse_string_literal(rest.trim())?;
    }
    Ok(())
}

fn sanitize_log_identifier(identifier: &str) -> String {
    let sanitized: String = identifier
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '-'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "log".to_string()
    } else {
        sanitized
    }
}

fn current_utc_parts() -> (i32, u32, u32, u32, u32, u32) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    (
        year,
        month,
        day,
        (rem / 3600) as u32,
        ((rem % 3600) / 60) as u32,
        (rem % 60) as u32,
    )
}

fn current_utc_date() -> String {
    let (y, m, d, _, _, _) = current_utc_parts();
    format!("{y:04}-{m:02}-{d:02}")
}

fn current_utc_timestamp() -> String {
    let (y, m, d, h, min, s) = current_utc_parts();
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{min:02}:{s:02}Z")
}

fn parse_utc_timestamp(text: &str) -> Result<(i32, u32, u32, u32, u32, u32), String> {
    let bytes = text.as_bytes();
    if bytes.len() != 20
        || [4, 7, 10, 13, 16, 19]
            .iter()
            .any(|&i| !matches!(bytes.get(i), Some(b'-' | b'T' | b':' | b'Z')))
    {
        return Err("formatDate requires UTC ISO timestamp YYYY-MM-DDTHH:mm:ssZ".to_string());
    }
    let number = |a, b| {
        text[a..b]
            .parse::<u32>()
            .map_err(|_| "formatDate timestamp contains invalid digits".to_string())
    };
    let y = text[0..4]
        .parse::<i32>()
        .map_err(|_| "formatDate timestamp contains invalid year".to_string())?;
    let m = number(5, 7)?;
    let d = number(8, 10)?;
    let h = number(11, 13)?;
    let min = number(14, 16)?;
    let s = number(17, 19)?;
    if !(1..=12).contains(&m)
        || !(1..=31).contains(&d)
        || h > 23
        || min > 59
        || s > 59
        || civil_from_days(days_from_civil(y, m, d)) != (y, m, d)
    {
        return Err("formatDate timestamp contains an invalid UTC date or time".to_string());
    }
    Ok((y, m, d, h, min, s))
}

fn format_date_helper(timestamp: &str, pattern: &str) -> Result<String, String> {
    let (y, m, d, h, min, s) = parse_utc_timestamp(timestamp)?;
    let values = [
        ("YYYY", format!("{y:04}")),
        ("MM", format!("{m:02}")),
        ("DD", format!("{d:02}")),
        ("HH", format!("{h:02}")),
        ("mm", format!("{min:02}")),
        ("ss", format!("{s:02}")),
    ];
    let mut result = String::new();
    let mut rest = pattern;
    while !rest.is_empty() {
        if let Some((token, value)) = values.iter().find(|(token, _)| rest.starts_with(token)) {
            result.push_str(value);
            rest = &rest[token.len()..];
        } else {
            let ch = rest.chars().next().unwrap();
            if ch.is_ascii_alphabetic() {
                return Err(format!(
                    "formatDate unsupported pattern token near '{rest}'; use YYYY, MM, DD, HH, mm, or ss"
                ));
            }
            result.push(ch);
            rest = &rest[ch.len_utf8()..];
        }
    }
    Ok(result)
}

fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = y as i64 - if m <= 2 { 1 } else { 0 };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = m as i64 + if m > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    ((y + if m <= 2 { 1 } else { 0 }) as i32, m as u32, d as u32)
}

fn control_signal_name(error: &str) -> &'static str {
    if parse_exit_signal(error).is_some() {
        "exit"
    } else if parse_return_signal(error).is_some() {
        "return"
    } else if is_break_signal(error) {
        "break"
    } else {
        "continue"
    }
}

fn validate_case_value(value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err("case value required".to_string());
    }
    Ok(())
}

fn split_when_values(values: &str) -> Result<Vec<&str>, String> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0i32;
    let mut in_string = false;
    let bytes = values.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'"' if index == 0 || bytes[index - 1] != b'\\' => in_string = !in_string,
            b'[' | b'{' | b'(' if !in_string => depth += 1,
            b']' | b'}' | b')' if !in_string => depth -= 1,
            b',' if !in_string && depth == 0 => {
                return Err(
                    "comma-separated when alternatives are unsupported; use `or`".to_string(),
                );
            }
            b'o' if !in_string
                && depth == 0
                && values[index..].starts_with("or")
                && (index == 0 || bytes[index - 1].is_ascii_whitespace())
                && (index + 2 == bytes.len() || bytes[index + 2].is_ascii_whitespace()) =>
            {
                if values[start..index].trim().is_empty() {
                    return Err("when value required".to_string());
                }
                parts.push(values[start..index].trim());
                start = index + 2;
                index += 1;
            }
            _ => {}
        }
        index += 1;
    }
    if in_string || depth != 0 {
        return Err("when value required".to_string());
    }
    if values[start..].trim().is_empty() {
        return Err("when value required".to_string());
    }
    parts.push(values[start..].trim());
    Ok(parts)
}

fn validate_when_values(values: &str) -> Result<(), String> {
    if values.trim().is_empty() {
        return Err("when value required".to_string());
    }
    split_when_values(values).map(|_| ())
}

fn collect_block(
    lines: &[(usize, String)],
    start: usize,
    kind: &str,
) -> Result<(Vec<(usize, String)>, usize), String> {
    let mut body = Vec::new();
    let mut depth = 0;
    let mut index = start + 1;
    while index < lines.len() {
        let line = lines[index].1.trim();
        if starts_block(line) {
            depth += 1;
        }
        if line == "}" && depth > 0 {
            depth -= 1;
        }
        if line == "end" {
            if depth == 0 {
                return Ok((body, index + 1));
            }
            depth -= 1;
        }
        if !line.is_empty() {
            body.push(lines[index].clone());
        }
        index += 1;
    }
    Err(format!("missing end for {kind}"))
}

fn collect_case_block(
    lines: &[(usize, String)],
    start: usize,
) -> Result<
    (
        Vec<(Vec<String>, Vec<(usize, String)>)>,
        Option<Vec<(usize, String)>>,
        usize,
    ),
    String,
> {
    let mut branches: Vec<(Vec<String>, Vec<(usize, String)>)> = Vec::new();
    let mut else_body: Option<Vec<(usize, String)>> = None;
    let mut depth = 0;
    let mut index = start + 1;
    while index < lines.len() {
        let line_no = lines[index].0;
        let line = lines[index].1.trim();
        if starts_block(line) {
            depth += 1;
        }
        if line == "}" && depth > 0 {
            depth -= 1;
        }
        if line == "end" {
            if depth == 0 {
                if branches.is_empty() {
                    return Err("case requires at least one when".to_string());
                }
                return Ok((branches, else_body, index + 1));
            }
            depth -= 1;
        }
        if depth == 0 {
            if let Some(values) = line.strip_prefix("when ") {
                if else_body.is_some() {
                    return Err(format!("line {line_no}: when after else"));
                }
                validate_when_values(values.trim())
                    .map_err(|error| format!("line {line_no}: {error}"))?;
                branches.push((
                    split_when_values(values.trim())?
                        .into_iter()
                        .map(|value| value.to_string())
                        .collect(),
                    Vec::new(),
                ));
                index += 1;
                continue;
            }
            if line == "when" {
                return Err(format!("line {line_no}: when value required"));
            }
            if line == "else" {
                if else_body.is_some() {
                    return Err(format!("line {line_no}: duplicate else"));
                }
                else_body = Some(Vec::new());
                index += 1;
                continue;
            }
        }
        if let Some(body) = else_body.as_mut() {
            body.push(lines[index].clone());
        } else if let Some((_, body)) = branches.last_mut() {
            body.push(lines[index].clone());
        } else if !line.is_empty() {
            return Err(format!("line {line_no}: case requires a when before body"));
        }
        index += 1;
    }
    Err("missing end for case".to_string())
}

fn starts_block(line: &str) -> bool {
    line.starts_with("case ")
        || line == "defer"
        || line.starts_with("if ")
        || line.starts_with("while ")
        || line.starts_with("for ")
        || line.starts_with("repeat ")
        || line.starts_with("timeout ")
        || line.starts_with("watch ")
        || line.starts_with("fn ")
        || line.starts_with("export fn ")
        || line == "try"
        || line.starts_with("try ")
        || line.starts_with("log ")
}

fn parse_try_header(line: &str) -> Result<Option<String>, String> {
    if line == "try" {
        return Ok(None);
    }
    let Some(rest) = line.strip_prefix("try ") else {
        return Err("try syntax is try or try -> $error".to_string());
    };
    let Some(destination) = rest.trim().strip_prefix("->") else {
        return Err("try syntax is try or try -> $error".to_string());
    };
    let (kind, name, extra) = parse_identifier(destination.trim())
        .map_err(|_| "try error destination must be a variable".to_string())?;
    if kind != IdentKind::Variable || !extra.trim().is_empty() {
        return Err("try error destination must be a variable".to_string());
    }
    Ok(Some(name))
}

fn collect_try_block(
    lines: &[(usize, String)],
    start: usize,
) -> Result<(Vec<(usize, String)>, Vec<(usize, String)>, usize), String> {
    let mut try_body = Vec::new();
    let mut catch_body = Vec::new();
    let mut in_catch = false;
    let mut depth = 0;
    let mut index = start + 1;
    while index < lines.len() {
        let line_no = lines[index].0;
        let line = lines[index].1.trim();
        if starts_block(line) {
            depth += 1;
        }
        if line == "}" && depth > 0 {
            depth -= 1;
        }
        if line == "end" {
            if depth == 0 {
                if !in_catch {
                    return Err("missing catch for try".to_string());
                }
                return Ok((try_body, catch_body, index + 1));
            }
            depth -= 1;
        } else if line == "catch" && depth == 0 {
            if in_catch {
                return Err(format!("line {line_no}: duplicate catch"));
            }
            in_catch = true;
            index += 1;
            continue;
        }
        if !line.is_empty() {
            if in_catch {
                catch_body.push(lines[index].clone());
            } else {
                try_body.push(lines[index].clone());
            }
        }
        index += 1;
    }
    if in_catch {
        Err("missing end for try".to_string())
    } else {
        Err("missing catch for try".to_string())
    }
}

fn parse_function_header(rest: &str) -> Result<(String, Vec<Param>), String> {
    let Some(open) = rest.find('(') else {
        return Err("old function syntax was removed; use fn name($arg) {".to_string());
    };
    let name = rest[..open].trim();
    if !is_identifier_name(name) {
        return Err("function name must start with an ASCII letter".to_string());
    }
    let after_open = &rest[open + 1..];
    let Some(close_rel) = after_open.rfind(')') else {
        return Err("function parameters must be enclosed in parentheses".to_string());
    };
    let params_text = &after_open[..close_rel];
    let tail = after_open[close_rel + 1..].trim();
    if tail != "{" {
        return Err("function definitions must end with {".to_string());
    }
    let mut params = Vec::new();
    let mut saw_default = false;
    if !params_text.trim().is_empty() {
        for item in split_commas(params_text.trim())? {
            let (left, default) = match item.split_once('=') {
                Some((left, right)) => (left.trim(), Some(right.trim())),
                None => (item.trim(), None),
            };
            if default.is_none() && saw_default {
                return Err("required parameter cannot follow default".to_string());
            }
            let (kind, name, extra) = parse_identifier(left)?;
            if kind != IdentKind::Variable || !extra.trim().is_empty() {
                return Err("function parameters must be variables".to_string());
            }
            let default = if let Some(value) = default {
                saw_default = true;
                Some(Interpreter::default().eval_value(value)?)
            } else {
                None
            };
            params.push(Param { name, default });
        }
    }
    Ok((name.to_string(), params))
}

fn parse_mock_declaration(line: &str) -> Result<(String, mock::Mock), String> {
    let rest = line
        .strip_prefix("mock ")
        .ok_or("expected mock declaration")?
        .trim();
    let (kind, rest) = rest.split_once(' ').ok_or("mock kind required")?;
    match kind {
        "http" => {
            let (method, rest) = rest.trim().split_once(' ').ok_or("HTTP method required")?;
            if !matches!(
                method,
                "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS"
            ) {
                return Err("unsupported HTTP method".to_string());
            }
            let (target, dict) = parser::split_string_literal_tail(rest.trim())?;
            let target = parser::parse_string_literal(target)?;
            if target.is_empty() || !target.starts_with("http") {
                return Err("HTTP target must be a literal URL".to_string());
            }
            let dict = parse_mock_dict(dict, &["status", "body"])?;
            let status = dict_value(&dict, "status")?;
            if !status.chars().all(|c| c.is_ascii_digit()) {
                return Err("status must be a number".to_string());
            }
            Ok((
                format!("http:{method}:{target}"),
                mock::Mock::Http {
                    method: method.to_string(),
                    url: target,
                    status: status.parse().unwrap(),
                    body: dict_value(&dict, "body")?,
                },
            ))
        }
        "read" => {
            let (target, tail) = parser::split_string_literal_tail(rest)?;
            if !tail.trim().starts_with("returns") {
                return Err("read mock requires returns".to_string());
            }
            let path = parser::parse_string_literal(target)?;
            let value =
                parser::parse_string_literal(tail.trim().strip_prefix("returns").unwrap().trim())?;
            Ok((
                format!("read:{}", normalize_mock_path(&path)),
                mock::Mock::Read { content: value },
            ))
        }
        "command" => {
            let (before, dict) = rest
                .split_once(" returns ")
                .ok_or("command mock requires returns")?;
            let atoms = literal_atoms(before)?;
            if atoms.is_empty() {
                return Err("command program required".to_string());
            }
            let fields = parse_mock_dict(dict, &["code", "stdout", "stderr"])?;
            let code = dict_value(&fields, "code")?;
            if !code.chars().all(|c| c.is_ascii_digit()) {
                return Err("code must be a number".to_string());
            }
            let key = format!("command:{}", atoms.join("\u{1f}"));
            Ok((
                key,
                mock::Mock::Command {
                    program: atoms[0].clone(),
                    args: atoms[1..].to_vec(),
                    code: code.parse().unwrap(),
                    stdout: dict_value(&fields, "stdout")?,
                    stderr: dict_value(&fields, "stderr")?,
                },
            ))
        }
        _ => Err("unsupported mock kind".to_string()),
    }
}

fn literal_atoms(input: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut rest = input.trim();
    while !rest.is_empty() {
        if rest.starts_with('"') {
            let (lit, tail) = parser::split_string_literal_tail(rest)?;
            out.push(parser::parse_string_literal(lit)?);
            rest = tail.trim();
        } else {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            let atom = &rest[..end];
            if atom.starts_with('$') {
                return Err("mock targets must be literals".to_string());
            }
            out.push(atom.to_string());
            rest = rest[end..].trim();
        }
    }
    Ok(out)
}

fn parse_mock_dict(input: &str, expected: &[&str]) -> Result<HashMap<String, String>, String> {
    let text = input.trim();
    if !text.starts_with('{') || !text.ends_with('}') {
        return Err("mock returns requires a dictionary".to_string());
    }
    let mut result = HashMap::new();
    for part in parser::split_commas(&text[1..text.len() - 1])? {
        let (key, value) = part.split_once(':').ok_or("malformed mock dictionary")?;
        let key = key.trim();
        if !expected.contains(&key) || result.contains_key(key) {
            return Err("malformed mock dictionary".to_string());
        }
        let value = value.trim();
        let valid = key == "status" || key == "code";
        if valid {
            if !parser::is_number_literal(value) {
                return Err("mock status/code must be a number".to_string());
            }
            result.insert(key.to_string(), value.to_string());
        } else {
            result.insert(key.to_string(), parser::parse_string_literal(value)?);
        }
    }
    if result.len() != expected.len() {
        return Err("malformed mock dictionary".to_string());
    }
    Ok(result)
}
fn dict_value(dict: &HashMap<String, String>, key: &str) -> Result<String, String> {
    dict.get(key)
        .cloned()
        .ok_or_else(|| "malformed mock dictionary".to_string())
}

fn normalize_mock_path(path: &str) -> String {
    let mut parts = Vec::new();
    for part in Path::new(path).components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::Normal(value) => parts.push(value.to_string_lossy().into_owned()),
            std::path::Component::RootDir => parts.clear(),
            _ => {}
        }
    }
    parts.join("/")
}

fn parse_call_syntax(text: &str) -> Result<(&str, Vec<&str>), String> {
    let Some(open) = text.find('(') else {
        let text = text.trim();
        let mut words = text.split_whitespace();
        let name = words.next().unwrap_or("");
        if is_identifier_name(name) {
            if words.next().is_none() {
                return Ok((name, Vec::new()));
            }
            return Err("old call syntax was removed; use call name(args)".to_string());
        }
        return Err("call requires a function name".to_string());
    };
    let name = text[..open].trim();
    if name.is_empty() {
        return Err("call requires a function name".to_string());
    }
    if !is_identifier_name(name) {
        return Err("function name must start with an ASCII letter".to_string());
    }
    let after_open = &text[open + 1..];
    let Some(close_rel) = after_open.rfind(')') else {
        return Err("call arguments must be enclosed in parentheses".to_string());
    };
    let args_text = &after_open[..close_rel];
    if !after_open[close_rel + 1..].trim().is_empty() {
        return Err("unexpected text after call arguments".to_string());
    }
    let args = if args_text.trim().is_empty() {
        Vec::new()
    } else {
        split_commas(args_text.trim())?
    };
    Ok((name, args))
}

fn validate_call_syntax(text: &str) -> Result<(), String> {
    let (call_text, destination) = match text.split_once("->") {
        Some((call_text, destination)) => (call_text.trim(), Some(destination.trim())),
        None => (text.trim(), None),
    };
    parse_call_syntax(call_text)?;
    if let Some(destination) = destination {
        let (kind, _, extra) = parse_identifier(destination)
            .map_err(|_| "function return destination must be a variable".to_string())?;
        if kind != IdentKind::Variable || !extra.trim().is_empty() {
            return Err("function return destination must be a variable".to_string());
        }
    }
    Ok(())
}

fn parse_function(
    rest: &str,
    lines: &[(usize, String)],
    start: usize,
) -> Result<(Function, usize), String> {
    let (name, params) = parse_function_header(rest)?;
    let mut body = Vec::new();
    let mut depth = 0;
    let mut index = start + 1;
    while index < lines.len() {
        let line_no = lines[index].0;
        let line = lines[index].1.trim();
        if starts_block(line) {
            depth += 1;
        }
        if line == "}" {
            if depth == 0 {
                return Ok((Function { name, params, body }, index + 1));
            }
            depth -= 1;
        }
        if line == "end" {
            if depth == 0 {
                return Err("old function syntax was removed; close functions with }".to_string());
            }
            depth -= 1;
        }
        if !line.is_empty() {
            body.push((line_no, line.to_string()));
        }
        index += 1;
    }
    Err(format!("missing }} for function {name}"))
}

struct NumericParser<'a> {
    input: &'a str,
    pos: usize,
    interpreter: &'a Interpreter,
}

impl<'a> NumericParser<'a> {
    fn new(input: &'a str, interpreter: &'a Interpreter) -> Self {
        Self {
            input,
            pos: 0,
            interpreter,
        }
    }

    fn parse_expr(&mut self) -> Result<f64, String> {
        let mut value = self.parse_term()?;
        loop {
            self.skip_ws();
            if self.consume('+') {
                value += self.parse_term()?;
            } else if self.consume('-') {
                value -= self.parse_term()?;
            } else {
                return Ok(value);
            }
        }
    }

    fn parse_term(&mut self) -> Result<f64, String> {
        let mut value = self.parse_factor()?;
        loop {
            self.skip_ws();
            if self.consume('*') {
                value *= self.parse_factor()?;
            } else if self.consume('/') {
                let divisor = self.parse_factor()?;
                if divisor == 0.0 {
                    return Err("division by zero".to_string());
                }
                value /= divisor;
            } else {
                return Ok(value);
            }
        }
    }

    fn parse_factor(&mut self) -> Result<f64, String> {
        self.skip_ws();
        if self.consume('-') {
            return Ok(-self.parse_factor()?);
        }
        if self.consume('+') {
            return self.parse_factor();
        }
        if self.consume('(') {
            let value = self.parse_expr()?;
            self.skip_ws();
            if !self.consume(')') {
                return Err("missing ')' in numeric expression".to_string());
            }
            return Ok(value);
        }
        if self.peek() == Some('$') {
            let tail = &self.input[self.pos..];
            let (kind, name, rest) = parse_identifier(tail)?;
            self.pos = self.input.len() - rest.len();
            let value = self.interpreter.lookup(kind, &name)?;
            return value.parse::<f64>().map_err(|_| {
                format!(
                    "numeric operand expected for {}",
                    display_identifier(kind, &name)
                )
            });
        }
        self.parse_number()
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        self.skip_ws();
        let start = self.pos;
        let mut saw_digit = false;
        while matches!(self.peek(), Some(ch) if ch.is_ascii_digit()) {
            saw_digit = true;
            self.advance();
        }
        if self.peek() == Some('.') {
            self.advance();
            while matches!(self.peek(), Some(ch) if ch.is_ascii_digit()) {
                saw_digit = true;
                self.advance();
            }
        }
        if !saw_digit {
            return Err("expected numeric operand".to_string());
        }
        self.input[start..self.pos]
            .parse::<f64>()
            .map_err(|_| "expected numeric operand".to_string())
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_whitespace()) {
            self.advance();
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) {
        if let Some(ch) = self.peek() {
            self.pos += ch.len_utf8();
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn remaining(&self) -> &str {
        &self.input[self.pos..]
    }
}

fn looks_like_numeric_expr(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.starts_with('$')
        || trimmed.starts_with('-')
        || trimmed.starts_with('+')
        || trimmed.starts_with('(')
        || trimmed
            .chars()
            .any(|ch| matches!(ch, '+' | '-' | '*' | '/' | '(' | ')' | '.'))
}

fn parse_filter(selector: &str) -> Option<(&'static str, &str)> {
    for op in ["==", "!=", ">=", "<=", ">", "<"] {
        if let Some(rhs) = selector.strip_prefix(op) {
            return Some((op, rhs.trim()));
        }
    }
    None
}
