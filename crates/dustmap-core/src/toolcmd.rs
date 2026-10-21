use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolRun {
    pub child_pid: u32,
    pub started_at: DateTime<Utc>,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub stderr: String,
}

impl ToolRun {
    pub fn succeeded(&self) -> bool {
        !self.timed_out && self.exit_code == Some(0)
    }
}

pub fn run(
    argv: &[String],
    timeout: Duration,
    on_start: impl FnOnce(u32, DateTime<Utc>) -> Result<()>,
) -> Result<ToolRun> {
    let (program, args) = argv.split_first().context("tool command argv is empty")?;
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start tool command {program}"))?;
    let child_pid = child.id();
    let started_at = Utc::now();
    if let Err(err) = on_start(child_pid, started_at) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(err);
    }

    let stderr_reader = child.stderr.take().map(|mut stderr| {
        thread::spawn(move || {
            let mut text = String::new();
            let _ = stderr.read_to_string(&mut text);
            text
        })
    });
    let deadline = Instant::now() + timeout;
    let (exit_code, timed_out) = loop {
        if let Some(status) = child.try_wait()? {
            break (status.code(), false);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            break (None, true);
        }
        thread::sleep(POLL_INTERVAL);
    };
    let stderr = stderr_reader
        .and_then(|reader| reader.join().ok())
        .unwrap_or_default();
    Ok(ToolRun {
        child_pid,
        started_at,
        exit_code,
        timed_out,
        stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| part.to_string()).collect()
    }

    #[test]
    fn reports_success_for_zero_exit() {
        let run = run(
            &argv(&["/usr/bin/true"]),
            Duration::from_secs(10),
            |_, _| Ok(()),
        )
        .unwrap();
        assert!(run.succeeded());
        assert_eq!(run.exit_code, Some(0));
    }

    #[test]
    fn reports_failure_for_non_zero_exit() {
        let run = run(
            &argv(&["/usr/bin/false"]),
            Duration::from_secs(10),
            |_, _| Ok(()),
        )
        .unwrap();
        assert!(!run.succeeded());
        assert_eq!(run.exit_code, Some(1));
    }

    #[test]
    fn kills_command_that_exceeds_timeout() {
        let run = run(
            &argv(&["/bin/sleep", "30"]),
            Duration::from_millis(100),
            |_, _| Ok(()),
        )
        .unwrap();
        assert!(run.timed_out);
        assert!(!run.succeeded());
    }

    #[test]
    fn reports_child_pid_before_waiting() {
        let mut seen = None;
        run(
            &argv(&["/usr/bin/true"]),
            Duration::from_secs(10),
            |pid, _| {
                seen = Some(pid);
                Ok(())
            },
        )
        .unwrap();
        assert!(seen.unwrap() > 0);
    }

    #[test]
    fn kills_child_when_start_callback_fails() {
        let result = run(
            &argv(&["/bin/sleep", "30"]),
            Duration::from_secs(10),
            |_, _| Err(anyhow!("journal unavailable")),
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("journal unavailable"));
    }

    #[test]
    fn rejects_empty_argv_and_missing_program() {
        assert!(run(&[], Duration::from_secs(1), |_, _| Ok(())).is_err());
        assert!(run(
            &argv(&["/nonexistent/dustmap-tool"]),
            Duration::from_secs(1),
            |_, _| Ok(())
        )
        .is_err());
    }
}
