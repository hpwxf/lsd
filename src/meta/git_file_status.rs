use crate::color::ColoredString;
use crate::git::GitStagedStatus;
use crate::icon::Icons;
use ansi_term::ANSIString;
use crate::git::StagedArea::{Workdir, Index};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitFileStatusOrError(pub Result<GitStagedStatus, String>);

impl GitFileStatusOrError {
    pub fn render(&self, icons: &Icons) -> ColoredString {
        match &self.0 {
            Ok(f) => ANSIString::from(std::format!(
                "{}{}",
                icons.git_status_symbol(&f.index, Index),
                icons.git_status_symbol(&f.workdir, Workdir)
            )),
            Err(e) => ANSIString::from(e),
        }
    }
}
