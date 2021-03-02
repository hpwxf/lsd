#[allow(unused)]
use log::{debug, error, info, trace, warn};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GitStatus {
    /// No changes
    Unmodified,
    /// Entry is ignored item in workdir
    Ignored,
    /// Entry does not exist in old version
    New,
    /// Type of entry changed between old and new
    Typechange,
    /// Entry does not exist in new version
    Deleted,
    /// Entry was renamed between old and new
    Renamed,
    /// Entry content changed between old and new
    Modified,
    /// Entry in the index is conflicted
    Conflicted,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GitStagedStatus {
    pub index: GitStatus,
    pub workdir: GitStatus,
}

pub enum StagedArea {
    Index,
    Workdir
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
        info!("Trying to retrieve Git statuses for {:?}", cachedir);

        let repo = match git2::Repository::discover(&path) {
            Ok(r) => r,
            Err(e) => {
                warn!("Git discovery error: {:?}", e);
                return Self::empty();
            }
        };

        if let Some(workdir) = repo.workdir() {
            let mut statuses = Vec::new();
            info!("Retrieving Git statuses for workdir {:?}", workdir);
            match repo.statuses(None) {
                Ok(status_list) => {
                    for status_entry in status_list.iter() {
                        let path = workdir.join(Path::new(status_entry.path().unwrap()));
                        let elem = (path, status_entry.status());
                        debug!("{:?}", elem);
                        statuses.push(elem);
                    }
                }
                Err(e) => {
                    warn!("Git retrieve statuses error: {:?}", e)
                }
            }
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

    pub fn get(&self, filepath: &PathBuf, is_directory: bool) -> GitStagedStatus {
        debug!("Look for [recurse={}] {:?}", is_directory, filepath);

        if is_directory {
            self.statuses
                .iter()
                .filter(|&x| x.0.starts_with(filepath))
                .inspect(|&x| debug!("\t{:?}", x.0))
                .map(|x| GitStagedStatus::new(x.1))
                .fold(GitStagedStatus::default(), |acc, x| GitStagedStatus {
                    index: std::cmp::max(acc.index, x.index),
                    workdir: std::cmp::max(acc.workdir, x.workdir),
                })
        } else {
            self.statuses
                .iter()
                .find(|&x| filepath == &x.0)
                .map(|e| GitStagedStatus::new(e.1))
                .unwrap_or(GitStagedStatus::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_git_status() {
        assert!(GitStatus::Unmodified < GitStatus::Conflicted);
    }
}
