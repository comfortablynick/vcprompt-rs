use crate::{git, hg, util::Status};
use anyhow::Result;
use std::{env, path::PathBuf};

/// Supported version control systems
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VCS {
    Git,
    Hg,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VCContext {
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

    /// Create new instance of VCContext
    pub fn new<P>(vcs: VCS, rootdir: P) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            system:  vcs,
            rootdir: rootdir.into(),
        }
    }

    /// Determine the inner most VCS.
    ///
    /// This functions works for nest (sub) repos and always returns
    /// the most inner repository type.
    pub fn get_vcs() -> Option<Self> {
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

    pub fn get_status(self) -> Result<Status> {
        match self.system {
            VCS::Git => git::status(self.rootdir),
            VCS::Hg => hg::status(self.rootdir),
        }
    }
}
