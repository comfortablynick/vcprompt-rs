//! Get Git status
use crate::{
    status::Status,
    util::{exec_cmd, CommandOutput},
    vcs::VCS,
};
use anyhow::{Context, Result};
use std::path::PathBuf;

static OPERATIONS: [(&str, &str); 6] = [
    ("rebase-merge", "REBASE"),
    ("rebase-apply", "AM/REBASE"),
    ("MERGE_HEAD", "MERGING"),
    ("CHERRY_PICK_HEAD", "CHERRY-PICKING"),
    ("REVERT_HEAD", "REVERTING"),
    ("BISECT_LOG", "BISECTING"),
];

/// Get the status for the cwd
pub fn status(rootdir: PathBuf) -> Result<Status> {
    let status_output = get_status()?;
    let diff_output = git_diff_numstat()?;
    let mut result = parse_status(&status_output.stdout)?;
    parse_diff(&diff_output.stdout, &mut result);
    get_operations(&mut result.operations, &rootdir);
    Ok(result)
}

fn git_diff_numstat() -> Result<CommandOutput> {
    exec_cmd("git", &["diff", "--numstat"])
}

fn parse_diff(diff: &str, status: &mut Status) {
    for line in diff.lines() {
        let mut split = line.split_whitespace();
        status.added += split.next().unwrap_or_default().parse().unwrap_or(0);
        status.deleted += split.next().unwrap_or_default().parse().unwrap_or(0);
    }
}

/// Run `git status` and return its output.
fn get_status() -> Result<CommandOutput> {
    exec_cmd(
        "git",
        &[
            "status",
            "--porcelain=2",
            "--branch",
            "--untracked-files=normal",
        ],
    )
    // .ok_or_else(|| format_err!("Command failed: `git status'"))
}

/// Parse the output string of `get_status()`.
fn parse_status(status: &str) -> Result<Status> {
    let mut result = Status::new(VCS::Git);

    for line in status.lines() {
        let mut parts = line.split(' ');
        // See https://git-scm.com/docs/git-status
        match parts.next().unwrap_or("") {
            "#" => match parts.next() {
                Some("branch.head") => {
                    result.branch = parts.next().unwrap_or(&"<unknown>").to_string()
                }
                Some("branch.oid") => {
                    result.commit = parts.next().unwrap_or(&"<unknown>").to_string()
                }
                Some("branch.ab") => {
                    result.ahead = parts
                        .next()
                        .unwrap_or("0")
                        .parse::<i32>()
                        .context("Failed to parse")?
                        .abs() as u32;
                    result.behind = parts
                        .next()
                        .unwrap_or("0")
                        .parse::<i32>()
                        .context("Failed to parse")?
                        .abs() as u32;
                }
                _ => (),
            },
            "1" | "2" => {
                if let Some(status) = parts.next() {
                    // We can ignore the submodule state as it is also indicated
                    // by ".M", so we already track it as a change.
                    if !status.starts_with('.') {
                        result.staged += 1;
                    }
                    if !status.ends_with('.') {
                        result.changed += 1;
                    }
                }
            }
            "u" => result.conflicts += 1,
            "?" => result.untracked += 1,
            _ => (),
        }
    }
    Ok(result)
}

/// Look for files that indicate an ongoing operation (e.g., a merge)
/// and update *list* accordingly
fn get_operations(list: &mut Vec<&str>, rootdir: &PathBuf) {
    let mut gitdir = rootdir.clone();
    gitdir.push(".git");
    for &(fname, op) in OPERATIONS.iter() {
        let mut file = gitdir.clone();
        file.push(fname);
        if file.exists() {
            list.push(op);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env::temp_dir,
        fs::{DirBuilder, File},
    };

    use super::*;

    #[test]
    fn parse_status_full() {
        let output = "
# branch.oid dc716b061d9a0bc6a59f4e02d72b9952cce28927
# branch.head master
# branch.upstream origin/master
# branch.ab +1 -2
1 .M <sub> <mH> <mI> <mW> <hH> <hI> modified.txt
1 .D <sub> <mH> <mI> <mW> <hH> <hI> deleted.txt
1 M. <sub> <mH> <mI> <mW> <hH> <hI> staged.txt
1 MM <sub> <mH> <mI> <mW> <hH> <hI> staged_modified.txt
1 MD <sub> <mH> <mI> <mW> <hH> <hI> staged_deleted.txt
1 A. <sub> <mH> <mI> <mW> <hH> <hI> added.txt
1 AM <sub> <mH> <mI> <mW> <hH> <hI> added_modified.txt
1 AD <sub> <mH> <mI> <mW> <hH> <hI> added_deleted.txt
1 D. <sub> <mH> <mI> <mW> <hH> <hI> deleted.txt
1 DM <sub> <mH> <mI> <mW> <hH> <hI> deleted_modified.txt
2 R. <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
2 RM <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
2 RD <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
2 C. <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
2 CM <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
2 CD <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
u UU <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>
? untracked.txt
! ignored.txt
";
        let mut expected = Status::new(VCS::Git);
        expected.branch = "master".to_owned();
        expected.commit = "dc716b061d9a0bc6a59f4e02d72b9952cce28927".to_owned();
        expected.ahead = 1;
        expected.behind = 2;
        expected.staged = 14;
        expected.changed = 11;
        expected.untracked = 1;
        expected.conflicts = 1;
        assert_eq!(parse_status(output).unwrap(), expected);
    }

    #[test]
    fn parse_status_clean() {
        let output = "
# branch.oid dc716b061d9a0bc6a59f4e02d72b9952cce28927
# branch.head master
";
        let mut expected = Status::new(VCS::Git);
        expected.branch = "master".to_owned();
        expected.commit = "dc716b061d9a0bc6a59f4e02d72b9952cce28927".to_owned();
        assert_eq!(parse_status(output).unwrap(), expected);
    }

    #[test]
    fn parse_status_empty() {
        assert_eq!(parse_status("").unwrap(), Status::new(VCS::Git));
    }

    #[test]
    fn detect_merge() {
        let mut result = Vec::<&str>::new();
        let mut rootdir = temp_dir();
        rootdir.push("test-vcprompt");

        let mut path = rootdir.clone();
        path.push(".git");
        DirBuilder::new()
            .recursive(true)
            .create(path.clone())
            .unwrap();
        path.push("MERGE_HEAD");
        File::create(path).unwrap();

        get_operations(&mut result, &rootdir);

        assert_eq!(result, vec!["MERGING"]);
    }
}
