mod git;
mod hg;
mod status;
mod util;
mod vcs;
use crate::{status::Status, util::globals::*, vcs::VCContext};
use anyhow::{Context, Result};
use getopts::Options;
use log::debug;
use std::{collections::HashMap, env};

/// Available formatting styles
enum OutputStyle {
    Detailed,
    Minimal,
}

/// Format and print the current VC status
fn print_result(status: &Status, style: OutputStyle) -> Result<String> {
    let mut variables: HashMap<&str, String> = [
        ("VCP_PREFIX", ""),
        ("VCP_SUFFIX", "{reset}"),
        ("VCP_SEPARATOR", "{reset}|"),
        ("VCP_NAME", "{symbol}"), // value|symbol
        ("VCP_BRANCH", "{cyan}{value}{reset}"),
        ("VCP_OPERATION", "{red}{value}{reset}"),
        ("VCP_BEHIND", "⇣{value}"),
        ("VCP_AHEAD", "⇡{value}"),
        ("VCP_STAGED", "{red}●{value}"),
        ("VCP_CONFLICTS", "{red}‼{value}"),
        ("VCP_CHANGED", "{blue}✚{value}"),
        ("VCP_UNTRACKED", "{reset}…{value}"),
        ("VCP_CLEAN", "{green}{bold}✔"),
    ]
    .iter()
    .map(|&(k, v)| (k, v.to_string()))
    .collect();

    for (k, v) in variables.iter_mut() {
        if let Ok(val) = env::var(k) {
            *v = val;
        }
    }

    let mut output = match style {
        OutputStyle::Detailed => format_full(&status, &variables)?,
        OutputStyle::Minimal => format_minimal(&status, &variables)?,
    };

    for (k, v) in COLORS.iter() {
        output = output.replace(k, v);
    }
    Ok(output)
}

fn format(status: &Status) -> Result<String> {
    let mut output = String::with_capacity(100);
    let mut fmt_string = "%n%b%t|%l".chars();
    let mut variables: Vec<(&str, String)> = vec![
        ("VCP_PREFIX", " "),
        ("VCP_SUFFIX", "{reset}"),
        ("VCP_SEPARATOR", "|"),
        ("VCP_NAME", "{symbol}"), // value|symbol
        ("VCP_BRANCH", "{blue}{value}{reset}"),
        ("VCP_OPERATION", "{red}{value}{reset}"),
        ("VCP_BEHIND", "↓{value}"),
        ("VCP_AHEAD", "↑{value}"),
        ("VCP_STAGED", "{blue}✚{value}"),
        ("VCP_CHANGED", "{red}●{value}"),
        ("VCP_CONFLICTS", "{red}✖{value}"),
        ("VCP_UNTRACKED", "{reset}…{value}"),
        ("VCP_CLEAN", "{green}{bold}✔"),
    ]
    .iter()
    .map(|&(k, v)| (k, v.to_string()))
    .collect();
    for (k, v) in variables.iter_mut() {
        if let Ok(val) = env::var(k) {
            *v = val;
        }
    }
    while let Some(c) = fmt_string.next() {
        if c == '%' {
            continue;
        }
        match &c {
            'b' => output.push_str(
                &variables
                    .iter()
                    .find(|x| x.0 == "VCP_BRANCH")
                    .context("Missing VCP_BRANCH")?
                    .1
                    .replace("{value}", &status.branch),
            ),
            'n' => output.push_str(
                &variables
                    .iter()
                    .find(|x| x.0 == "VCP_NAME")
                    .context("Missing VCP_NAME")?
                    .1
                    .replace("{value}", &status.name.to_string())
                    .replace("{symbol}", &status.symbol),
            ),
            _ => (),
        }
    }
    for (k, v) in COLORS.iter() {
        output = output.replace(k, v);
    }
    Ok(output)
}

/// Format *status* in detailed style
/// (`{name}{branch}{branch tracking}|{local status}`).
fn format_full(status: &Status, variables: &HashMap<&str, String>) -> Result<String> {
    let mut output = String::with_capacity(100);
    output.push_str(&variables.get("VCP_PREFIX").unwrap());
    output.push_str(
        &variables
            .get("VCP_NAME")
            .unwrap()
            .replace("{value}", &status.name.to_string())
            .replace("{symbol}", &status.symbol),
    );
    output.push_str(
        &variables
            .get("VCP_BRANCH")
            .unwrap()
            .replace("{value}", &status.branch),
    );
    if status.behind > 0 {
        output.push_str(
            &variables
                .get("VCP_BEHIND")
                .unwrap()
                .replace("{value}", &status.behind.to_string()),
        );
    }
    if status.ahead > 0 {
        output.push_str(
            &variables
                .get("VCP_AHEAD")
                .unwrap()
                .replace("{value}", &status.ahead.to_string()),
        );
    }
    for op in status.operations.iter() {
        output.push_str(&variables.get("VCP_SEPARATOR").unwrap());
        output.push_str(
            &variables
                .get("VCP_OPERATION")
                .unwrap()
                .replace("{value}", op),
        );
    }
    output.push_str(&variables.get("VCP_SEPARATOR").unwrap());
    if status.staged > 0 {
        output.push_str(
            &variables
                .get("VCP_STAGED")
                .unwrap()
                .replace("{value}", &status.staged.to_string()),
        );
    }
    if status.conflicts > 0 {
        output.push_str(
            &variables
                .get("VCP_CONFLICTS")
                .unwrap()
                .replace("{value}", &status.conflicts.to_string()),
        );
    }
    if status.changed > 0 {
        output.push_str(
            &variables
                .get("VCP_CHANGED")
                .unwrap()
                .replace("{value}", &status.changed.to_string()),
        );
    }
    if status.untracked > 0 {
        output.push_str(
            &variables
                .get("VCP_UNTRACKED")
                .unwrap()
                .replace("{value}", &status.untracked.to_string()),
        );
    }
    if status.is_clean() {
        output.push_str(&variables.get("VCP_CLEAN").unwrap());
    }
    output.push_str(&variables.get("VCP_SUFFIX").unwrap());
    Ok(output)
}

/// Format status in minimal style
fn format_minimal(status: &Status, variables: &HashMap<&str, String>) -> Result<String> {
    let mut output = String::with_capacity(100);
    output.push_str(&variables.get("VCP_PREFIX").unwrap());
    output.push_str(
        &variables
            .get("VCP_BRANCH")
            .unwrap()
            .replace("{value}", &status.branch),
    );
    if status.staged > 0 {
        output.push_str("{bold}{yellow}+{reset}");
    }
    if !status.is_clean() {
        output.push_str("{red}*{reset}");
    }
    if status.behind > 0 {
        output.push_str(
            &variables
                .get("VCP_BEHIND")
                .unwrap()
                .replace("{value}", &status.behind.to_string()),
        );
    }
    if status.ahead > 0 {
        output.push_str(
            &variables
                .get("VCP_AHEAD")
                .unwrap()
                .replace("{value}", &status.ahead.to_string()),
        );
    }
    output.push_str(&variables.get("VCP_SUFFIX").unwrap());

    Ok(output)
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!(
        "Usage: {} [options] <DIRECTORY>\n\n{}",
        program, DESCRIPTION
    );
    eprint!("{}", opts.usage(&brief));
}

fn main() -> Result<()> {
    let mut args = env::args();
    let program = args.next().context("Error getting cli args")?;

    let mut opts = Options::new();
    // Build options object
    opts.optflag("h", "help", "print this help message and exit")
        .optflag("V", "version", "print version info and exit")
        .optflagmulti(
            "v",
            "verbose",
            "increase debug verbosity (-v, -vv, -vvv, etc.)",
        )
        // program options
        .optflag("m", "minimal", "use minimal format instead of full")
        .optflag("t", "test", "use test function");
    let matches = match opts.parse(args) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}\n\n{}", e, opts.short_usage(&program));
            std::process::exit(1);
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(());
    }
    if matches.opt_present("V") {
        println!("{} v{}", program, VERSION);
        return Ok(());
    }

    env_logger::Builder::from_env(env_logger::Env::new().default_filter_or(
        match matches.opt_count("v") {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        },
    ))
    .init();

    // TODO: check to see if this is only executed when
    // log level == debug
    debug!("Run with args: {:?}", std::env::args());

    let style = if matches.opt_present("m") {
        OutputStyle::Minimal
    } else {
        OutputStyle::Detailed
    };

    if let Some(dir) = matches.free.get(0) {
        debug!("Changing dir to {}", dir);
        env::set_current_dir(dir)?;
    }

    if let Some(vcs) = VCContext::get_vcs() {
        debug!("{:?}", vcs);
        let status = vcs.get_status()?;
        debug!("Status: {:#?}", &status);

        if matches.opt_present("test") {
            println!("{}", format(&status)?);
            return Ok(());
        }
        println!("{}", print_result(&status, style)?);
    }
    Ok(())
}
