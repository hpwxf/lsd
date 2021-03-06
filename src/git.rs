use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use crate::meta::git_file_status::GitFileStatus;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GitStatus {
    /// No status info
    Default,
    /// No changes (got from git status)
    Unmodified,
    /// Entry is ignored item in workdir
    Ignored,
    /// Entry does not exist in old version (now in stage)
    NewInStage,
    /// Entry does not exist in old version (not in stage)
    NewInWorkdir,
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

    pub fn get(&self, filepath: &PathBuf, is_directory: bool) -> GitFileStatus {
        debug!("Look for [recurse={}] {:?}", is_directory, filepath);

        if is_directory {
            self.statuses
                .iter()
                .filter(|&x| x.0.starts_with(filepath))
                .inspect(|&x| debug!("\t{:?}", x.0))
                .map(|x| GitFileStatus::new(x.1))
                .fold(GitFileStatus::default(), |acc, x| GitFileStatus {
                    index: std::cmp::max(acc.index, x.index),
                    workdir: std::cmp::max(acc.workdir, x.workdir),
                })
        } else {
            self.statuses
                .iter()
                .find(|&x| filepath == &x.0)
                .map(|e| GitFileStatus::new(e.1))
                .unwrap_or(GitFileStatus::default())
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
