use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

mod comments;
mod parser;
mod shell;
mod value;

use comments::strip_comments;
use parser::{
    is_identifier_name, is_number_literal, parse_identifier, parse_string_literal, parse_var_ref,
    split_commas, split_string_literal_tail,
};
use shell::{read_stdin_line, shell_command};
use value::{
    IdentKind, Value, compare_values, display_identifier, format_list, format_number, is_truthy,
};

const WHILE_ITERATION_LIMIT: usize = 10_000;
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn main() {
    let mut args = env::args_os().skip(1);
    let result = match args.next() {
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
                Ok(None) => Err("Usage: shelly check [--format json] <script.shelly>".to_string()),
                Err(error) => Err(error),
            }
        }
        Some(path) => run_script(PathBuf::from(path)),
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
            eprintln!("{error}");
        }
        std::process::exit(1);
    }
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
struct Diagnostic {
    file: String,
    line: usize,
    column: usize,
    severity: &'static str,
    message: String,
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
            eprintln!("line {}: {}", diagnostic.line, diagnostic.message);
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

fn check_source(source: &str, file: &str) -> Vec<Diagnostic> {
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
            let mut checker = Checker::new(file.to_string());
            checker.check_lines(&lines);
            checker.diagnostics
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

fn run_script(path: PathBuf) -> Result<(), String> {
    let path = path
        .canonicalize()
        .map_err(|error| format!("could not read script '{}': {error}", path.display()))?;
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("could not read script '{}': {error}", path.display()))?;
    let mut interpreter = Interpreter::default();
    interpreter.current_dir = path.parent().map(PathBuf::from).unwrap_or_default();
    interpreter.stack.push(path);
    interpreter.run(&source)
}

#[derive(Default)]
struct Interpreter {
    vars: HashMap<String, Value>,
    consts: HashMap<String, Value>,
    functions: HashMap<String, Function>,
    current_dir: PathBuf,
    stack: Vec<PathBuf>,
    imported: HashSet<PathBuf>,
    logs: Vec<LogContext>,
}

#[derive(Clone)]
struct LogContext {
    path: PathBuf,
}

impl Interpreter {
    fn run(&mut self, source: &str) -> Result<(), String> {
        let source = normalize_multiline_strings(source)?;
        let lines = strip_comments(&source)?;
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
        let mut index = 0;
        while index < lines.len() {
            let line_no = lines[index].0;
            let line = lines[index].1.trim();
            if line.is_empty() || line.starts_with('#') {
                index += 1;
                continue;
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
            if matches!(line, "elif" | "else" | "catch" | "end") || line.starts_with("elif ") {
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
            Value::Scalar(_) => return Err("for iterable must be a list".to_string()),
        };

        for value in values {
            self.vars.insert(item_name.clone(), Value::Scalar(value));
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
                    self.vars.insert(name, Value::Scalar(error));
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

    fn log_path(&self, identifier: &str) -> Result<PathBuf, String> {
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

    fn log_action(&self, message: &str) {
        for ctx in &self.logs {
            let _ = self.write_log(ctx, message);
        }
    }

    fn write_log(&self, ctx: &LogContext, message: &str) -> Result<(), String> {
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

    fn run_line(&mut self, line: &str, line_no: usize) -> Result<(), String> {
        if let Some(rest) = line.strip_prefix("import ") {
            return self.import_file(rest.trim(), line_no);
        }
        if let Some(rest) = line.strip_prefix("include ") {
            return self.include_file(rest.trim(), line_no);
        }
        if let Some(rest) = line.strip_prefix("append ") {
            let rest = rest.trim();
            if rest.trim_start().starts_with('"') || !rest.contains(',') {
                if let Ok((path_text, _)) = split_path_expr_tail(rest, "append") {
                    if matches!(self.eval_expr_value(path_text), Ok(Value::List(_))) {
                        return self.push_list_item(rest, true);
                    }
                }
                self.log_action(&format!("append: {}", rest));
                return self.append_file_command(rest);
            }
            return self.push_list_item(rest, true);
        }
        if let Some(rest) = line.strip_prefix("prepend ") {
            return self.push_list_item(rest.trim(), false);
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
            if rest.trim_start().starts_with('"') || !looks_like_list_mutation_target(rest) {
                self.log_action(&format!("remove: {}", rest));
                return self.remove_path(rest);
            }
            return self.remove_list_item(rest);
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
        if let Some(rest) = line.strip_prefix("shellout") {
            let value = self.eval_expr(rest.trim())?;
            self.log_action(&format!("shellout: {value}"));
            println!("{value}");
            return Ok(());
        }
        if let Some(rest) = line.strip_prefix("shellin") {
            let name = parse_var_ref(rest.trim())?;
            let input = read_stdin_line()?;
            self.vars.insert(name, Value::Scalar(input));
            return Ok(());
        }
        if let Some(rest) = line.strip_prefix("ask") {
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
            println!("{question}");
            for (index, option) in options.iter().enumerate() {
                println!("{}) {option}", index + 1);
            }
            loop {
                print!("Choose: ");
                io::stdout()
                    .flush()
                    .map_err(|error| format!("failed to flush stdout: {error}"))?;
                let input = read_stdin_line()?;
                if let Ok(index) = input.parse::<usize>() {
                    if (1..=options.len()).contains(&index) {
                        self.vars
                            .insert(destination, Value::Scalar(options[index - 1].clone()));
                        return Ok(());
                    }
                }
                if let Some(option) = options.iter().find(|option| *option == &input) {
                    self.vars.insert(destination, Value::Scalar(option.clone()));
                    return Ok(());
                }
            }
        }
        if let Some(rest) = line.strip_prefix("exists ") {
            return self.capture_predicate(rest.trim(), |this, args| this.eval_exists(args));
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
            let (command_text, tail) = split_string_literal_tail(rest.trim())?;
            let command = self.eval_string(command_text)?;
            let tail = tail.trim();
            if tail.is_empty() {
                self.log_action(&format!("command: {command}"));
                let status = shell_command(&command)
                    .status()
                    .map_err(|error| format!("failed to run command: {error}"))?;
                if !status.success() {
                    return Err(format!("command exited with status {status}"));
                }
                return Ok(());
            }
            let Some(destination) = tail.strip_prefix("->") else {
                return Err("command capture syntax is command \"...\" -> $var".to_string());
            };
            let (kind, name, extra) = parse_identifier(destination.trim())
                .map_err(|_| "command capture destination must be a variable".to_string())?;
            if kind != IdentKind::Variable || !extra.trim().is_empty() {
                return Err("command capture destination must be a variable".to_string());
            }
            self.log_action(&format!("command: {command}"));
            let output = shell_command(&command)
                .stdout(Stdio::piped())
                .output()
                .map_err(|error| format!("failed to capture command: {error}"))?;
            if !output.status.success() {
                return Err(format!("command exited with status {}", output.status));
            }
            let value = String::from_utf8(output.stdout)
                .map_err(|error| format!("command output was not UTF-8: {error}"))?
                .trim_end_matches(['\r', '\n'])
                .to_string();
            self.vars.insert(name, Value::Scalar(value));
            return Ok(());
        }
        if line.starts_with("capture") {
            return Err("capture syntax was removed; use command \"...\" -> $var".to_string());
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
                Value::List(_) => Err(error.to_string()),
            };
        }
        self.eval_expr(text).map_err(|_| error.to_string())
    }

    fn create_path(&self, text: &str) -> Result<(), String> {
        let raw = self.eval_path_expr(text, "create requires a string path")?;
        let path = self.resolve_user_path(&raw);
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

    fn copy_path(&self, text: &str) -> Result<(), String> {
        let (from, to) = parse_two_string_paths(text, "copy")?;
        let from = self.resolve_user_path(&from);
        let to = self.resolve_user_path(&to);
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

    fn move_path(&self, text: &str) -> Result<(), String> {
        let (from, to) = parse_two_string_paths(text, "move")?;
        let from = self.resolve_user_path(&from);
        let to = self.resolve_user_path(&to);
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

    fn rename_path(&self, text: &str) -> Result<(), String> {
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

    fn remove_path(&self, text: &str) -> Result<(), String> {
        let raw = self.eval_path_expr(text, "remove requires a string path")?;
        let path = self.resolve_user_path(&raw);
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
        let path = self.resolve_user_path(&raw);
        if path.is_dir() {
            return Err(format!("read path is a directory: {}", path.display()));
        }
        let content = fs::read_to_string(&path)
            .map_err(|error| format!("could not read file '{}': {error}", path.display()))?;
        self.vars.insert(destination, Value::Scalar(content));
        Ok(())
    }

    fn append_file_command(&self, text: &str) -> Result<(), String> {
        let (file_text, value_text) = split_path_expr_tail(text, "append")?;
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

    fn write_file_command(&self, text: &str) -> Result<(), String> {
        let (file_text, value_text) = split_path_expr_tail(text, "write")?;
        if value_text.trim().is_empty() {
            return Err("write syntax is write \"file\" $text".to_string());
        }
        let raw = self.eval_path_expr(file_text, "write requires a string file path")?;
        let path = self.resolve_user_path(&raw);
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
                        .ok_or_else(|| format!("undefined global constant $${name}"))?;
                    self.consts.insert(name, value);
                }
            }
        }
        Ok(())
    }

    fn assignment(&mut self, line: &str, _line_no: usize) -> Result<(), String> {
        let (kind, name, rest) = parse_identifier(line)?;
        let rest = rest.trim_start();
        if rest.starts_with('[') {
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
            return Err(format!("constant $${name} cannot be reassigned"));
        }
        if rest.starts_with("capture") {
            return Err("capture syntax was removed; use command \"...\" -> $var".to_string());
        }
        let value = self.eval_value_kind(rest)?;
        match kind {
            IdentKind::Variable => {
                self.vars.insert(name, value);
            }
            IdentKind::Constant => {
                self.consts.insert(name, value);
            }
        }
        Ok(())
    }

    fn push_list_item(&mut self, rest: &str, append: bool) -> Result<(), String> {
        let args = split_commas(rest)?;
        let verb = if append { "append" } else { "prepend" };
        if args.len() != 2 {
            return Err(format!("{verb} requires exactly two arguments"));
        }
        let (kind, name, extra) = parse_identifier(args[0])
            .map_err(|_| "mutation target must be a variable list".to_string())?;
        if kind != IdentKind::Variable || !extra.trim().is_empty() {
            return Err("mutation target must be a variable list".to_string());
        }
        let value = match self.eval_value_kind(args[1])? {
            Value::Scalar(value) => value,
            Value::List(_) => return Err("nested lists are not supported yet".to_string()),
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
            Value::Scalar(_) => Err("mutation target must be a list".to_string()),
        }
    }

    fn remove_list_item(&mut self, target: &str) -> Result<(), String> {
        let (kind, name, selector) = self.parse_mutation_target(target)?;
        if kind != IdentKind::Variable {
            return Err("mutation target must be a variable list".to_string());
        }
        let len = match self.vars.get(&name) {
            Some(Value::List(values)) => values.len(),
            Some(Value::Scalar(_)) => return Err("mutation target must be a list".to_string()),
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
            Some(Value::Scalar(_)) => return Err("mutation target must be a list".to_string()),
            None => return Err(format!("undefined variable ${name}")),
        };
        let index = self.parse_mutation_index(len, selector.trim(), &format!("${name}"))?;
        let value = match self.eval_value_kind(value_text.trim())? {
            Value::Scalar(value) => value,
            Value::List(_) => return Err("nested lists are not supported yet".to_string()),
        };
        let Value::List(values) = self.vars.get_mut(name).expect("checked list") else {
            unreachable!();
        };
        values[index - 1] = value;
        Ok(())
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

    fn eval_contains(&self, args: &str) -> Result<bool, String> {
        let (value_text, search_text) = split_predicate_two_args(args)
            .ok_or_else(|| "contains requires a value and search value".to_string())?;
        let search = self.eval_value(search_text)?;
        match self.eval_expr_value(value_text)? {
            Value::Scalar(value) => Ok(value.contains(&search)),
            Value::List(values) => Ok(values.iter().any(|value| value == &search)),
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
                    text.split(&sep).map(ToString::to_string).collect()
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
                };
                Value::Scalar(len.to_string())
            }
            "join" => {
                expect_arg_count(name, &args, 2)?;
                let Value::List(items) = self.eval_expr_value(args[0])? else {
                    return Err("join requires a list".to_string());
                };
                Value::Scalar(items.join(&self.eval_value(args[1])?))
            }
            "sort" | "reverse" | "unique" => {
                expect_arg_count(name, &args, 1)?;
                let Value::List(mut items) = self.eval_expr_value(args[0])? else {
                    return Err(format!("{name} requires a list"));
                };
                match name {
                    "sort" => items.sort(),
                    "reverse" => items.reverse(),
                    "unique" => {
                        let mut seen = HashSet::new();
                        items.retain(|item| seen.insert(item.clone()));
                    }
                    _ => {}
                }
                Value::List(items)
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
                    }
                    .to_string(),
                )
            }
            "isNumber" | "isText" | "isList" => {
                expect_arg_count(name, &args, 1)?;
                let value = self.eval_expr_value(args[0])?;
                let ok = match (name, value) {
                    ("isList", Value::List(_)) => true,
                    ("isNumber", Value::Scalar(v)) => v.parse::<f64>().is_ok(),
                    ("isText", Value::Scalar(v)) => {
                        v.parse::<f64>().is_err() && v != "true" && v != "false"
                    }
                    _ => false,
                };
                Value::Scalar(ok.to_string())
            }
            "basename" | "dirname" | "extension" | "withoutExtension" => {
                expect_arg_count(name, &args, 1)?;
                let raw = self.eval_value(args[0])?;
                Value::Scalar(path_helper(name, &raw))
            }
            "now" => {
                expect_arg_count(name, &args, 0)?;
                Value::Scalar(current_utc_timestamp())
            }
            "today" => {
                expect_arg_count(name, &args, 0)?;
                Value::Scalar(current_utc_date())
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
        self.vars.insert(destination, Value::List(matches));
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
            if rest.starts_with('[') {
                return self.eval_index_value(kind, &name, rest);
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
                return match self.lookup_value(kind, &name)? {
                    Value::Scalar(value) => Ok(value.clone()),
                    Value::List(values) => Ok(format_list(values)),
                };
            }
            if rest.starts_with('[') {
                return self.eval_index(kind, &name, rest);
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
        }
    }

    fn eval_value_kind(&self, text: &str) -> Result<Value, String> {
        let text = text.trim();
        if text.starts_with('[') {
            return self.eval_list(text);
        }
        if text.starts_with('"') {
            return Ok(Value::Scalar(self.eval_string(text)?));
        }
        if text.starts_with('$') {
            return Ok(Value::Scalar(self.eval_expr(text)?));
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
        for item in split_commas(inner)? {
            match self.eval_value_kind(item)? {
                Value::Scalar(value) => values.push(value),
                Value::List(_) => return Err("nested lists are not supported yet".to_string()),
            }
        }
        Ok(Value::List(values))
    }

    fn eval_index(&self, kind: IdentKind, name: &str, rest: &str) -> Result<String, String> {
        match self.eval_index_value(kind, name, rest)? {
            Value::Scalar(value) => Ok(value),
            Value::List(values) => Ok(format_list(&values)),
        }
    }

    fn eval_index_value(&self, kind: IdentKind, name: &str, rest: &str) -> Result<Value, String> {
        if !rest.ends_with(']') || rest.matches('[').count() != 1 || rest.matches(']').count() != 1
        {
            return Err("malformed list index syntax".to_string());
        }
        let selector = rest[1..rest.len() - 1].trim();
        if selector.is_empty() {
            return Err("malformed list index syntax".to_string());
        }
        match self.lookup_value(kind, name)? {
            Value::List(values) => self.eval_list_selector_value(kind, name, values, selector),
            Value::Scalar(_) => Err(format!(
                "cannot index non-list value {}",
                display_identifier(kind, name)
            )),
        }
    }

    fn eval_list_selector_value(
        &self,
        kind: IdentKind,
        name: &str,
        values: &[String],
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
        Ok(Value::Scalar(values[index - 1].clone()))
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
        values: &[String],
        op: &str,
        rhs: &str,
    ) -> Result<Vec<String>, String> {
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
                let item_num = item
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
        let selected: Vec<String> = values
            .iter()
            .filter(|item| match op {
                "==" => *item == &target,
                "!=" => *item != &target,
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
                    let end = index_tail
                        .find(']')
                        .ok_or_else(|| "malformed list index syntax".to_string())?
                        + 1;
                    output.push_str(&self.eval_index(kind, &name, &index_tail[..end])?);
                    index += consumed + leading_ws + end;
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

    fn lookup_value(&self, kind: IdentKind, name: &str) -> Result<&Value, String> {
        match kind {
            IdentKind::Variable => self
                .vars
                .get(name)
                .ok_or_else(|| format!("undefined variable ${name}")),
            IdentKind::Constant => self
                .consts
                .get(name)
                .ok_or_else(|| format!("undefined constant $${name}")),
        }
    }

    fn lookup(&self, kind: IdentKind, name: &str) -> Result<String, String> {
        match self.lookup_value(kind, name)? {
            Value::Scalar(value) => Ok(value.clone()),
            Value::List(_) => Err(format!(
                "cannot use list {} as scalar value",
                display_identifier(kind, name)
            )),
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
    Loop,
    Function,
    Try { saw_catch: bool },
    Log,
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
        if let Some(kind) = self.blocks.last() {
            let name = match kind {
                BlockKind::If => "if",
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
            };
            self.error(
                lines.last().map(|(n, _)| *n).unwrap_or(1),
                format!("missing end for {name}"),
            );
        }
    }

    fn check_line(&mut self, line: &str) -> Result<(), String> {
        if let Some(rest) = line
            .strip_prefix("export fn ")
            .or_else(|| line.strip_prefix("fn "))
        {
            parse_function_signature(rest.trim())?;
            self.blocks.push(BlockKind::Function);
            return Ok(());
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
            if !matches!(self.blocks.last(), Some(BlockKind::If)) {
                return Err("unexpected conditional marker".to_string());
            }
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
                Some(_) => {}
                None => return Err("unexpected conditional marker".to_string()),
            }
            return Ok(());
        }
        if line == "break" || line == "continue" {
            if !self.blocks.iter().any(|kind| *kind == BlockKind::Loop) {
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

fn validate_statement_form(line: &str) -> Result<(), String> {
    if line.starts_with('$') {
        return Interpreter::default().assignment(line, 0).map(|_| ());
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
    if let Some(rest) = line.strip_prefix("command") {
        let (_, tail) = split_string_literal_tail(rest.trim())?;
        let tail = tail.trim();
        if tail.is_empty() {
            return Ok(());
        }
        let Some(destination) = tail.strip_prefix("->") else {
            return Err("command capture syntax is command \"...\" -> $var".to_string());
        };
        let (kind, _, extra) = parse_identifier(destination.trim())
            .map_err(|_| "command capture destination must be a variable".to_string())?;
        if kind != IdentKind::Variable || !extra.trim().is_empty() {
            return Err("command capture destination must be a variable".to_string());
        }
        return Ok(());
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
    } else if condition == "contains" {
        return Err("contains requires a value and search value".to_string());
    }
    Ok(())
}

fn split_command_name(line: &str) -> Option<(&str, &str)> {
    line.split_once(char::is_whitespace)
        .unwrap_or((line, ""))
        .into()
}

fn is_standard_helper(name: &str) -> bool {
    matches!(
        name,
        "trim"
            | "lower"
            | "upper"
            | "replace"
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
            | "basename"
            | "dirname"
            | "extension"
            | "withoutExtension"
            | "now"
            | "today"
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
        | "toString" | "isEmpty" | "isNumber" | "isText" | "isList" | "basename" | "dirname"
        | "extension" | "withoutExtension" => 1,
        "split" | "startsWith" | "endsWith" | "join" | "randomInt" => 2,
        "replace" => 3,
        "now" | "today" | "random" => 0,
        _ => unreachable!(),
    };
    expect_arg_count(name, &args, n)
}

fn expect_arg_count(name: &str, args: &[&str], expected: usize) -> Result<(), String> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(format!("{name} expects {expected} argument(s)"))
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

fn parse_integer_arg(text: &str, error: &str) -> Result<i64, String> {
    if text.contains('.') {
        return Err(error.to_string());
    }
    text.parse::<i64>().map_err(|_| error.to_string())
}

fn path_helper(name: &str, raw: &str) -> String {
    let p = Path::new(raw);
    match name {
        "basename" => p
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
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
    if pattern.split('/').any(|segment| segment == "**") {
        return Err("files does not support recursive ** patterns yet".to_string());
    }
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
            } else if entry.path().is_dir() {
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
    } else if path.is_dir() {
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
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
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
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

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
            '(' if bracket_depth == 0 => paren_depth += 1,
            ')' if bracket_depth == 0 => paren_depth = paren_depth.saturating_sub(1),
            _ if bracket_depth == 0 && paren_depth == 0 => {
                let tail = &condition[index..];
                if tail.starts_with(keyword) {
                    let before = condition[..index].chars().next_back();
                    let after = condition[index + keyword.len()..].chars().next();
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
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

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
            '(' if bracket_depth == 0 => paren_depth += 1,
            ')' if bracket_depth == 0 => paren_depth = paren_depth.saturating_sub(1),
            _ if bracket_depth == 0 && paren_depth == 0 => {
                let tail = &condition[index..];
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

fn starts_block(line: &str) -> bool {
    line.starts_with("if ")
        || line.starts_with("while ")
        || line.starts_with("for ")
        || line.starts_with("repeat ")
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

fn starts_function_block(line: &str) -> bool {
    line.starts_with("fn ") || line.starts_with("export fn ")
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

fn parse_call_syntax(text: &str) -> Result<(&str, Vec<&str>), String> {
    let Some(open) = text.find('(') else {
        return Err("old call syntax was removed; use call name(args)".to_string());
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
        if starts_function_block(line) {
            depth += 1;
        }
        if line == "}" {
            if depth == 0 {
                return Ok((Function { name, params, body }, index + 1));
            }
            depth -= 1;
        }
        if line == "end" && depth == 0 {
            return Err("old function syntax was removed; close functions with }".to_string());
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
