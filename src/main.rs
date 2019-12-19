mod format;
mod git;
mod hg;
mod status;
mod util;
mod vcs;

use crate::{format::OutputStyle, util::globals::*, vcs::VCContext};
use anyhow::{Context, Result};
use getopts::Options;
use log::debug;
use std::env;

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
        // TODO: make this non-optional when finished with testing
        .optflagopt(
            "f",
            "format",
            "format output using this printf-style string",
            "FORMAT_STRING",
        )
        .optflag("m", "minimal", "use minimal format instead of full");
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

    // debug!("Run with args: {:?}", std::env::args());

    let style = if matches.opt_present("m") {
        OutputStyle::Minimal
    } else if matches.opt_present("f") {
        OutputStyle::FormatString
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

        println!(
            "{}",
            format::get_output(&status, style, matches.opt_str("f"))?
        );
    }
    Ok(())
}
