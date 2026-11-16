//! Pure widget constructors with named truncation bounds.
//! Every panel is a pure `(state) -> Paragraph` construction, tested
//! through ratatui's `TestBackend`.

use ratatui::text::{Line, Text};
use ratatui::widgets::Paragraph;

use crate::LINE_CHARS_MAX;

/// Truncate a line to `LINE_CHARS_MAX` characters, marking truncation.
pub fn truncate_line(text: &str) -> String {
    if text.chars().count() <= LINE_CHARS_MAX {
        return text.to_string();
    }
    let kept: String = text.chars().take(LINE_CHARS_MAX - 1).collect();
    format!("{kept}…")
}

/// Build a titled paragraph from bounded lines.
pub fn panel<'a>(title: &'a str, lines: Vec<String>) -> Paragraph<'a> {
    let bounded: Vec<Line> = lines
        .into_iter()
        .map(|l| Line::from(truncate_line(&l)))
        .collect();
    Paragraph::new(Text::from(bounded))
        .block(ratatui::widgets::Block::bordered().title(title.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn truncate_bounds_long_lines() {
        let long = "x".repeat(LINE_CHARS_MAX + 100);
        assert_eq!(truncate_line(&long).chars().count(), LINE_CHARS_MAX);
        assert_eq!(truncate_line("short"), "short");
    }

    #[test]
    fn panel_renders_on_test_backend() {
        let backend = TestBackend::new(60, 12);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal
            .draw(|frame| {
                let widget = panel("Overview", vec!["strobe".into(), "crates: 4".into()]);
                frame.render_widget(widget, frame.area());
            })
            .expect("draw");
        let text = format!("{:?}", terminal.backend().buffer());
        assert!(text.contains("strobe"));
    }
}
