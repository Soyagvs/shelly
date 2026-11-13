use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute, terminal,
};
use std::io::{self, BufRead, IsTerminal, Write};

#[derive(Clone, Debug)]
pub(crate) struct MenuOption {
    pub key: char,
    pub label: String,
    pub value: crate::Value,
    pub hint: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuKeyMode {
    Numbers,
    LettersLower,
    LettersUpper,
    Explicit,
}
#[derive(Clone, Debug)]
pub(crate) struct MenuSpec {
    pub title: String,
    pub subtitle: Option<String>,
    pub default: usize,
    pub options: Vec<MenuOption>,
    key_mode: MenuKeyMode,
    pub destination: String,
    pub blank_rows: usize,
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
    let (title_expr, destination) = header
        .split_once("->")
        .ok_or("menu syntax requires -> $destination")?;
    let title = scalar(eval(title_expr.trim())?)?;
    let destination = destination.trim();
    let destination = crate::parse_var_ref(destination)
        .map_err(|_| "menu destination must be a mutable variable".to_string())?;
    let mut spec = MenuSpec {
        title,
        subtitle: None,
        default: 1,
        options: Vec::new(),
        key_mode: MenuKeyMode::Explicit,
        destination,
        blank_rows: 0,
    };
    let mut i = start + 1;
    let mut saw_option = false;
    let mut saw_default = false;
    let mut declared_mode = false;
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
            if declared_mode {
                for (i, option) in spec.options.iter_mut().enumerate() {
                    if option.key != '\0' {
                        return Err(
                            "menu key modes cannot mix declared keys with option-local keys".into(),
                        );
                    }
                    option.key = match spec.key_mode {
                        MenuKeyMode::Numbers => char::from_digit((i + 1) as u32, 10)
                            .ok_or("menu numbers mode supports at most 9 options")?,
                        MenuKeyMode::LettersLower if i < 26 => (b'a' + i as u8) as char,
                        MenuKeyMode::LettersUpper if i < 26 => (b'A' + i as u8) as char,
                        MenuKeyMode::LettersLower | MenuKeyMode::LettersUpper => {
                            return Err("menu letter modes support at most 26 options".into());
                        }
                        MenuKeyMode::Explicit => unreachable!(),
                    };
                }
            }
            let keys: std::collections::HashSet<_> = spec.options.iter().map(|o| o.key).collect();
            if keys.len() != spec.options.len() || spec.options.iter().any(|o| o.key == '\0') {
                return Err(if !declared_mode {
                    "menu must declare keys or use option-local keys"
                } else {
                    "menu option keys must be unique"
                }
                .into());
            }
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
        } else if line.starts_with("style ") {
            return Err("menu style directives were removed; use shared style(...) expressions or shellout \"\" for blank rows".into());
        } else if line == "shellout \"\"" {
            spec.blank_rows += 1;
        } else if line.starts_with("shellout ") {
            return Err("menu shellout only accepts exactly shellout \"\" as a blank visual row; move commands outside the menu".into());
        } else if line.starts_with("theme ") {
            return Err(
                "menu theme was removed; remove it because menus use one fixed compact layout"
                    .into(),
            );
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
        } else if line.starts_with("group ") {
            return Err(
                "menu group was removed; remove it because menus use one global option list".into(),
            );
        } else if let Some(rest) = line.strip_prefix("keys ") {
            if saw_option || declared_mode {
                return Err("menu keys must appear once before options".into());
            }
            declared_mode = true;
            spec.key_mode = match rest.trim() {
                "numbers" => MenuKeyMode::Numbers,
                "letters lower" => MenuKeyMode::LettersLower,
                "letters upper" => MenuKeyMode::LettersUpper,
                _ => {
                    return Err("menu keys must be numbers, letters lower, or letters upper".into());
                }
            };
        } else if let Some(rest) = line.strip_prefix("option") {
            let (key, rest) = if let Some(rest) = rest.strip_prefix("[") {
                let (key, rest) = rest.split_once(']').ok_or("menu option key requires ]")?;
                let mut chars = key.chars();
                let key = chars
                    .next()
                    .ok_or("menu option key must be one printable ASCII character")?;
                if chars.next().is_some() || !key.is_ascii() || !key.is_ascii_graphic() {
                    return Err("menu option key must be one printable ASCII character".into());
                }
                (key, rest.trim_start())
            } else {
                ('\0', rest)
            };
            let (label, value) = rest
                .trim_start()
                .split_once("->")
                .ok_or("menu option requires -> VALUE_EXPR")?;
            let label = scalar(eval(label.trim())?)?;
            let value = eval(value.trim())?;
            spec.options.push(MenuOption {
                key,
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
        if options[..i]
            .iter()
            .any(|o| visible_text(&o.label) == visible_text(&options[i].label))
        {
            return Err(format!(
                "menu option labels must be unique: '{}'",
                options[i].label
            ));
        }
    }
    Ok(())
}

pub(crate) fn select(spec: &MenuSpec, color: bool) -> Result<usize, String> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return line_mode(spec);
    }
    tty_mode(spec, color)
}
fn line_mode(spec: &MenuSpec) -> Result<usize, String> {
    println!("{}", visible_text(&spec.title));
    if let Some(s) = &spec.subtitle {
        println!("{}", visible_text(s));
    }
    for _ in 0..spec.blank_rows {
        println!();
    }
    for o in &spec.options {
        println!("{}: {}", o.key, visible_text(&o.label));
        if let Some(h) = &o.hint {
            println!("   {}", visible_text(h));
        }
    }
    print!("Menu: ");
    io::stdout()
        .flush()
        .map_err(|e| format!("failed to flush stdout: {e}"))?;
    read_line_selection(spec, &mut io::stdin().lock())
}

fn visible_text(text: &str) -> String {
    crate::strip_ansi(text)
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
    if text.chars().count() == 1
        && let Some(i) = spec
            .options
            .iter()
            .position(|o| o.key == text.chars().next().unwrap())
    {
        return Ok(i);
    }
    if let Some(i) = spec
        .options
        .iter()
        .position(|o| visible_text(&o.label) == text)
    {
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
fn tty_mode(spec: &MenuSpec, color: bool) -> Result<usize, String> {
    terminal::enable_raw_mode().map_err(|e| format!("menu raw mode failed: {e}"))?;
    let _cleanup = Cleanup;
    let mut out = io::stdout();
    execute!(out, Hide).map_err(|e| format!("menu cursor setup failed: {e}"))?;
    let mut selected = spec.default.saturating_sub(1).min(spec.options.len() - 1);
    let mut rendered_lines = render(&mut out, spec, selected, None, color)?;
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
                code: KeyCode::Char(ch),
                kind: KeyEventKind::Press,
                ..
            }) => {
                if let Some(i) = spec.options.iter().position(|o| o.key == ch) {
                    return Ok(i);
                }
            }
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
        rendered_lines = render(&mut out, spec, selected, Some(rendered_lines), color)?;
    }
}
fn render(
    out: &mut impl Write,
    spec: &MenuSpec,
    selected: usize,
    previous_lines: Option<usize>,
    color: bool,
) -> Result<usize, String> {
    render_at_width(
        out,
        spec,
        selected,
        previous_lines,
        terminal::size().map(|(w, _)| w as usize).unwrap_or(80),
        color,
    )
}

fn render_at_width(
    out: &mut impl Write,
    spec: &MenuSpec,
    selected: usize,
    previous_lines: Option<usize>,
    width: usize,
    color: bool,
) -> Result<usize, String> {
    let mut lines: Vec<String> = Vec::new();
    let project = |text: &str| {
        if color {
            text.to_string()
        } else {
            visible_text(text)
        }
    };
    let width = width.max(1);
    lines.extend(wrap_text(&project(&spec.title), width, 0));
    if let Some(s) = &spec.subtitle {
        lines.extend(wrap_text(&project(s), width, 0));
    }
    let key_width = spec
        .options
        .iter()
        .map(|o| o.key.len_utf8())
        .max()
        .unwrap_or(1);
    for (i, o) in spec.options.iter().enumerate() {
        let prefix = format!(
            "{} {:>key_width$}) ",
            if i == selected { ">" } else { " " },
            o.key
        );
        let available = width.saturating_sub(display_width(&prefix)).max(1);
        let label = project(&o.label);
        let wrapped = wrap_text(&label, available, 0);
        lines.push(format!("{prefix}{}", wrapped[0]));
        for continuation in wrapped.iter().skip(1) {
            lines.push(format!(
                "{}{}",
                " ".repeat(display_width(&prefix)),
                continuation
            ));
        }
        if let Some(hint) = &o.hint {
            let indent = " ".repeat(display_width(&prefix));
            let hint = project(hint);
            for line in wrap_text(&hint, width.saturating_sub(indent.len()).max(1), 0) {
                lines.push(format!("{indent}{line}"));
            }
        }
    }
    for _ in 0..spec.blank_rows {
        lines.push(String::new());
    }
    lines.extend(wrap_text("[Enter] select  [Esc] cancel", width, 0));
    if let Some(previous) = previous_lines {
        write!(out, "\x1b[{}A", previous).map_err(|e| format!("menu render failed: {e}"))?;
    }
    let count = lines.len();
    for line in lines {
        writeln!(out, "\r\x1b[2K{line}\r").map_err(|e| format!("menu render failed: {e}"))?;
    }
    out.flush()
        .map_err(|e| format!("menu render failed: {e}"))?;
    Ok(count)
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
    fn spec() -> MenuSpec {
        MenuSpec {
            title: "界面 title".into(),
            subtitle: Some("subtitle".into()),
            default: 1,
            key_mode: MenuKeyMode::Explicit,
            destination: "x".into(),
            blank_rows: 0,
            options: vec![
                MenuOption {
                    key: 'a',
                    label: "first option".into(),
                    value: crate::Value::Scalar("v".into()),
                    hint: Some("a useful hint".into()),
                },
                MenuOption {
                    key: 'b',
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
        render_at_width(&mut p, &spec(), 0, None, 30, true).unwrap();
        let p = String::from_utf8(p).unwrap();
        assert!(!p.contains("+----------------"));
        assert!(p.contains("> a) first option"));
        assert!(p.contains("a useful hint"));
        assert!(p.contains("[Enter]"));
    }

    #[test]
    fn panel_cards_wrap_hints_and_fit_narrow_width() {
        let mut out = Vec::new();
        render_at_width(&mut out, &spec(), 0, None, 8, true).unwrap();
        let rendered = String::from_utf8(out).unwrap();
        for line in rendered
            .lines()
            .map(|line| line.strip_prefix("\r\x1b[2K").unwrap_or(line))
        {
            assert!(display_width(line) <= 8, "line too wide: {line:?}");
        }
        assert!(rendered.contains("界"));
        assert!(rendered.contains("a"));
    }
    #[test]
    fn wraps_unicode_and_redraws_actual_lines() {
        let mut out = Vec::new();
        render_at_width(&mut out, &spec(), 0, None, 12, true).unwrap();
        let first = String::from_utf8(out).unwrap();
        let count = first.lines().count();
        let mut redraw = Vec::new();
        render_at_width(&mut redraw, &spec(), 1, Some(count), 12, true).unwrap();
        assert!(
            String::from_utf8(redraw)
                .unwrap()
                .starts_with(&format!("\x1b[{}A", count))
        );
    }
    #[test]
    fn styled_text_is_plain_in_pipe_projection_and_visible_width_is_used() {
        let styled = "\x1b[1;96m界面 title\x1b[0m";
        assert_eq!(visible_text(styled), "界面 title");
        assert_eq!(display_width(&visible_text(styled)), 10);
        let mut out = Vec::new();
        let mut menu = spec();
        menu.title = styled.into();
        menu.subtitle = Some("\x1b[31msubtitle\x1b[0m".into());
        menu.options[0].label = "\x1b[32mfirst option\x1b[0m".into();
        menu.options[0].hint = Some("\x1b[33ma useful hint\x1b[0m".into());
        render_at_width(&mut out, &menu, 0, None, 12, false).unwrap();
        let rendered = String::from_utf8(out).unwrap();
        assert!(!rendered.contains("\x1b[1;96m"));
        assert!(!rendered.contains("\x1b[31m"));
        assert!(!rendered.contains("\x1b[32m"));
        assert!(!rendered.contains("\x1b[33m"));
        assert!(rendered.contains("界面 title") || rendered.contains("界面"));
        assert!(rendered.contains("useful") && rendered.contains("hint"));
    }

    #[test]
    fn color_enabled_projection_preserves_styles() {
        let mut out = Vec::new();
        let mut menu = spec();
        menu.title = "\x1b[1mTitle\x1b[0m".into();
        render_at_width(&mut out, &menu, 0, None, 40, true).unwrap();
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("\x1b[1mTitle\x1b[0m")
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
                buf.push_str("b\r\n");
                Ok(3)
            }
        }
        let mut reader = OneLine;
        assert_eq!(read_line_selection(&spec(), &mut reader), Ok(1));
    }

    #[test]
    fn removed_directives_report_migration_diagnostics() {
        for directive in ["theme panel", "group \"Core\""] {
            let lines = vec![
                (1, "menu \"Title\" -> $choice".to_string()),
                (2, directive.to_string()),
                (3, "option \"a\" -> \"1\"".to_string()),
                (4, "end".to_string()),
            ];
            let error = parse_spec(&lines, 0, &mut |expr| {
                Ok(crate::Value::Scalar(expr.trim_matches('"').into()))
            })
            .unwrap_err();
            assert_eq!(
                error,
                if directive.starts_with("theme") {
                    "menu theme was removed; remove it because menus use one fixed compact layout"
                } else {
                    "menu group was removed; remove it because menus use one global option list"
                }
            );
        }
    }

    #[test]
    fn runtime_rejects_invalid_and_duplicate_style_directives() {
        for directive in [
            "style spacing 2",
            "style spacing 1\nstyle spacing 0",
            "style color nope red",
            "style color title red\nstyle color title blue",
        ] {
            let expected = "menu style directives were removed; use shared style(...) expressions or shellout \"\" for blank rows";
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
    fn blank_shellout_is_a_visual_row() {
        let lines = vec![
            (1, "menu \"Title\" -> $choice".into()),
            (2, "shellout \"\"".into()),
            (3, "option[a] \"A\" -> \"1\"".into()),
            (4, "end".into()),
        ];
        let (spec, _) = parse_spec(&lines, 0, &mut |expr| {
            Ok(crate::Value::Scalar(expr.trim_matches('\"').into()))
        })
        .unwrap();
        let mut out = Vec::new();
        render_at_width(&mut out, &spec, 0, None, 80, true).unwrap();
        let rendered = String::from_utf8(out).unwrap();
        assert_eq!(spec.blank_rows, 1);
        assert!(rendered.contains("\r\x1b[2K\r\n"));
        assert_eq!(spec.options.len(), 1);
    }

    #[test]
    fn non_blank_menu_shellout_is_rejected() {
        for line in ["shellout \"command\"", "shellout $command"] {
            let lines = vec![
                (1, "menu \"Title\" -> $choice".into()),
                (2, line.into()),
                (3, "option[a] \"A\" -> \"1\"".into()),
                (4, "end".into()),
            ];
            let error = parse_spec(&lines, 0, &mut |_expr| {
                Ok(crate::Value::Scalar("unused".into()))
            })
            .unwrap_err();
            assert!(error.contains("only accepts exactly shellout \"\""));
        }
    }

    #[test]
    #[cfg(any())]
    fn grouped_menu_parses_headings_and_rejects_empty_groups() {
        let lines = vec![
            (1, "menu \"Title\" -> $choice".into()),
            (2, "group \"Core\"".into()),
            (3, "option \"One\" -> \"1\"".into()),
            (4, "group \"Extra\"".into()),
            (5, "option \"Two\" -> \"2\"".into()),
            (6, "end".into()),
        ];
        let (spec, _) = parse_spec(&lines, 0, &mut |expr| {
            Ok(crate::Value::Scalar(expr.trim_matches('"').into()))
        })
        .unwrap();
        assert_eq!(
            spec.groups
                .iter()
                .map(|g| g.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Core", "Extra"]
        );
        assert_eq!(spec.groups[1].start, 1);
        let bad = vec![
            (1, "menu \"Title\" -> $choice".into()),
            (2, "group \"\"".into()),
            (3, "end".into()),
        ];
        assert_eq!(
            parse_spec(&bad, 0, &mut |expr| Ok(crate::Value::Scalar(
                expr.trim_matches('"').into()
            )))
            .unwrap_err(),
            "menu group name must not be empty"
        );
    }

    #[test]
    #[cfg(any())]
    fn grouped_panel_renders_heading() {
        let mut grouped = spec();
        grouped.groups = vec![MenuGroup {
            name: "Core".into(),
            start: 0,
        }];
        let mut out = Vec::new();
        render_at_width(&mut out, &grouped, 0, false, 50).unwrap();
        let rendered = String::from_utf8(out).unwrap();
        assert!(rendered.contains("Core"));
        assert!(rendered.contains("> 1 first option"));
        assert!(
            rendered
                .lines()
                .all(|line| display_width(line.strip_prefix("\r\x1b[2K").unwrap_or(line)) <= 50)
        );
    }

    #[test]
    #[cfg(any())]
    fn dynamic_group_names_are_rejected_as_non_literals() {
        let lines = vec![
            (1, "menu \"Title\" -> $choice".into()),
            (2, "group $name".into()),
            (3, "option \"One\" -> \"1\"".into()),
            (4, "group $other".into()),
            (5, "option \"Two\" -> \"2\"".into()),
            (6, "end".into()),
        ];
        let error = parse_spec(&lines, 0, &mut |_expr| {
            Ok(crate::Value::Scalar("unused".into()))
        })
        .unwrap_err();
        assert_eq!(error, "menu group name must be a string literal");
    }

    #[test]
    fn duplicate_visible_labels_rejected() {
        let options = vec![
            MenuOption {
                key: 'a',
                label: "same".into(),
                value: crate::Value::Scalar("1".into()),
                hint: None,
            },
            MenuOption {
                key: 'b',
                label: "same".into(),
                value: crate::Value::Scalar("2".into()),
                hint: None,
            },
        ];
        assert!(validate_labels(&options).is_err());
    }
}
