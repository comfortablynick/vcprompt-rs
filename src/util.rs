//! Commonly used utilities
use std::{
    io,
    process::{Command, Output, Stdio},
    str,
};

/// The current VC status
#[derive(PartialEq, Debug)]
pub struct Status {
    /// VCS name
    pub name:       String,
    /// VCS symbol
    pub symbol:     String,
    /// The branch name
    pub branch:     String,
    /// Number of revisions we are ahead of upstream
    pub ahead:      u32,
    /// Number of revisions we are behind upstream
    pub behind:     u32,
    /// Number of staged files
    pub staged:     u32,
    /// Number of modified/added/removed files
    pub changed:    u32,
    /// Number of untracked files
    pub untracked:  u32,
    /// Number of conflicts
    pub conflicts:  u32,
    /// Ongoing operations (e.g., merging)
    pub operations: Vec<&'static str>,
}

impl Status {
    /// Create a new instance with all values set to `<unknown>` branch and `0`.
    pub fn new<S>(name: S, symbol: S) -> Status
    where
        S: Into<String>,
    {
        Status {
            name:       name.into(),
            symbol:     symbol.into(),
            branch:     "<unknown>".to_string(),
            ahead:      0,
            behind:     0,
            staged:     0,
            changed:    0,
            untracked:  0,
            conflicts:  0,
            operations: vec![],
        }
    }

    /// Returns true if repo has no changes
    pub fn is_clean(&self) -> bool {
        (self.staged == 0 && self.conflicts == 0 && self.changed == 0 && self.untracked == 0)
    }
}

/// Spawn subprocess for `cmd` and access stdout/stderr
///
/// Fails if process output != 0
pub fn exec(cmd: &[&str]) -> io::Result<Output> {
    let command = Command::new(&cmd[0])
        .args(cmd.get(1..).expect("missing args in cmd"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let result = command.wait_with_output()?;

    if !result.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            str::from_utf8(&result.stderr)
                .unwrap_or("cmd returned non-zero status")
                .trim_end(),
        ));
    }
    Ok(result)
}
