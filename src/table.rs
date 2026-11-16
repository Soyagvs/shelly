#[derive(Clone, Copy)]
pub enum Align {
    Left,
    Center,
    Right,
}
#[derive(Clone, Copy)]
pub enum Border {
    Ascii,
    Unicode,
    Rounded,
}

pub struct Table {
    pub headers: Vec<(String, Align)>,
    pub rows: Vec<Vec<String>>,
    pub footer: Option<Vec<String>>,
    pub border: Border,
}

pub fn parse_cells(text: &str) -> Result<Vec<&str>, String> {
    let mut out = Vec::new();
    let mut start = None;
    let mut quote = false;
    let mut depth = 0;
    for (i, c) in text.char_indices() {
        if c == '"' {
            quote = !quote;
        }
        if !quote {
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
            }
        }
        if start.is_none() && !c.is_whitespace() {
            start = Some(i);
        }
        if start.is_some() && c.is_whitespace() && !quote && depth == 0 {
            out.push(&text[start.take().unwrap()..i]);
        }
    }
    if let Some(s) = start {
        out.push(&text[s..]);
    }
    if out.is_empty() {
        return Err("table row requires at least one cell".into());
    }
    if quote || depth != 0 {
        return Err("malformed table cell expression".into());
    }
    Ok(out)
}

fn width(s: &str) -> usize {
    let mut n = 0;
    let mut esc = false;
    let mut ansi = false;
    for c in s.chars() {
        if esc {
            if c == 'm' {
                esc = false;
                ansi = false;
            }
            continue;
        }
        if c == '\x1b' {
            ansi = true;
            esc = true;
            continue;
        }
        if ansi {
            continue;
        }
        n += if c == '\u{200d}' || ('\u{300}'..='\u{36f}').contains(&c) {
            0
        } else if c as u32 >= 0x1100 {
            2
        } else {
            1
        };
    }
    n
}
fn pad(s: &str, target: usize, align: Align) -> String {
    let w = width(s);
    let d = target - w;
    match align {
        Align::Left => format!("{s}{}", " ".repeat(d)),
        Align::Right => format!("{}{s}", " ".repeat(d)),
        Align::Center => format!("{}{}{}", " ".repeat(d / 2), s, " ".repeat(d - d / 2)),
    }
}

pub fn render(t: &Table) -> String {
    let cols = t.headers.len();
    let mut widths = t.headers.iter().map(|(s, _)| width(s)).collect::<Vec<_>>();
    for row in t.rows.iter().chain(t.footer.iter()) {
        for (i, s) in row.iter().enumerate() {
            widths[i] = widths[i].max(width(s));
        }
    }
    let (tl, tm, tr, ml, mm, mr, bl, bm, br, h, vl, vr) = match t.border {
        Border::Ascii => ('+', '+', '+', '+', '+', '+', '+', '+', '+', '-', '|', '|'),
        Border::Unicode => ('┌', '┬', '┐', '├', '┼', '┤', '└', '┴', '┘', '─', '│', '│'),
        Border::Rounded => ('╭', '┬', '╮', '├', '┼', '┤', '╰', '┴', '╯', '─', '│', '│'),
    };
    let line = |left: char, middle: char, right: char| {
        format!(
            "{}{}{}\n",
            left,
            widths
                .iter()
                .map(|w| std::iter::repeat_n(h, *w + 2).collect::<String>())
                .collect::<Vec<_>>()
                .join(&middle.to_string()),
            right
        )
    };
    let mut out = line(tl, tm, tr);
    let headers = t.headers.iter().map(|(s, _)| s.clone()).collect::<Vec<_>>();
    for (kind, row) in [(0, headers.as_slice())]
        .into_iter()
        .chain(t.rows.iter().map(|r| (1, r.as_slice())))
    {
        out.push(vl);
        for (i, s) in row.iter().enumerate() {
            out.push(' ');
            out.push_str(&pad(
                s,
                widths[i],
                if kind == 0 {
                    t.headers[i].1
                } else {
                    Align::Left
                },
            ));
            out.push(' ');
            if i + 1 < cols {
                out.push(vl);
            }
        }
        out.push(vr);
        out.push('\n');
        if kind == 0 && !t.rows.is_empty() {
            out.push_str(&line(ml, mm, mr));
        }
    }

    if let Some(row) = &t.footer {
        out.push_str(&line(ml, mm, mr));
        out.push(vl);
        for (i, s) in row.iter().enumerate() {
            out.push(' ');
            out.push_str(&pad(s, widths[i], t.headers[i].1));
            out.push(' ');
            if i + 1 < cols {
                out.push(vl);
            }
        }
        out.push(vr);
        out.push('\n');
    }
    out.push_str(&line(bl, bm, br));
    out
}
