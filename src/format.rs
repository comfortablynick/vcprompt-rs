use crate::{
    status::Status,
    util::{globals::*, logger::*},
};
use anyhow::Result;
use std::{collections::HashMap, env};

/// Available formatting styles
pub enum OutputStyle {
    Detailed,
    Minimal,
    FormatString,
}

/// Get formatted output depending on OutputStyle
pub fn get_output(
    status: &Status,
    style: OutputStyle,
    fmt_string: Option<String>,
) -> Result<String> {
    let variables: HashMap<&'static str, String> = [
        ("VCP_PREFIX", ""),
        ("VCP_SUFFIX", "{reset}"),
        ("VCP_SEPARATOR", "{reset}|"),
        ("VCP_NAME", "{symbol}"),
        ("VCP_BRANCH", "{cyan}{value}{reset}"),
        ("VCP_COMMIT", "{black_on_green}{value}{reset}"),
        ("VCP_OPERATION", "{red}{value}{reset}"),
        ("VCP_BEHIND", "⇣{value}"),
        ("VCP_AHEAD", "⇡{value}"),
        ("VCP_STAGED", "{blue}●{value}"),
        ("VCP_CHANGED", "{yellow}Δ{value}"), // ✚
        ("VCP_CONFLICTS", "{red}‼{value}"),
        ("VCP_UNTRACKED", "{gray}…{value}"),
        ("VCP_CLEAN", "{green}{bold}✔"),
    ]
    .iter()
    .map(|(k, v)| (*k, env::var(k).unwrap_or(v.to_string())))
    .collect();
    debug!("{:?}", variables);

    let mut output = match style {
        OutputStyle::Detailed => format_full(&status, &variables)?,
        OutputStyle::Minimal => format_minimal(&status, &variables)?,
        OutputStyle::FormatString => format_from_string(&status, &variables, fmt_string)?,
    };

    for (k, v) in COLORS.iter() {
        output = output.replace(k, v);
    }
    Ok(output)
}

fn format_from_string(
    status: &Status,
    variables: &HashMap<&'static str, String>,
    fmt_string: Option<String>,
) -> Result<String> {
    let mut output = String::with_capacity(100);
    // TODO: should this be combined with `variables`?
    let fmt_string = fmt_string
        .unwrap_or_else(|| env::var("VCP_FORMAT").unwrap_or_else(|_| String::from("%n %b %o")));
    let mut fmt_string_chars = fmt_string.chars();

    while let Some(c) = fmt_string_chars.next() {
        if c == '%' {
            if let Some(c) = fmt_string_chars.next() {
                match &c {
                    'n' => output.push_str(
                        &variables
                            .get("VCP_NAME")
                            .unwrap()
                            .replace("{value}", &status.name.to_string())
                            .replace("{symbol}", &status.symbol),
                    ),
                    'b' => output.push_str(
                        &variables
                            .get("VCP_BRANCH")
                            .unwrap()
                            .replace("{value}", &status.branch),
                    ),
                    'c' => output.push_str(
                        &variables
                            .get("VCP_COMMIT")
                            .unwrap()
                            .replace("{value}", status.fmt_commit(7)),
                    ),
                    'A' => {
                        if status.ahead > 0 {
                            output.push_str(
                                &variables
                                    .get("VCP_AHEAD")
                                    .unwrap()
                                    .replace("{value}", &status.ahead.to_string()),
                            )
                        }
                    }
                    'B' => {
                        if status.behind > 0 {
                            output.push_str(
                                &variables
                                    .get("VCP_BEHIND")
                                    .unwrap()
                                    .replace("{value}", &status.behind.to_string()),
                            )
                        }
                    }
                    's' => {
                        if status.staged > 0 {
                            output.push_str(
                                &variables
                                    .get("VCP_STAGED")
                                    .unwrap()
                                    .replace("{value}", &status.staged.to_string()),
                            )
                        }
                    }
                    // Unmerged
                    'U' => {
                        if status.conflicts > 0 {
                            output.push_str(
                                &variables
                                    .get("VCP_CONFLICTS")
                                    .unwrap()
                                    .replace("{value}", &status.conflicts.to_string()),
                            )
                        }
                    }
                    // Modified
                    'm' => {
                        if status.changed > 0 {
                            output.push_str(
                                &variables
                                    .get("VCP_CHANGED")
                                    .unwrap()
                                    .replace("{value}", &status.changed.to_string()),
                            )
                        }
                    }
                    'u' => {
                        if status.untracked > 0 {
                            output.push_str(
                                &variables
                                    .get("VCP_UNTRACKED")
                                    .unwrap()
                                    .replace("{value}", &status.untracked.to_string()),
                            )
                        }
                    }
                    'o' => {
                        for op in status.operations.iter() {
                            output.push_str(
                                &variables
                                    .get("VCP_OPERATION")
                                    .unwrap()
                                    .replace("{value}", op),
                            );
                        }
                    }
                    _ => output.push(c),
                }
            }
        } else {
            // Push unchaged to string
            output.push(c);
        }
    }
    if status.is_clean() {
        output.push_str(&variables.get("VCP_CLEAN").unwrap());
    }
    output.push_str(&variables.get("VCP_SUFFIX").unwrap());

    for (k, v) in COLORS.iter() {
        output = output.replace(k, v);
    }
    Ok(output)
}

/// Format *status* in detailed style
/// (`{name}{branch}{branch tracking}|{local status}`).
fn format_full(status: &Status, variables: &HashMap<&'static str, String>) -> Result<String> {
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
fn format_minimal(status: &Status, variables: &HashMap<&'static str, String>) -> Result<String> {
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
