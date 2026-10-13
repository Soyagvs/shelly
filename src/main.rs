use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Stdio;

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

fn check_source(source: &str, file: &str) -> Vec<Diagnostic> {
    match strip_comments(source) {
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
}

impl Interpreter {
    fn run(&mut self, source: &str) -> Result<(), String> {
        let lines = strip_comments(source)?;
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
            if matches!(line, "elif" | "else" | "end") || line.starts_with("elif ") {
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
            return self.push_list_item(rest.trim(), true);
        }
        if let Some(rest) = line.strip_prefix("prepend ") {
            return self.push_list_item(rest.trim(), false);
        }
        if let Some(rest) = line.strip_prefix("remove ") {
            return self.remove_list_item(rest.trim());
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
        if let Some(rest) = line.strip_prefix("command") {
            let (command_text, tail) = split_string_literal_tail(rest.trim())?;
            let command = self.eval_string(command_text)?;
            let tail = tail.trim();
            if tail.is_empty() {
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
        let (name, rest) = call_text
            .split_once(char::is_whitespace)
            .unwrap_or((call_text, ""));
        let function = self
            .functions
            .get(name)
            .cloned()
            .ok_or_else(|| format!("unknown command or function: {name}"))?;
        let arg_texts = if rest.trim().is_empty() {
            Vec::new()
        } else {
            split_commas(rest.trim())?
        };
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

    fn eval_condition(&self, condition: &str) -> Result<bool, String> {
        if condition.is_empty() {
            return Err("condition required".to_string());
        }
        if condition.contains('(') || condition.contains(')') {
            return Err("unsupported complex condition".to_string());
        }
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
                return self.lookup(kind, &name);
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
            if rest.trim().is_empty() {
                return Err("condition required".to_string());
            }
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
            if rest.trim().is_empty() {
                return Err("condition required".to_string());
            }
            return Ok(());
        }
        if line == "else" {
            if !matches!(self.blocks.last(), Some(BlockKind::If)) {
                return Err("unexpected conditional marker".to_string());
            }
            return Ok(());
        }
        if let Some(rest) = line.strip_prefix("while ") {
            if rest.trim().is_empty() {
                return Err("condition required".to_string());
            }
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
        if line == "end" {
            if self.blocks.pop().is_none() {
                return Err("unexpected conditional marker".to_string());
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

fn parse_function_signature(rest: &str) -> Result<(), String> {
    let (name, params_text) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    if !is_identifier_name(name) {
        return Err("function name must start with an ASCII letter".to_string());
    }
    if !params_text.trim().is_empty() {
        for item in split_commas(params_text.trim())? {
            let left = item
                .split_once('=')
                .map(|(left, _)| left)
                .unwrap_or(item)
                .trim();
            let (kind, _, extra) = parse_identifier(left)?;
            if kind != IdentKind::Variable || !extra.trim().is_empty() {
                return Err("function parameters must be variables".to_string());
            }
        }
    }
    Ok(())
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
        let name = rest.trim().split_whitespace().next().unwrap_or("");
        if name.is_empty() {
            return Err("call requires a function name".to_string());
        }
        if !is_identifier_name(name) {
            return Err("function name must start with an ASCII letter".to_string());
        }
        return Ok(());
    }
    if line == "call" {
        return Err("call requires a function name".to_string());
    }
    if line.strip_prefix("shellout").is_some()
        || line.strip_prefix("shellin").is_some()
        || line.starts_with("import ")
        || line.starts_with("include ")
        || line.starts_with("exit")
        || line.starts_with("append ")
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

fn split_condition_keyword<'a>(condition: &'a str, keyword: &str) -> Option<(&'a str, &'a str)> {
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
            _ if bracket_depth == 0 => {
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
            _ if bracket_depth == 0 => {
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
}

fn parse_function(
    rest: &str,
    lines: &[(usize, String)],
    start: usize,
) -> Result<(Function, usize), String> {
    let (name, params_text) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    if !is_identifier_name(name) {
        return Err("function name must start with an ASCII letter".to_string());
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
            if kind != IdentKind::Variable {
                return Err("function parameters must be variables".to_string());
            }
            if !extra.trim().is_empty() {
                return Err("unexpected text after parameter".to_string());
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
    let mut body = Vec::new();
    let mut index = start + 1;
    while index < lines.len() {
        let line_no = lines[index].0;
        let line = lines[index].1.trim();
        if line == "end" {
            return Ok((
                Function {
                    name: name.to_string(),
                    params,
                    body,
                },
                index + 1,
            ));
        }
        if !line.is_empty() {
            body.push((line_no, line.to_string()));
        }
        index += 1;
    }
    Err(format!("missing end for function {name}"))
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
