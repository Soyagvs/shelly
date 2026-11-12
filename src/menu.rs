use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute, terminal,
};
use std::io::{self, BufRead, IsTerminal, Write};

#[derive(Clone, Debug)]
pub(crate) struct MenuOption {
    pub label: String,
    pub value: crate::Value,
    pub hint: Option<String>,
}
#[derive(Clone, Debug)]
pub(crate) struct MenuSpec {
    pub title: String,
    pub subtitle: Option<String>,
    pub theme: MenuTheme,
    pub default: usize,
    pub options: Vec<MenuOption>,
    pub destination: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MenuTheme {
    Panel,
    Minimal,
}

pub(crate) fn parse_spec(
    lines: &[(usize, String)],
    start: usize,
    eval: &mut dyn FnMut(&str) -> Result<crate::Value, String>,
) -> Result<(MenuSpec, usize), String> {
    let header = lines[start]
        .1
        .trim()
        .strip_prefix("menu ")
        .ok_or("menu header required")?;
    let (title_expr, tail) = crate::split_string_literal_tail(header)
        .map_err(|_| "menu syntax is menu \"Title\" -> $destination".to_string())?;
    let title = scalar(eval(title_expr)?)?;
    let destination = tail
        .trim()
        .strip_prefix("->")
        .ok_or("menu syntax requires -> $destination")?
        .trim();
    let destination = crate::parse_var_ref(destination)
        .map_err(|_| "menu destination must be a mutable variable".to_string())?;
    let mut spec = MenuSpec {
        title,
        subtitle: None,
        theme: MenuTheme::Panel,
        default: 1,
        options: Vec::new(),
        destination,
    };
    let mut i = start + 1;
    let mut saw_option = false;
    let mut saw_theme = false;
    let mut saw_default = false;
    while i < lines.len() {
        let line = lines[i].1.trim();
        if line == "end" {
            if spec.options.is_empty() {
                return Err("menu options must not be empty".into());
            }
            if spec.default > spec.options.len() {
                return Err(format!(
                    "default {} is out of range for {} options",
                    spec.default,
                    spec.options.len()
                ));
            }
            validate_labels(&spec.options)?;
            return Ok((spec, i + 1));
        }
        if line.starts_with("menu ")
            || line.starts_with("if ")
            || line.starts_with("for ")
            || line.starts_with("while ")
            || line.starts_with("case ")
            || line.starts_with("repeat ")
        {
            return Err("nested blocks in menus are unsupported".into());
        }
        if let Some(rest) = line.strip_prefix("subtitle ") {
            if saw_option {
                return Err("menu subtitle must appear before the first option".into());
            }
            if spec.subtitle.is_some() {
                return Err("menu subtitle may appear at most once".into());
            }
            spec.subtitle = Some(scalar(eval(rest.trim())?)?);
        } else if let Some(rest) = line.strip_prefix("theme ") {
            if saw_option {
                return Err("menu theme must appear before the first option".into());
            }
            if saw_theme {
                return Err("menu theme may appear at most once".into());
            }
            saw_theme = true;
            spec.theme = match rest.trim() {
                "panel" => MenuTheme::Panel,
                "minimal" => MenuTheme::Minimal,
                _ => return Err("menu theme must be panel or minimal".into()),
            };
        } else if let Some(rest) = line.strip_prefix("default ") {
            if saw_option {
                return Err("menu default must appear before the first option".into());
            }
            if saw_default {
                return Err("menu default may appear at most once".into());
            }
            saw_default = true;
            spec.default = rest
                .trim()
                .parse()
                .map_err(|_| "menu default must be a positive integer".to_string())?;
            if spec.default == 0 {
                return Err("menu default must be a positive integer".into());
            }
        } else if let Some(rest) = line.strip_prefix("option ") {
            let (label, value) = rest
                .split_once("->")
                .ok_or("menu option requires -> VALUE_EXPR")?;
            let label = scalar(eval(label.trim())?)?;
            let value = eval(value.trim())?;
            spec.options.push(MenuOption {
                label,
                value,
                hint: None,
            });
            saw_option = true;
        } else if let Some(rest) = line.strip_prefix("hint ") {
            let option = spec
                .options
                .last_mut()
                .ok_or("menu hint must immediately follow an option")?;
            if option.hint.is_some() {
                return Err("menu option may have at most one hint".into());
            }
            option.hint = Some(scalar(eval(rest.trim())?)?);
        } else {
            return Err("unexpected line in menu block".into());
        }
        i += 1;
    }
    Err("missing end for menu".into())
}
fn scalar(value: crate::Value) -> Result<String, String> {
    match value {
        crate::Value::Scalar(value) => Ok(value),
        _ => Err("menu text must be a string".into()),
    }
}
fn validate_labels(options: &[MenuOption]) -> Result<(), String> {
    for i in 0..options.len() {
        if options[..i].iter().any(|o| o.label == options[i].label) {
            return Err(format!(
                "menu option labels must be unique: '{}'",
                options[i].label
            ));
        }
    }
    Ok(())
}

pub(crate) fn select(spec: &MenuSpec) -> Result<usize, String> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return line_mode(spec);
    }
    tty_mode(spec)
}
fn line_mode(spec: &MenuSpec) -> Result<usize, String> {
    println!("{}", spec.title);
    if let Some(s) = &spec.subtitle {
        println!("{s}");
    }
    for (i, o) in spec.options.iter().enumerate() {
        println!("{}) {}", i + 1, o.label);
        if let Some(h) = &o.hint {
            println!("   {h}");
        }
    }
    print!("Menu: ");
    io::stdout()
        .flush()
        .map_err(|e| format!("failed to flush stdout: {e}"))?;
    read_line_selection(spec, &mut io::stdin().lock())
}

fn read_line_selection(spec: &MenuSpec, reader: &mut impl BufRead) -> Result<usize, String> {
    let mut input = String::new();
    let n = reader
        .read_line(&mut input)
        .map_err(|e| format!("menu input read failed: {e}"))?;
    parse_line_selection(spec, n, &input)
}

fn parse_line_selection(spec: &MenuSpec, n: usize, input: &str) -> Result<usize, String> {
    if n == 0 {
        return Err("menu input closed".into());
    }
    let text = input.trim_end_matches(['\r', '\n']);
    if let Ok(n) = text.parse::<usize>()
        && (1..=spec.options.len()).contains(&n)
    {
        return Ok(n - 1);
    }
    if let Some(i) = spec.options.iter().position(|o| o.label == text) {
        return Ok(i);
    }
    Err("menu selection is invalid".into())
}
struct Cleanup;
impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), Show);
    }
}
fn tty_mode(spec: &MenuSpec) -> Result<usize, String> {
    terminal::enable_raw_mode().map_err(|e| format!("menu raw mode failed: {e}"))?;
    let _cleanup = Cleanup;
    let mut out = io::stdout();
    execute!(out, Hide).map_err(|e| format!("menu cursor setup failed: {e}"))?;
    let mut selected = spec.default.saturating_sub(1).min(spec.options.len() - 1);
    render(&mut out, spec, selected, false)?;
    loop {
        let ev = event::read().map_err(|e| format!("menu event read failed: {e}"))?;
        match ev {
            Event::Key(KeyEvent {
                code: KeyCode::Up | KeyCode::Char('k'),
                kind: KeyEventKind::Press,
                ..
            }) => selected = (selected + spec.options.len() - 1) % spec.options.len(),
            Event::Key(KeyEvent {
                code: KeyCode::Down | KeyCode::Char('j'),
                kind: KeyEventKind::Press,
                ..
            }) => selected = (selected + 1) % spec.options.len(),
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => return Ok(selected),
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => return Err("menu cancelled".into()),
            Event::Resize(_, _) => {}
            _ => continue,
        };
        render(&mut out, spec, selected, true)?;
    }
}
fn render(
    out: &mut impl Write,
    spec: &MenuSpec,
    selected: usize,
    redraw: bool,
) -> Result<(), String> {
    render_at_width(
        out,
        spec,
        selected,
        redraw,
        terminal::size().map(|(w, _)| w as usize).unwrap_or(80),
    )
}

fn render_at_width(
    out: &mut impl Write,
    spec: &MenuSpec,
    selected: usize,
    redraw: bool,
    width: usize,
) -> Result<(), String> {
    let mut lines = Vec::new();
    let panel = matches!(spec.theme, MenuTheme::Panel);
    let inner = width.saturating_sub(if panel { 4 } else { 0 }).max(1);
    for text in wrap_text(&spec.title, inner, 0) {
        lines.push(if panel { format!("+ {text} +") } else { text });
    }
    if let Some(s) = &spec.subtitle {
        lines.extend(wrap_text(s, inner, 0));
    }
    for (i, o) in spec.options.iter().enumerate() {
        let prefix = format!("{} {}) ", if i == selected { ">" } else { " " }, i + 1);
        let avail = inner.saturating_sub(display_width(&prefix));
        let wrapped = wrap_text(&o.label, avail.max(1), display_width(&prefix));
        for (n, text) in wrapped.into_iter().enumerate() {
            lines.push(if n == 0 {
                format!("{prefix}{text}")
            } else {
                format!("{}{}", " ".repeat(display_width(&prefix)), text)
            });
        }
        if let Some(h) = &o.hint {
            lines.extend(
                wrap_text(h, inner.saturating_sub(3).max(1), 3)
                    .into_iter()
                    .map(|s| format!("   {s}")),
            );
        }
    }
    lines.push("[Enter] select  [Esc] cancel".into());
    if redraw {
        write!(out, "\x1b[{}A", lines.len()).map_err(|e| format!("menu render failed: {e}"))?;
    }
    for line in lines {
        writeln!(out, "\r\x1b[2K{line}\r").map_err(|e| format!("menu render failed: {e}"))?;
    }
    out.flush().map_err(|e| format!("menu render failed: {e}"))
}

fn display_width(text: &str) -> usize {
    text.chars().map(char_width).sum()
}
fn char_width(c: char) -> usize {
    if c.is_control() || matches!(c as u32, 0x300..=0x36f | 0x1ab0..=0x1aff) {
        0
    } else if matches!(c as u32, 0x1100..=0x115f | 0x2329..=0x232a | 0x2e80..=0xa4cf | 0xac00..=0xd7a3 | 0xf900..=0xfaff | 0xfe10..=0xfe19 | 0xfe30..=0xfe6f | 0xff00..=0xff60 | 0xffe0..=0xffe6)
    {
        2
    } else {
        1
    }
}
fn wrap_text(text: &str, width: usize, indent: usize) -> Vec<String> {
    let width = width.max(1);
    let mut out = Vec::new();
    let mut line = String::new();
    let mut used = 0;
    for word in text.split_whitespace() {
        let mut chars = word.chars().peekable();
        while chars.peek().is_some() {
            let mut chunk = String::new();
            let mut chunk_width = 0;
            while let Some(&ch) = chars.peek() {
                let cw = char_width(ch);
                if chunk_width + cw <= width {
                    chunk.push(ch);
                    chunk_width += cw;
                    chars.next();
                } else {
                    break;
                }
            }
            if chunk.is_empty() {
                chunk.push('…');
                chars.next();
                chunk_width = 1;
            }
            if used > 0 && used + 1 + chunk_width > width {
                out.push(std::mem::take(&mut line));
                used = 0;
            }
            if used > 0 {
                line.push(' ');
                used += 1;
            }
            line.push_str(&chunk);
            used += chunk_width;
            if chars.peek().is_some() {
                out.push(std::mem::take(&mut line));
                used = 0;
            }
        }
    }
    if !line.is_empty() {
        out.push(line);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    let _ = indent;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spec(theme: MenuTheme) -> MenuSpec {
        MenuSpec {
            title: "界面 title".into(),
            subtitle: Some("subtitle".into()),
            theme,
            default: 1,
            destination: "x".into(),
            options: vec![
                MenuOption {
                    label: "first option".into(),
                    value: crate::Value::Scalar("v".into()),
                    hint: Some("a useful hint".into()),
                },
                MenuOption {
                    label: "second".into(),
                    value: crate::Value::Scalar("w".into()),
                    hint: None,
                },
            ],
        }
    }
    #[test]
    fn panel_and_minimal_output_have_semantics() {
        let mut p = Vec::new();
        render_at_width(&mut p, &spec(MenuTheme::Panel), 0, false, 30).unwrap();
        let p = String::from_utf8(p).unwrap();
        assert!(p.contains("+ ") && p.contains("> 1)"));
        let mut m = Vec::new();
        render_at_width(&mut m, &spec(MenuTheme::Minimal), 0, false, 30).unwrap();
        let m = String::from_utf8(m).unwrap();
        assert!(!m.contains("+ "));
        assert!(m.contains("[Enter]"));
    }
    #[test]
    fn wraps_unicode_and_redraws_actual_lines() {
        let mut out = Vec::new();
        render_at_width(&mut out, &spec(MenuTheme::Panel), 0, false, 12).unwrap();
        let first = String::from_utf8(out).unwrap();
        let count = first.lines().count();
        let mut redraw = Vec::new();
        render_at_width(&mut redraw, &spec(MenuTheme::Panel), 1, true, 12).unwrap();
        assert!(
            String::from_utf8(redraw)
                .unwrap()
                .starts_with(&format!("\x1b[{}A", count))
        );
    }
    #[test]
    fn long_unbreakable_tokens_split_safely_at_display_width() {
        let lines = wrap_text("abcdefghij界", 4, 0);
        assert!(lines.iter().all(|line| display_width(line) <= 4));
        assert_eq!(wrap_text("界", 1, 0), vec!["…"]);
    }
    #[test]
    fn line_selection_reads_only_one_line() {
        struct OneLine;
        impl io::Read for OneLine {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                unreachable!()
            }
        }
        impl BufRead for OneLine {
            fn fill_buf(&mut self) -> io::Result<&[u8]> {
                unreachable!()
            }
            fn consume(&mut self, _: usize) {}
            fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
                buf.push_str("2\r\n");
                Ok(3)
            }
        }
        let mut reader = OneLine;
        assert_eq!(
            read_line_selection(&spec(MenuTheme::Panel), &mut reader),
            Ok(1)
        );
    }

    #[test]
    fn runtime_rejects_invalid_directive_placement_and_repetition() {
        for (directive, expected) in [
            (
                "subtitle \"again\"\noption \"b\" -> \"2\"\nsubtitle \"late\"",
                "menu subtitle must appear before the first option",
            ),
            (
                "theme panel\ntheme minimal",
                "menu theme may appear at most once",
            ),
            (
                "theme panel\noption \"b\" -> \"2\"\ntheme minimal",
                "menu theme must appear before the first option",
            ),
            (
                "default 1\ndefault 2",
                "menu default may appear at most once",
            ),
            (
                "option \"b\" -> \"2\"\ndefault 1",
                "menu default must appear before the first option",
            ),
        ] {
            let mut lines = vec![(1, "menu \"Title\" -> $choice".to_string())];
            lines.extend(
                directive
                    .lines()
                    .enumerate()
                    .map(|(i, line)| (i + 2, line.to_string())),
            );
            lines.push((99, "option \"a\" -> \"1\"".into()));
            lines.push((100, "end".into()));
            let error = parse_spec(&lines, 0, &mut |expr| {
                Ok(crate::Value::Scalar(expr.trim_matches('"').into()))
            })
            .unwrap_err();
            assert_eq!(error, expected);
        }
    }

    #[test]
    fn duplicate_visible_labels_rejected() {
        let options = vec![
            MenuOption {
                label: "same".into(),
                value: crate::Value::Scalar("1".into()),
                hint: None,
            },
            MenuOption {
                label: "same".into(),
                value: crate::Value::Scalar("2".into()),
                hint: None,
            },
        ];
        assert!(validate_labels(&options).is_err());
    }
}
