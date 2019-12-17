//! Commonly used utilities
use anyhow::{format_err, Context, Result};
use std::{
    process::{Command, Output, Stdio},
    str,
};

/// Spawn subprocess for `cmd` and access stdout/stderr
///
/// Fails if process output != 0
pub fn exec(cmd: &[&str]) -> Result<Output> {
    let command = Command::new(&cmd[0])
        .args(cmd.get(1..).context("missing args in command")?)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let result = command.wait_with_output()?;

    if !result.status.success() {
        format_err!(
            "{}",
            str::from_utf8(&result.stderr)
                .unwrap_or("cmd returned non-zero status")
                .trim_end()
        );
    }
    Ok(result)
}
