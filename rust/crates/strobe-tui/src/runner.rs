//! Bounded subprocess runner (Tiger Style OS-boundary rules).
//!
//! Contract: argv is passed without a shell; stdout and stderr are
//! drained on separate threads into a capped buffer; the child is
//! killed and reaped when the deadline passes; output beyond
//! `output_bytes_max` is truncated and the truncation is reported,
//! never silent.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::OUTPUT_BYTES_MAX;

/// Outcome of one bounded run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunOutcome {
    /// Process exit code; `None` when killed or unspawnable.
    pub exit_code: Option<i32>,
    /// True when the deadline fired and the child was killed.
    pub timed_out: bool,
    /// True when output exceeded the cap and was truncated.
    pub truncated: bool,
    /// Combined stdout+stderr, capped at the configured byte limit.
    pub output: String,
    /// Spawn or wait failure, if any.
    pub error: Option<String>,
}

fn drain(stream: impl Read + Send + 'static, sink: Arc<Mutex<Vec<u8>>>, cap: usize) {
    std::thread::spawn(move || {
        let mut reader = stream;
        let mut chunk = [0u8; 4096];
        loop {
            match reader.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(count) => {
                    if let Ok(mut guard) = sink.lock() {
                        let room = cap.saturating_sub(guard.len());
                        let take = room.min(count);
                        guard.extend_from_slice(&chunk[..take]);
                    }
                }
            }
        }
    });
}

fn wait_bounded(child: &mut Child, deadline: Duration) -> (Option<i32>, bool) {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return (status.code(), false),
            Ok(None) => {
                if start.elapsed() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return (None, true);
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(_) => return (None, false),
        }
    }
}

/// Run `argv` in `cwd` with a deadline and an output cap.
///
/// # Failure
/// A spawn failure is reported in [`RunOutcome::error`], never panics.
pub fn run_bounded(argv: &[String], cwd: &std::path::Path, deadline: Duration) -> RunOutcome {
    run_bounded_capped(argv, cwd, deadline, OUTPUT_BYTES_MAX)
}

/// [`run_bounded`] with an explicit output cap (bytes), for tests.
pub fn run_bounded_capped(
    argv: &[String],
    cwd: &std::path::Path,
    deadline: Duration,
    cap: usize,
) -> RunOutcome {
    let Some((program, args)) = argv.split_first() else {
        return RunOutcome {
            exit_code: None,
            timed_out: false,
            truncated: false,
            output: String::new(),
            error: Some("empty argv".to_string()),
        };
    };
    let spawned = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(err) => {
            return RunOutcome {
                exit_code: None,
                timed_out: false,
                truncated: false,
                output: String::new(),
                error: Some(format!("spawn {program}: {err}")),
            };
        }
    };
    let buffer = Arc::new(Mutex::new(Vec::<u8>::new()));
    if let Some(out) = child.stdout.take() {
        drain(out, Arc::clone(&buffer), cap + 1);
    }
    if let Some(err) = child.stderr.take() {
        drain(err, Arc::clone(&buffer), cap + 1);
    }
    let (exit_code, timed_out) = wait_bounded(&mut child, deadline);
    // Give drain threads a brief bounded window to flush final bytes.
    std::thread::sleep(Duration::from_millis(30));
    let bytes = buffer.lock().map(|g| g.clone()).unwrap_or_default();
    let truncated = bytes.len() > cap;
    let kept = &bytes[..bytes.len().min(cap)];
    RunOutcome {
        exit_code,
        timed_out,
        truncated,
        output: String::from_utf8_lossy(kept).into_owned(),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn runaway_child_is_killed_at_deadline() {
        let outcome = run_bounded(
            &argv(&["sh", "-c", "sleep 30"]),
            std::path::Path::new("/tmp"),
            Duration::from_millis(300),
        );
        assert!(outcome.timed_out);
        assert_eq!(outcome.exit_code, None);
    }

    #[test]
    fn output_is_capped_and_truncation_is_reported() {
        let outcome = run_bounded_capped(
            &argv(&["sh", "-c", "printf 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'"]),
            std::path::Path::new("/tmp"),
            Duration::from_secs(5),
            8,
        );
        assert!(!outcome.timed_out);
        assert!(outcome.truncated);
        assert_eq!(outcome.output.len(), 8);
    }

    #[test]
    fn missing_program_is_typed_error_not_panic() {
        let outcome = run_bounded(
            &argv(&["strobe-tui-no-such-program-xyz"]),
            std::path::Path::new("/tmp"),
            Duration::from_secs(5),
        );
        assert!(outcome.error.is_some());
        assert_eq!(outcome.exit_code, None);
    }

    #[test]
    fn empty_argv_is_typed_error() {
        let outcome = run_bounded(&[], std::path::Path::new("/tmp"), Duration::from_secs(1));
        assert_eq!(outcome.error.as_deref(), Some("empty argv"));
    }
}
