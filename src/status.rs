use crate::vcs::VCS;

/// The current VC status
#[derive(PartialEq, Debug)]
pub struct Status {
    /// Version control system
    pub name:       VCS,
    /// VCS symbol
    pub symbol:     String,
    /// The branch name
    pub branch:     String,
    /// Commit hash
    pub commit:     String,
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
    /// Number added chunks
    pub added:      u32,
    /// Number deleted chunks
    pub deleted:    u32,
    /// Ongoing operations (e.g., merging)
    pub operations: Vec<&'static str>,
}

impl Status {
    /// Create a new instance with all values set to default.
    pub fn new(vcs: VCS) -> Status {
        Status {
            name:       vcs,
            symbol:     vcs.default_symbol().to_owned(),
            branch:     String::with_capacity(40),
            commit:     String::with_capacity(40), // Should be max length of git commit hash
            ahead:      0,
            behind:     0,
            staged:     0,
            changed:    0,
            untracked:  0,
            conflicts:  0,
            added:      0,
            deleted:    0,
            operations: vec![],
        }
    }

    /// Returns true if repo has no changes
    pub fn is_clean(&self) -> bool {
        (self.staged == 0 && self.conflicts == 0 && self.changed == 0 && self.untracked == 0)
    }

    /// Format commit hash
    pub fn fmt_commit(&self, len: usize) -> &str {
        if self.commit != "(initial)" {
            return &self.commit[..len];
        }
        &self.commit
    }

    /// Format diff numstat
    pub fn fmt_diff(&self) -> Option<String> {
        if self.changed == 0 {
            return None;
        }
        let mut s = String::with_capacity(10);
        if self.added > 0 {
            // write!(buf, "+{}", self.insertions)?;
            s.push('+');
            s.push_str(&self.added.to_string());
            if self.deleted > 0 {
                s.push_str("/-");
                s.push_str(&self.deleted.to_string());
            }
        }
        Some(s)
    }
}
