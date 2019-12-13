mod git;
mod hg;
mod util;
use crate::util::Status;
use getopts::Options;
use log::debug;
use std::{collections::HashMap, env, path::PathBuf};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");

/// Available formatting styles
enum OutputStyle {
    Detailed,
    Minimal,
}

/// Supported version control systems
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VCS {
    Git,
    Hg,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VCContext {
    system:  VCS,
    rootdir: PathBuf,
}

impl VCContext {
    pub fn defaults() -> [VCContext; 2] {
        [
            VCContext {
                system:  VCS::Git,
                rootdir: PathBuf::from(".git/HEAD"),
            },
            VCContext {
                system:  VCS::Hg,
                rootdir: PathBuf::from(".hg/00changelog.i"),
            },
        ]
    }

    pub fn new(vcs: VCS, rootdir: PathBuf) -> Self {
        Self {
            system: vcs,
            rootdir,
        }
    }

    fn get_status(self) -> Option<Status> {
        match self.system {
            VCS::Git => Some(git::status(self.rootdir)),
            VCS::Hg => Some(hg::status(self.rootdir)),
        }
    }
}

/// Determine the inner most VCS.
///
/// This functions works for nest (sub) repos and always returns
/// the most inner repository type.
fn get_vcs() -> Option<VCContext> {
    let mut cwd = env::current_dir().ok();
    while let Some(path) = cwd {
        for def in VCContext::defaults().iter() {
            let mut fname = path.clone();
            fname.push(&def.rootdir);
            if fname.exists() {
                return Some(VCContext::new(def.system, path));
            }
        }
        cwd = path.parent().map(PathBuf::from);
    }
    None
}

/// Format and print the current VC status
fn print_result(status: &Status, style: OutputStyle) {
    let colors: HashMap<&str, &str> = [
        ("{reset}", "\x1B[00m"),
        ("{bold}", "\x1B[01m"),
        ("{black}", "\x1B[30m"),
        ("{red}", "\x1B[31m"),
        ("{green}", "\x1B[32m"),
        ("{yellow}", "\x1B[33m"),
        ("{blue}", "\x1B[34m"),
        ("{magenta}", "\x1B[35m"),
        ("{cyan}", "\x1B[36m"),
        ("{white}", "\x1B[37m"),
    ]
    .iter()
    .cloned()
    .collect();

    let mut variables: HashMap<&str, String> = [
        ("VCP_PREFIX", " "),
        ("VCP_SUFFIX", "{reset}"),
        ("VCP_SEPARATOR", "|"),
        ("VCP_NAME", "{symbol}"), // value|symbol
        ("VCP_BRANCH", "{blue}{value}{reset}"),
        ("VCP_OPERATION", "{red}{value}{reset}"),
        ("VCP_BEHIND", "↓{value}"),
        ("VCP_AHEAD", "↑{value}"),
        ("VCP_STAGED", "{red}●{value}"),
        ("VCP_CONFLICTS", "{red}✖{value}"),
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
        OutputStyle::Detailed => format_full(&status, &variables),
        OutputStyle::Minimal => format_minimal(&status, &variables),
    };

    for (k, v) in colors.iter() {
        output = output.replace(k, v);
    }
    println!("{}", output);
}

/// Format *status* in detailed style
/// (`{name}{branch}{branch tracking}|{local status}`).
fn format_full(status: &Status, variables: &HashMap<&str, String>) -> String {
    let mut output = String::with_capacity(100);
    output.push_str(&variables.get("VCP_PREFIX").unwrap());
    output.push_str(
        &variables
            .get("VCP_NAME")
            .unwrap()
            .replace("{value}", &status.name)
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

    output
}

/// Format *status* in minimal style
/// (`{branch}{colored_symbol}`).
fn format_minimal(status: &Status, variables: &HashMap<&str, String>) -> String {
    let mut output = String::with_capacity(100);
    output.push_str(&variables.get("VCP_PREFIX").unwrap());
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
    if status.is_clean() {
        output.push_str("{bold}{green}");
    } else if status.staged > 0 {
        output.push_str("{bold}{red}");
    } else {
        output.push_str("{bold}{yellow}");
    }
    output.push_str(&status.symbol);
    output.push_str("{reset}");
    output.push_str(&variables.get("VCP_SUFFIX").unwrap());

    output
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]\n\n{}", program, DESCRIPTION);
    print!("{}", opts.usage(&brief));
}

fn main() -> Result<(), std::io::Error> {
    let args = env::args().collect::<Vec<String>>();
    let program = args[0].clone();
    let mut opts = Options::new();
    // Build options object
    opts.optflag("h", "help", "print this help message and exit")
        .optflag("V", "version", "print version info and exit")
        .optflag("m", "minimal", "use minimal format instead of full")
        .optflagmulti(
            "v",
            "verbose",
            "increase debug verbosity (-v, -vv, -vvv, etc.)",
        );
    let matches = opts.parse(&args[1..]).unwrap();
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

    debug!("Running with args: {:?}", &args[1..]);
    let style = if matches.opt_present("m") {
        OutputStyle::Minimal
    } else {
        OutputStyle::Detailed
    };

    if let Some(vcs) = get_vcs() {
        if let Some(status) = vcs.get_status() {
            print_result(&status, style);
        }
    }
    Ok(())
}
