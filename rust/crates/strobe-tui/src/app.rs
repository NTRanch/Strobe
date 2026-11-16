//! Screen state machine + run control (plan / auto).
//!
//! Invalid states are unrepresentable: a run can only execute from a
//! `StagedRun` value; in plan mode staging creates that value and
//! nothing executes until `approve` consumes it; `abort` discards it.

use serde_json::Value;

use crate::facade::{OpKind, OpsFacade, RunRecord, StagedRun, Target};
use crate::{AUDIT_LINES_MAX, LOG_LINES_MAX};

/// The four screens of rung 1. No Media screen: scene screenshots are
/// a follow-on and are not stubbed (task ruling).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    /// Workspace crates, versions, last build/smoke states.
    Overview,
    /// Build / check / smoke with target selection and plan/auto.
    Ops,
    /// Bounded scrolling run output.
    Log,
    /// Fleet panel: trigger rules empty-by-design, audit present.
    Fleet,
}

impl Screen {
    /// All screens in tab order.
    pub const ALL: [Self; 4] = [Self::Overview, Self::Ops, Self::Log, Self::Fleet];
    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Ops => "Ops",
            Self::Log => "Log",
            Self::Fleet => "Fleet",
        }
    }
}

/// Run mode: plan stages for approval; auto executes directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunMode {
    /// Stage the exact command line; wait for approval.
    Plan,
    /// Execute declared operations directly.
    Auto,
}

/// Application state.
pub struct App {
    /// Current screen.
    pub screen: Screen,
    /// Current run mode.
    pub mode: RunMode,
    /// Selected operation index into `OpKind::ALL`.
    pub op_idx: usize,
    /// Selected target index into `Target::ALL`.
    pub target_idx: usize,
    /// Staged run awaiting approval (plan mode only).
    pub staged: Option<StagedRun>,
    /// Bounded log lines.
    pub log: Vec<String>,
    /// Completed runs, newest last.
    pub runs: Vec<RunRecord>,
    /// Set when the user asks to quit.
    pub quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::Overview,
            mode: RunMode::Plan,
            op_idx: 0,
            target_idx: 0,
            staged: None,
            log: Vec::new(),
            runs: Vec::new(),
            quit: false,
        }
    }
}

impl App {
    /// Selected operation.
    pub fn op(&self) -> OpKind {
        OpKind::ALL[self.op_idx % OpKind::ALL.len()]
    }
    /// Selected target.
    pub fn target(&self) -> Target {
        Target::ALL[self.target_idx % Target::ALL.len()]
    }

    fn push_log(&mut self, line: String) {
        self.log.push(line);
        if self.log.len() > LOG_LINES_MAX {
            let excess = self.log.len() - LOG_LINES_MAX;
            self.log.drain(..excess);
        }
    }

    /// Stage the current selection. Executes nothing.
    pub fn stage_current(&mut self, facade: &dyn OpsFacade) {
        let staged = facade.stage(self.op(), self.target());
        self.push_log(format!("staged: {}", staged.display()));
        self.staged = Some(staged);
    }

    /// Approve and execute the staged run. No-op without a stage.
    pub fn approve(&mut self, facade: &mut dyn OpsFacade) {
        let Some(staged) = self.staged.take() else {
            self.push_log("approve: nothing staged".into());
            return;
        };
        self.execute(facade, staged);
    }

    /// Abort the staged run; nothing executes.
    pub fn abort(&mut self) {
        if self.staged.take().is_some() {
            self.push_log("aborted staged run".into());
        }
    }

    /// The shared execute path for plan-approve and auto mode.
    pub fn run_current(&mut self, facade: &mut dyn OpsFacade) {
        match self.mode {
            RunMode::Plan => self.stage_current(facade),
            RunMode::Auto => {
                let staged = facade.stage(self.op(), self.target());
                self.execute(facade, staged);
            }
        }
    }

    fn execute(&mut self, facade: &mut dyn OpsFacade, staged: StagedRun) {
        self.push_log(format!("run: {}", staged.display()));
        let record = facade.execute(&staged);
        for line in record.outcome.output.lines() {
            self.push_log(line.to_string());
        }
        if record.outcome.timed_out {
            self.push_log("run: TIMED OUT (killed at deadline)".into());
        }
        if record.outcome.truncated {
            self.push_log("run: output truncated at cap".into());
        }
        if let Some(err) = &record.outcome.error {
            self.push_log(format!("run: error: {err}"));
        }
        self.push_log(format!(
            "done: {} {} -> {}",
            record.op.name(),
            record.target.name(),
            if record.succeeded() { "ok" } else { "FAILED" }
        ));
        self.runs.push(record);
        self.screen = Screen::Log;
    }

    /// Handle one line-mode command; returns a response line.
    /// Shares the same stage/approve/execute path as the full-screen UI.
    pub fn handle_line(&mut self, facade: &mut dyn OpsFacade, line: &str) -> String {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.as_slice() {
            ["quit"] | ["exit"] => {
                self.quit = true;
                "bye".into()
            }
            ["status"] => facade.status().to_string(),
            ["tools"] => facade.tools().to_string(),
            ["fleet"] => {
                let audit = facade.audit_tail();
                format!(
                    "rules: 0 (empty-by-design)\naudit lines: {}\n{}",
                    audit.len().min(AUDIT_LINES_MAX),
                    audit.join("\n")
                )
            }
            ["mode", "plan"] => {
                self.mode = RunMode::Plan;
                "mode: plan".into()
            }
            ["mode", "auto"] => {
                self.mode = RunMode::Auto;
                "mode: auto".into()
            }
            ["stage", op, target] | ["run", op, target] => {
                let (Some(op), Some(target)) = (OpKind::parse(op), Target::parse(target)) else {
                    return "error: unknown op or target".into();
                };
                self.op_idx = OpKind::ALL.iter().position(|o| *o == op).unwrap_or(0);
                self.target_idx = Target::ALL.iter().position(|t| *t == target).unwrap_or(0);
                if parts[0] == "stage" || self.mode == RunMode::Plan {
                    self.stage_current(facade);
                    format!("staged: {}", self.staged.as_ref().map(|s| s.display()).unwrap_or_default())
                } else {
                    self.run_current(facade);
                    "ran".into()
                }
            }
            ["approve"] => {
                self.approve(facade);
                "approved".into()
            }
            ["abort"] => {
                self.abort();
                "aborted".into()
            }
            ["log"] => self.log.join("\n"),
            _ => "error: commands: status|tools|fleet|mode plan|auto|stage <op> <target>|run <op> <target>|approve|abort|log|quit".into(),
        }
    }

    /// Status value enriched with local run history for Overview.
    pub fn overview(&mut self, facade: &mut dyn OpsFacade) -> Value {
        facade.status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facade::FakeFacade;
    use crate::runner::RunOutcome;

    #[test]
    fn plan_mode_stages_without_executing() {
        let mut fake = FakeFacade::default();
        let mut app = App::default();
        let response = app.handle_line(&mut fake, "run check core");
        assert!(response.starts_with("staged: cargo check"));
        assert!(app.staged.is_some());
        // The proof: no execute call reached the facade.
        assert!(fake.calls.iter().all(|c| !c.starts_with("execute")));
    }

    #[test]
    fn approve_executes_exactly_once() {
        let mut fake = FakeFacade::default();
        let mut app = App::default();
        app.handle_line(&mut fake, "stage build bridge");
        app.handle_line(&mut fake, "approve");
        assert_eq!(
            fake.calls
                .iter()
                .filter(|c| c.starts_with("execute"))
                .count(),
            1
        );
        assert!(app.staged.is_none());
    }

    #[test]
    fn approval_then_abort_executes_nothing() {
        let mut fake = FakeFacade::default();
        let mut app = App::default();
        app.handle_line(&mut fake, "stage check game");
        app.handle_line(&mut fake, "abort");
        app.handle_line(&mut fake, "approve");
        assert!(fake.calls.iter().all(|c| !c.starts_with("execute")));
    }

    #[test]
    fn auto_mode_runs_directly() {
        let mut fake = FakeFacade::default();
        let mut app = App::default();
        app.handle_line(&mut fake, "mode auto");
        app.handle_line(&mut fake, "run check core");
        assert_eq!(
            fake.calls
                .iter()
                .filter(|c| c.starts_with("execute"))
                .count(),
            1
        );
    }

    #[test]
    fn failed_smoke_shows_as_failure_in_log() {
        let mut fake = FakeFacade::default();
        fake.push_outcome(RunOutcome {
            exit_code: Some(0),
            timed_out: false,
            truncated: false,
            output: "not smoke output at all".into(),
            error: None,
        });
        let mut app = App::default();
        app.handle_line(&mut fake, "mode auto");
        app.handle_line(&mut fake, "run smoke game");
        assert!(app.log.iter().any(|l| l.contains("FAILED")));
        assert!(!app.runs[0].succeeded());
    }

    #[test]
    fn unknown_command_is_error_not_panic() {
        let mut fake = FakeFacade::default();
        let mut app = App::default();
        assert!(
            app.handle_line(&mut fake, "frobnicate !!!")
                .starts_with("error:")
        );
    }

    #[test]
    fn log_is_bounded() {
        let mut app = App::default();
        for i in 0..(LOG_LINES_MAX + 50) {
            app.push_log(format!("line {i}"));
        }
        assert_eq!(app.log.len(), LOG_LINES_MAX);
    }
}
