use crate::color::ColoredString;
use crate::git::GitStagedStatus;
use crate::git::StagedArea::{Index, Workdir};
use crate::icon::Icons;
use ansi_term::ANSIString;

#[derive(Clone, Debug)]
pub enum GitFileStatus {
    Ok(GitStagedStatus),
    Err(String),
}

impl GitFileStatus {
    pub fn render(&self, icons: &Icons) -> ColoredString {
        match &self {
            GitFileStatus::Ok(f) => ANSIString::from(std::format!(
                "{}{}",
                icons.git_status_symbol(&f.index, Index),
                icons.git_status_symbol(&f.workdir, Workdir)
            )),
            GitFileStatus::Err(e) => ANSIString::from(e),
        }
    }
}
