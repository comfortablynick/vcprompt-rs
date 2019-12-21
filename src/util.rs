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

#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
}

/// Execute a command and return the output on stdout and stderr if sucessful
pub fn exec_cmd(cmd: &[&str]) -> Result<CommandOutput> {
    log::debug!("Executing command '{:?}'", cmd);
    let output = Command::new(&cmd[0])
        .args(cmd.get(1..).context("missing args in command")?)
        .output()?;
    let stdout_string = String::from_utf8(output.stdout)?;
    let stderr_string = String::from_utf8(output.stderr)?;

    if !output.status.success() {
        log::warn!("Non-zero exit code '{:?}'", output.status.code());
        log::debug!("stdout: {}", stdout_string);
        log::debug!("stderr: {}", stderr_string);
        format_err!("Failed to execute \"cmd[0]\": {}", stderr_string.trim_end());
    }

    Ok(CommandOutput {
        stdout: stdout_string,
        stderr: stderr_string,
    })
}

pub mod logger {
    // Format output of env_logger buffer
    use chrono::Local;
    use env_logger::{fmt::Color, Env};
    use log::{self, Level};
    use std::io::Write;

    pub use log::{debug, error, info, trace, warn};

    // Colors
    const DIM_CYAN: u8 = 37;
    const DIM_GREEN: u8 = 34;
    const DIM_YELLOW: u8 = 142;
    const DIM_ORANGE: u8 = 130;
    const DIM_MAGENTA: u8 = 127;

    /// Initialize customized instance of env_logger
    pub fn init_logger(verbose: u8) {
        env_logger::Builder::from_env(Env::new().default_filter_or(match verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }))
        .format(|buf, record| {
            let mut level_style = buf.style();
            match record.level() {
                Level::Trace => level_style.set_color(Color::Ansi256(DIM_YELLOW)),
                Level::Debug => level_style.set_color(Color::Ansi256(DIM_CYAN)),
                Level::Info => level_style.set_color(Color::Ansi256(DIM_GREEN)),
                Level::Warn => level_style.set_color(Color::Ansi256(DIM_ORANGE)),
                Level::Error => level_style.set_color(Color::Ansi256(DIM_MAGENTA)),
            };

            let level = level_style.value(format!("{:5}", record.level()));
            let tm_fmt = "%F %H:%M:%S%.3f";
            let time = Local::now().format(tm_fmt);

            let mut dim_white_style = buf.style();
            dim_white_style.set_color(Color::White);

            let mut subtle_style = buf.style();
            subtle_style.set_color(Color::Black).set_intense(true);

            let mut gray_style = buf.style();
            gray_style.set_color(Color::Ansi256(250));

            writeln!(
                buf,
                "\
             {lbracket}\
             {time}\
             {rbracket}\
             {level}\
             {lbracket}\
             {file}\
             {colon}\
             {line_no}\
             {rbracket} \
             {record_args}\
             ",
                lbracket = subtle_style.value("["),
                rbracket = subtle_style.value("]"),
                colon = subtle_style.value(":"),
                file = gray_style.value(record.file().unwrap_or("<unnamed>")),
                time = gray_style.value(time),
                level = level,
                line_no = gray_style.value(record.line().unwrap_or(0)),
                record_args = &record.args(),
            )
        })
        .init();
    }
}
