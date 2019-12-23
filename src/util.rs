//! Commonly used utilities
use anyhow::{format_err, Result};
use std::process::Command;

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

#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
}

impl PartialEq for CommandOutput {
    fn eq(&self, other: &Self) -> bool {
        self.stdout == other.stdout && self.stderr == other.stderr
    }
}

/// Execute a command and return the output on stdout and stderr if sucessful
///
/// Most of this borrowed from Starship
/// https://github.com/starship/starship/blob/master/src/utils.rs
///
/// If no arguments, pass empty array slice `&[]`
pub fn exec_cmd(cmd: &str, args: &[&str]) -> Result<CommandOutput> {
    log::trace!("Executing command '{:?}' with args '{:?}'", cmd, args);
    let output = Command::new(cmd).args(args).output()?;
    let stdout_string = String::from_utf8(output.stdout).unwrap_or_default();
    let stderr_string = String::from_utf8(output.stderr).unwrap_or_default();

    if !output.status.success() {
        log::trace!("Non-zero exit code '{:?}'", output.status.code());
        log::trace!("stdout: {}", stdout_string);
        log::trace!("stderr: {}", stderr_string);
        return Err(format_err!(
            "Command failed: `{}': {}",
            stdout_string,
            stderr_string
        ));
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exec_no_output() {
        let result = exec_cmd("true", &[]).unwrap();
        let expected = CommandOutput {
            stdout: String::from(""),
            stderr: String::from(""),
        };
        assert_eq!(result, expected)
    }

    #[test]
    fn exec_with_output_stdout() {
        let result = exec_cmd("/bin/echo", &["-n", "hello"]).unwrap();
        let expected = CommandOutput {
            stdout: String::from("hello"),
            stderr: String::from(""),
        };
        assert_eq!(result, expected)
    }

    #[test]
    fn exec_with_output_stderr() {
        let result = exec_cmd("/bin/sh", &["-c", "echo hello >&2"]).unwrap();
        let expected = CommandOutput {
            stdout: String::from(""),
            stderr: String::from("hello\n"),
        };
        assert_eq!(result, expected)
    }

    #[test]
    fn exec_with_output_both() {
        let result = exec_cmd("/bin/sh", &["-c", "echo hello; echo world >&2"]).unwrap();
        let expected = CommandOutput {
            stdout: String::from("hello\n"),
            stderr: String::from("world\n"),
        };
        assert_eq!(result, expected)
    }

    #[test]
    fn exec_with_non_zero_exit_code() {
        let result = exec_cmd("false", &[]);
        assert!(result.is_err(), "Result wasn't an error")
    }
}
