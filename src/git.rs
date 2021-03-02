#[allow(unused)]
use log::{debug, error, info, trace, warn};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GitStatus {
    /// No changes
    Unmodified,
    /// Entry does not exist in old version
    New,
    /// Entry does not exist in new version
    Deleted,
    /// Entry content changed between old and new
    Modified,
    /// Entry was renamed between old and new
    Renamed,
    /// Entry is ignored item in workdir
    Ignored,
    /// Type of entry changed between old and new
    Typechange,
    /// Entry in the index is conflicted
    Conflicted,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GitStagedStatus {
    pub index: GitStatus,
    pub workdir: GitStatus,
}

impl Default for GitStagedStatus {
    fn default() -> Self {
        Self {
            index: GitStatus::Unmodified,
            workdir: GitStatus::Unmodified,
        }
    }
}

impl GitStagedStatus {
    fn new(status: git2::Status) -> Self {
        Self {
            index: match status {
                s if s.contains(git2::Status::INDEX_NEW) => GitStatus::New,
                s if s.contains(git2::Status::INDEX_DELETED) => GitStatus::Deleted,
                s if s.contains(git2::Status::INDEX_MODIFIED) => GitStatus::Modified,
                s if s.contains(git2::Status::INDEX_RENAMED) => GitStatus::Renamed,
                s if s.contains(git2::Status::INDEX_TYPECHANGE) => GitStatus::Typechange,
                _ => GitStatus::Unmodified,
            },

            workdir: match status {
                s if s.contains(git2::Status::WT_NEW) => GitStatus::New,
                s if s.contains(git2::Status::WT_DELETED) => GitStatus::Deleted,
                s if s.contains(git2::Status::WT_MODIFIED) => GitStatus::Modified,
                s if s.contains(git2::Status::WT_RENAMED) => GitStatus::Renamed,
                s if s.contains(git2::Status::IGNORED) => GitStatus::Ignored,
                s if s.contains(git2::Status::WT_TYPECHANGE) => GitStatus::Typechange,
                s if s.contains(git2::Status::CONFLICTED) => GitStatus::Conflicted,
                _ => GitStatus::Unmodified,
            },
        }
    }
}

pub struct GitCache {
    statuses: Vec<(PathBuf, git2::Status)>,
    _cached_dir: Option<PathBuf>,
}

impl GitCache {
    pub fn new(path: &PathBuf) -> GitCache {
        let cachedir = fs::canonicalize(&path).unwrap();
        debug!("GIT PROCESSING {:?}", cachedir);

        let repo = match git2::Repository::discover(&path) {
            Ok(r) => r,
            Err(e) => {
                error!("Error discovering Git repositories: {:?}", e);
                return Self::empty();
            }
        };

        if let Some(workdir) = repo.workdir() {
            let mut statuses = Vec::new();
            info!("Getting Git statuses for repo with workdir {:?}", workdir);
            match repo.statuses(None) {
                Ok(status_list) => {
                    for status_entry in status_list.iter() {
                        let path = workdir.join(Path::new(status_entry.path().unwrap()));
                        let elem = (path, status_entry.status());
                        info!("{:?}", elem);
                        statuses.push(elem);
                    }
                }
                Err(e) => {
                    error!("Error looking up Git statuses: {:?}", e)
                }
            }
            // info!("Cache: {:?}", statuses);
            info!("GitCache path: {:?}", cachedir);

            GitCache {
                statuses,
                _cached_dir: Some(cachedir),
            }
        } else {
            debug!("No workdir");
            Self::empty()
        }
    }

    pub fn empty() -> Self {
        GitCache {
            statuses: Vec::new(),
            _cached_dir: None,
        }
    }

    pub fn get(&self, filepath: &PathBuf) -> GitStagedStatus {
        debug!("Look for {:?}", filepath);
        self.statuses
            .iter()
            .find(|&x| filepath == &x.0)
            .map(|e| GitStagedStatus::new(e.1))
            .unwrap_or(GitStagedStatus::default())
    }
}
