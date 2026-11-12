use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute, terminal,
};
use std::io::{self, IsTerminal, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MenuEvent {
    Up,
    Down,
    Select,
    Resize,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MenuState {
    pub selected: usize,
    pub count: usize,
}

impl MenuState {
    pub(crate) fn new(count: usize, default: Option<usize>) -> Self {
        Self {
            selected: default
                .unwrap_or(1)
                .saturating_sub(1)
                .min(count.saturating_sub(1)),
            count,
        }
    }

    pub(crate) fn apply(&mut self, event: MenuEvent) -> bool {
        if self.count == 0 {
            return false;
        }
        match event {
            MenuEvent::Up => self.selected = (self.selected + self.count - 1) % self.count,
            MenuEvent::Down => self.selected = (self.selected + 1) % self.count,
            MenuEvent::Select => return true,
            _ => {}
        }
        false
    }
}

pub(crate) fn select(
    question: &str,
    labels: &[String],
    default: Option<usize>,
) -> Result<usize, String> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return line_mode(question, labels, default);
    }
    tty_mode(question, labels, default)
}

fn line_mode(question: &str, labels: &[String], _default: Option<usize>) -> Result<usize, String> {
    println!("{question}");
    for (index, label) in labels.iter().enumerate() {
        println!("{}) {label}", index + 1);
    }
    loop {
        print!("Choose: ");
        io::stdout()
            .flush()
            .map_err(|e| format!("failed to flush stdout: {e}"))?;
        let mut input = String::new();
        let bytes = io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("choose input read failed: {e}"))?;
        if bytes == 0 {
            return Err("choose input closed".to_string());
        }
        let input = input.trim_end_matches(['\n', '\r']);
        if let Ok(index) = input.parse::<usize>()
            && (1..=labels.len()).contains(&index)
        {
            return Ok(index - 1);
        }
        if let Some(index) = labels.iter().position(|label| label == input) {
            return Ok(index);
        }
    }
}

struct Cleanup;
impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = execute!(io::stdout(), Show);
    }
}

fn tty_mode(question: &str, labels: &[String], default: Option<usize>) -> Result<usize, String> {
    terminal::enable_raw_mode().map_err(|e| format!("choose raw mode failed: {e}"))?;
    let _cleanup = Cleanup;
    let mut stdout = io::stdout();
    execute!(stdout, Hide).map_err(|e| format!("choose cursor setup failed: {e}"))?;
    let mut state = MenuState::new(labels.len(), default);
    render(&mut stdout, question, labels, &state, false)?;
    loop {
        let event = event::read().map_err(|e| format!("choose event read failed: {e}"))?;
        let menu_event = match event {
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                kind: KeyEventKind::Press,
                ..
            }) => MenuEvent::Up,
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                kind: KeyEventKind::Press,
                ..
            }) => MenuEvent::Down,
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => MenuEvent::Select,
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                ..
            }) => return Err("choose cancelled".to_string()),
            Event::Resize(_, _) => MenuEvent::Resize,
            _ => MenuEvent::Other,
        };
        if state.apply(menu_event) {
            return Ok(state.selected);
        }
        if matches!(
            menu_event,
            MenuEvent::Up | MenuEvent::Down | MenuEvent::Resize
        ) {
            render(&mut stdout, question, labels, &state, true)?;
        }
    }
}

fn render(
    out: &mut impl Write,
    question: &str,
    labels: &[String],
    state: &MenuState,
    redraw: bool,
) -> Result<(), String> {
    if redraw {
        write!(out, "\x1b[{}A", labels.len() + 1)
            .map_err(|e| format!("choose render failed: {e}"))?;
    }
    writeln!(out, "\r\x1b[2K{question}").map_err(|e| format!("choose render failed: {e}"))?;
    for (index, label) in labels.iter().enumerate() {
        let marker = if index == state.selected { ">" } else { " " };
        writeln!(out, "\x1b[2K{marker} {}) {label}", index + 1)
            .map_err(|e| format!("choose render failed: {e}"))?;
    }
    out.flush()
        .map_err(|e| format!("choose render failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_and_navigation_wrap() {
        let mut state = MenuState::new(3, Some(2));
        assert_eq!(state.selected, 1);
        state.apply(MenuEvent::Down);
        assert_eq!(state.selected, 2);
        state.apply(MenuEvent::Down);
        assert_eq!(state.selected, 0);
        state.apply(MenuEvent::Up);
        assert_eq!(state.selected, 2);
    }
    #[test]
    fn escape_is_not_selection() {
        let mut state = MenuState::new(2, None);
        assert!(!state.apply(MenuEvent::Other));
    }
}
