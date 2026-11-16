//! The narrow facade the UI programs against (DESIGN §D2).
//!
//! The real facade wraps this workspace's own operations — cargo
//! build/check of the workspace crates and the `strobe-smoke` boot
//! smoke — through the bounded runner, and reads bridge/core state
//! through the `strobe-bridge` / `strobe-core` libraries. The fake is
//! in-memory, scripted, and records every call for tests.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Duration;

use serde_json::{Value, json};
use strobe_bridge::{Gate, GateConfig};

use crate::runner::{RunOutcome, run_bounded};
use crate::{ARGV_ITEMS_MAX, RUN_DEADLINE_SECS};

/// A workspace operation the console can run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpKind {
    /// `cargo check` the selected target.
    Check,
    /// `cargo build` the selected target.
    Build,
    /// Run the rendered boot smoke (`strobe-smoke`).
    Smoke,
}

impl OpKind {
    /// Parse a line-mode / CLI name.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "check" => Some(Self::Check),
            "build" => Some(Self::Build),
            "smoke" => Some(Self::Smoke),
            _ => None,
        }
    }
    /// Stable name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Check => "check",
            Self::Build => "build",
            Self::Smoke => "smoke",
        }
    }
    /// All operations, in display order.
    pub const ALL: [Self; 3] = [Self::Check, Self::Build, Self::Smoke];
}

/// A selectable run target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// The whole workspace.
    Workspace,
    /// `strobe-core` only.
    Core,
    /// `strobe-bridge` only.
    Bridge,
    /// `strobe-game` only.
    Game,
    /// `strobe-tui` only.
    Tui,
}

impl Target {
    /// Parse a line-mode / CLI name.
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "workspace" => Some(Self::Workspace),
            "core" | "strobe-core" => Some(Self::Core),
            "bridge" | "strobe-bridge" => Some(Self::Bridge),
            "game" | "strobe-game" => Some(Self::Game),
            "tui" | "strobe-tui" => Some(Self::Tui),
            _ => None,
        }
    }
    /// Stable name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::Core => "strobe-core",
            Self::Bridge => "strobe-bridge",
            Self::Game => "strobe-game",
            Self::Tui => "strobe-tui",
        }
    }
    /// Cargo `-p` package name; `None` for the whole workspace.
    pub fn package(self) -> Option<&'static str> {
        match self {
            Self::Workspace => None,
            Self::Core => Some("strobe-core"),
            Self::Bridge => Some("strobe-bridge"),
            Self::Game => Some("strobe-game"),
            Self::Tui => Some("strobe-tui"),
        }
    }
    /// All targets, in display order.
    pub const ALL: [Self; 5] = [
        Self::Workspace,
        Self::Core,
        Self::Bridge,
        Self::Game,
        Self::Tui,
    ];
}

/// A staged run: the exact command line, shown for approval in plan
/// mode before anything executes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StagedRun {
    /// Operation kind.
    pub op: OpKind,
    /// Selected target.
    pub target: Target,
    /// Exact argv, no shell.
    pub argv: Vec<String>,
}

impl StagedRun {
    /// The command line as one display string.
    pub fn display(&self) -> String {
        self.argv.join(" ")
    }
}

/// Parsed verdict of a smoke run's output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmokeVerdict {
    /// The smoke reported its hub variant and a successful exit.
    Passed,
    /// The smoke ran but reported failure, or its output was
    /// malformed / incomplete — shown honestly as a failure.
    Failed,
}

/// Parse smoke output honestly: only the exact success markers pass;
/// anything malformed or partial is a failure.
pub fn parse_smoke(output: &str, exit_code: Option<i32>) -> SmokeVerdict {
    let has_variant = output
        .lines()
        .any(|l| l.starts_with("strobe-smoke: hub variant key:"));
    let has_success = output
        .lines()
        .any(|l| l.contains("strobe-smoke: exit: Success"));
    if has_variant && has_success && exit_code == Some(0) {
        SmokeVerdict::Passed
    } else {
        SmokeVerdict::Failed
    }
}

/// One completed run, recorded for the Overview and Log screens.
#[derive(Clone, Debug)]
pub struct RunRecord {
    /// Operation kind.
    pub op: OpKind,
    /// Selected target.
    pub target: Target,
    /// Bounded subprocess outcome.
    pub outcome: RunOutcome,
    /// Smoke verdict when `op` is smoke.
    pub smoke_verdict: Option<SmokeVerdict>,
}

impl RunRecord {
    /// True when the run succeeded (and, for smoke, parsed as passed).
    pub fn succeeded(&self) -> bool {
        if self.outcome.timed_out || self.outcome.error.is_some() {
            return false;
        }
        if self.op == OpKind::Smoke {
            return self.smoke_verdict == Some(SmokeVerdict::Passed);
        }
        self.outcome.exit_code == Some(0)
    }
}

/// Build the exact argv for an operation/target pair.
pub fn staged_argv(op: OpKind, target: Target) -> Vec<String> {
    let mut argv: Vec<String> = Vec::new();
    match op {
        OpKind::Check | OpKind::Build => {
            argv.push("cargo".into());
            argv.push(op.name().into());
            argv.push("--locked".into());
            if let Some(pkg) = target.package() {
                argv.push("-p".into());
                argv.push(pkg.into());
            }
        }
        OpKind::Smoke => {
            // The smoke binary requires strobe-game's `render` feature.
            for part in [
                "cargo",
                "run",
                "--locked",
                "-p",
                "strobe-game",
                "--features",
                "render",
                "--bin",
                "strobe-smoke",
                "--",
                "--smoke",
                "120",
            ] {
                argv.push(part.into());
            }
        }
    }
    debug_assert!(argv.len() <= ARGV_ITEMS_MAX);
    argv
}

/// The trait the UI programs against.
pub trait OpsFacade {
    /// Workspace status as JSON: crates + versions, bridge/core state.
    fn status(&mut self) -> Value;
    /// This member's own operation list as JSON (the D2 `tools()` slot;
    /// Strobe is a game workspace, not an MCP server, so these are ops).
    fn tools(&self) -> Value;
    /// Stage a run without executing it (plan mode).
    fn stage(&self, op: OpKind, target: Target) -> StagedRun;
    /// Execute a previously staged run.
    fn execute(&mut self, staged: &StagedRun) -> RunRecord;
    /// Fleet trigger audit tail: rules are empty-by-design tonight, so
    /// this is an honest empty list with the reason recorded.
    fn audit_tail(&self) -> Vec<String>;
}

/// Real facade: subprocesses in the workspace root + library state.
pub struct RealFacade {
    workspace_root: PathBuf,
    last: Vec<RunRecord>,
}

impl RealFacade {
    /// Create a facade rooted at the Strobe `rust/` workspace dir.
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            workspace_root,
            last: Vec::new(),
        }
    }
    /// Most recent runs, newest last.
    pub fn last_runs(&self) -> &[RunRecord] {
        &self.last
    }
}

fn crate_versions() -> Value {
    json!([
        {"name": "strobe-core", "version": env!("CARGO_PKG_VERSION")},
        {"name": "strobe-bridge", "version": "0.1.0"},
        {"name": "strobe-game", "version": "0.1.0"},
        {"name": "strobe-tui", "version": env!("CARGO_PKG_VERSION")},
    ])
}

fn bridge_state_json() -> Value {
    let gate = Gate::new(GateConfig::default());
    json!({
        "state": format!("{:?}", gate.state()),
        "armed": gate.armed(),
        "output_active": gate.output_active(),
    })
}

impl OpsFacade for RealFacade {
    fn status(&mut self) -> Value {
        let stages = strobe_core::W5Stage::from_discriminant(0).is_ok();
        json!({
            "member": "strobe",
            "workspace": self.workspace_root.display().to_string(),
            "crates": crate_versions(),
            "bridge": bridge_state_json(),
            "core_w5_parse_ok": stages,
            "last_runs": self.last.iter().map(|r| json!({
                "op": r.op.name(), "target": r.target.name(), "succeeded": r.succeeded(),
                "exit_code": r.outcome.exit_code, "timed_out": r.outcome.timed_out,
            })).collect::<Vec<_>>(),
        })
    }

    fn tools(&self) -> Value {
        json!(OpKind::ALL.iter().map(|op| op.name()).collect::<Vec<_>>())
    }

    fn stage(&self, op: OpKind, target: Target) -> StagedRun {
        StagedRun {
            op,
            target,
            argv: staged_argv(op, target),
        }
    }

    fn execute(&mut self, staged: &StagedRun) -> RunRecord {
        let outcome = run_bounded(
            &staged.argv,
            &self.workspace_root,
            Duration::from_secs(RUN_DEADLINE_SECS),
        );
        let smoke_verdict =
            (staged.op == OpKind::Smoke).then(|| parse_smoke(&outcome.output, outcome.exit_code));
        let record = RunRecord {
            op: staged.op,
            target: staged.target,
            outcome,
            smoke_verdict,
        };
        self.last.push(record.clone());
        record
    }

    fn audit_tail(&self) -> Vec<String> {
        Vec::new()
    }
}

/// In-memory scripted facade for tests; records every call.
#[derive(Default)]
pub struct FakeFacade {
    /// Recorded calls, in order (e.g. `execute:check:strobe-core`).
    pub calls: Vec<String>,
    scripted: VecDeque<RunOutcome>,
}

impl FakeFacade {
    /// Script the next execute outcome.
    pub fn push_outcome(&mut self, outcome: RunOutcome) {
        self.scripted.push_back(outcome);
    }
}

impl OpsFacade for FakeFacade {
    fn status(&mut self) -> Value {
        self.calls.push("status".into());
        json!({
            "member": "strobe", "fake": true,
            "crates": crate_versions(),
            "bridge": bridge_state_json(),
            "last_runs": [],
        })
    }

    fn tools(&self) -> Value {
        json!(["check", "build", "smoke"])
    }

    fn stage(&self, op: OpKind, target: Target) -> StagedRun {
        StagedRun {
            op,
            target,
            argv: staged_argv(op, target),
        }
    }

    fn execute(&mut self, staged: &StagedRun) -> RunRecord {
        self.calls.push(format!(
            "execute:{}:{}",
            staged.op.name(),
            staged.target.name()
        ));
        let outcome = self.scripted.pop_front().unwrap_or(RunOutcome {
            exit_code: Some(0),
            timed_out: false,
            truncated: false,
            output: "ok".into(),
            error: None,
        });
        let smoke_verdict =
            (staged.op == OpKind::Smoke).then(|| parse_smoke(&outcome.output, outcome.exit_code));
        RunRecord {
            op: staged.op,
            target: staged.target,
            outcome,
            smoke_verdict,
        }
    }

    fn audit_tail(&self) -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staged_argv_is_exact_for_check_core() {
        assert_eq!(
            staged_argv(OpKind::Check, Target::Core),
            vec!["cargo", "check", "--locked", "-p", "strobe-core"]
        );
    }

    #[test]
    fn malformed_smoke_output_is_failure() {
        assert_eq!(parse_smoke("garbage\n", Some(0)), SmokeVerdict::Failed);
        assert_eq!(parse_smoke("", None), SmokeVerdict::Failed);
        assert_eq!(
            parse_smoke("strobe-smoke: hub variant key: hub\n", Some(0)),
            SmokeVerdict::Failed
        );
    }

    #[test]
    fn wellformed_smoke_output_passes_only_with_exit_zero() {
        let out = "strobe-smoke: hub variant key: hub_unaware\nstrobe-smoke: exit: Success\n";
        assert_eq!(parse_smoke(out, Some(0)), SmokeVerdict::Passed);
        assert_eq!(parse_smoke(out, Some(1)), SmokeVerdict::Failed);
    }

    #[test]
    fn fake_records_execute_calls() {
        let mut fake = FakeFacade::default();
        let staged = fake.stage(OpKind::Build, Target::Bridge);
        let record = fake.execute(&staged);
        assert!(record.succeeded());
        assert_eq!(fake.calls, vec!["execute:build:strobe-bridge"]);
    }
}
