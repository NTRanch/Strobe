//! The member's screens (DESIGN §D2 core, adapted to a game
//! workspace): Overview, Ops, Log/Live, Fleet. Provider control is
//! honestly absent — this workspace routes no model calls. No Media
//! screen tonight.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

use crate::AUDIT_LINES_MAX;
use crate::app::{App, RunMode, Screen};
use crate::facade::{OpKind, OpsFacade, Target};
use crate::panels::panel;

/// Render the current screen.
pub fn render(frame: &mut Frame, app: &App, facade: &mut dyn OpsFacade) {
    let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(frame.area());
    let tabs: Vec<String> = Screen::ALL
        .iter()
        .map(|s| {
            if *s == app.screen {
                format!("[{}]", s.name())
            } else {
                s.name().to_string()
            }
        })
        .collect();
    frame.render_widget(
        ratatui::widgets::Paragraph::new(format!(
            "strobe-tui  {}   (Tab switch, q quit)",
            tabs.join("  ")
        )),
        chunks[0],
    );
    match app.screen {
        Screen::Overview => overview(frame, chunks[1], app, facade),
        Screen::Ops => ops(frame, chunks[1], app),
        Screen::Log => log(frame, chunks[1], app),
        Screen::Fleet => fleet(frame, chunks[1], facade),
    }
}

fn overview(frame: &mut Frame, area: ratatui::layout::Rect, app: &App, facade: &mut dyn OpsFacade) {
    let status = facade.status();
    let mut lines = vec![
        "member: strobe (dev/ops console)".to_string(),
        format!("workspace: {}", status["workspace"].as_str().unwrap_or("?")),
    ];
    if let Some(crates) = status["crates"].as_array() {
        for krate in crates {
            lines.push(format!(
                "crate: {} {}",
                krate["name"].as_str().unwrap_or("?"),
                krate["version"].as_str().unwrap_or("?")
            ));
        }
    }
    lines.push(format!(
        "bridge: state={} armed={} output_active={}",
        status["bridge"]["state"].as_str().unwrap_or("?"),
        status["bridge"]["armed"],
        status["bridge"]["output_active"],
    ));
    if app.runs.is_empty() {
        lines.push("last build/smoke: none this session".into());
    }
    for run in &app.runs {
        lines.push(format!(
            "last run: {} {} -> {}",
            run.op.name(),
            run.target.name(),
            if run.succeeded() { "ok" } else { "FAILED" }
        ));
    }
    frame.render_widget(panel("Overview", lines), area);
}

fn ops(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let mut lines = vec![
        format!(
            "mode: {} (m toggles; plan stages, auto executes)",
            match app.mode {
                RunMode::Plan => "plan",
                RunMode::Auto => "auto",
            }
        ),
        "provider: absent — this workspace routes no model calls".to_string(),
        String::new(),
    ];
    for (idx, op) in OpKind::ALL.iter().enumerate() {
        let mark = if idx == app.op_idx { ">" } else { " " };
        lines.push(format!("{mark} op: {}", op.name()));
    }
    for (idx, target) in Target::ALL.iter().enumerate() {
        let mark = if idx == app.target_idx { ">" } else { " " };
        lines.push(format!("{mark} target: {}", target.name()));
    }
    lines.push(String::new());
    lines.push("keys: j/k op, h/l target, Enter run/stage, a approve, x abort".into());
    if let Some(staged) = &app.staged {
        lines.push(format!("STAGED (awaiting approval): {}", staged.display()));
    }
    frame.render_widget(panel("Ops", lines), area);
}

fn log(frame: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let lines: Vec<String> = if app.log.is_empty() {
        vec!["no runs yet".into()]
    } else {
        app.log.clone()
    };
    frame.render_widget(panel("Log / Live", lines), area);
}

fn fleet(frame: &mut Frame, area: ratatui::layout::Rect, facade: &mut dyn OpsFacade) {
    let audit = facade.audit_tail();
    let mut lines = vec![
        "trigger rules: 0 (empty-by-design — no rules tonight)".to_string(),
        "audit: present; no firings recorded".to_string(),
    ];
    for line in audit.iter().take(AUDIT_LINES_MAX) {
        lines.push(line.clone());
    }
    frame.render_widget(panel("Fleet", lines), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crate::facade::FakeFacade;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render_text(app: &App) -> String {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let mut fake = FakeFacade::default();
        let mut app_copy = App {
            screen: app.screen,
            ..Default::default()
        };
        app_copy.mode = app.mode;
        terminal
            .draw(|frame| render(frame, &app_copy, &mut fake))
            .expect("draw");
        format!("{:?}", terminal.backend().buffer())
    }

    #[test]
    fn first_screen_renders_headless() {
        let text = render_text(&App::default());
        assert!(text.contains("Overview"));
        assert!(text.contains("strobe-core"));
    }

    #[test]
    fn ops_screen_states_provider_absent() {
        let app = App {
            screen: Screen::Ops,
            ..Default::default()
        };
        assert!(render_text(&app).contains("absent"));
    }

    #[test]
    fn fleet_screen_shows_empty_rules() {
        let app = App {
            screen: Screen::Fleet,
            ..Default::default()
        };
        assert!(render_text(&app).contains("empty-by-design"));
    }
}
