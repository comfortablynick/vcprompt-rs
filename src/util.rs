//! Commonly used utilities
pub mod globals {
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
    pub const COLORS: [(&str, &str); 12] = [
        ("{reset}", "\x1B[00m"),
        ("{bold}", "\x1B[01m"),
        ("{black}", "\x1B[30m"),
        ("{red}", "\x1B[31m"),
        ("{green}", "\x1B[32m"),
        ("{yellow}", "\x1B[33m"),
        ("{blue}", "\x1B[34m"),
        ("{magenta}", "\x1B[35m"),
        ("{cyan}", "\x1B[36m"),
        ("{gray}", "\x1B[38;5;248m"),
        ("{white}", "\x1B[37m"),
        ("{black_on_green}", "\x1B[48;5;2m\x1B[30m"),
    ];
}
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
