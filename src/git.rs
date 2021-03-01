use log::{debug, error, info, trace, warn};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

impl GitStatus {
    fn worktree_new(status: git2::Status) -> GitStatus {
        match status {
            s if s.contains(git2::Status::WT_NEW) => GitStatus::New,
            s if s.contains(git2::Status::WT_DELETED) => GitStatus::Deleted,
            s if s.contains(git2::Status::WT_MODIFIED) => GitStatus::Modified,
            s if s.contains(git2::Status::WT_RENAMED) => GitStatus::Renamed,
            s if s.contains(git2::Status::IGNORED) => GitStatus::Ignored,
            s if s.contains(git2::Status::WT_TYPECHANGE) => GitStatus::Typechange,
            s if s.contains(git2::Status::CONFLICTED) => GitStatus::Conflicted,
            _ => GitStatus::Unmodified,
        }
    }
    fn index_new(status: git2::Status) -> GitStatus {
        match status {
            s if s.contains(git2::Status::INDEX_NEW) => GitStatus::New,
            s if s.contains(git2::Status::INDEX_DELETED) => GitStatus::Deleted,
            s if s.contains(git2::Status::INDEX_MODIFIED) => GitStatus::Modified,
            s if s.contains(git2::Status::INDEX_RENAMED) => GitStatus::Renamed,
            s if s.contains(git2::Status::INDEX_TYPECHANGE) => GitStatus::Typechange,
            _ => GitStatus::Unmodified,
        }
    }
}

impl Default for GitStatus {
    fn default() -> Self {
        GitStatus::Unmodified
    }
}

pub struct GitCache {
    statuses: Vec<(PathBuf, git2::Status)>,
    pub currentdir: PathBuf,
}

impl GitCache {
    pub fn new(path: &PathBuf) -> Result<GitCache, &PathBuf> {
        let cachedir = fs::canonicalize(&path).unwrap();
        info!("GIT PROCESSING {:?}", cachedir);

        let repo = match git2::Repository::discover(&path) {
            Ok(r) => r,
            Err(e) => {
                error!("Error discovering Git repositories: {:?}", e);
                return Err(path);
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

            return Ok(GitCache {
                statuses,
                currentdir: cachedir,
            }); // FIXME unwrap
        }
        return Err(path);
    }

    pub fn get(&self, filepath: &PathBuf) -> (GitStatus, GitStatus) {
        // let filepath = filepath.strip_prefix("./").unwrap();
        //
        info!("Look for {:?}", filepath);
        // let repo = match git2::Repository::discover(&self.currentdir) {
        //     Ok(r) => r,
        //     Err(e) => {
        //         panic!("Error discovering Git repositories: {:?}", e);
        //     }
        // };
        // match repo.status_file(filepath) {
        //     Ok(s) => Some((GitStatus::worktree_new(s), GitStatus::index_new(s))),
        //     Err(e) => {
        //         warn!("Not git found {:?} {:?}", filepath, e);
        //         None
        //     }
        // }

        self.statuses
            .iter()
            .find(|&x| filepath == &x.0)
            .map(|e| (GitStatus::worktree_new(e.1), GitStatus::index_new(e.1)))
            .unwrap_or((GitStatus::default(), GitStatus::default()))
    }
}
