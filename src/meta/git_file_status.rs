use crate::color::{ColoredString, Colors, Elem};
use crate::git::GitStatus;
use crate::icon::Icons;
use ansi_term::ANSIStrings;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GitFileStatus {
    pub index: GitStatus,
    pub workdir: GitStatus,
}

impl Default for GitFileStatus {
    fn default() -> Self {
        Self {
            index: GitStatus::Default,
            workdir: GitStatus::Default,
        }
    }
}

impl GitFileStatus {
    pub fn new(status: git2::Status) -> Self {
        Self {
            index: match status {
                s if s.contains(git2::Status::INDEX_NEW) => GitStatus::NewInIndex,
                s if s.contains(git2::Status::INDEX_DELETED) => GitStatus::Deleted,
                s if s.contains(git2::Status::INDEX_MODIFIED) => GitStatus::Modified,
                s if s.contains(git2::Status::INDEX_RENAMED) => GitStatus::Renamed,
                s if s.contains(git2::Status::INDEX_TYPECHANGE) => GitStatus::Typechange,
                _ => GitStatus::Unmodified,
            },

            workdir: match status {
                s if s.contains(git2::Status::WT_NEW) => GitStatus::NewInWorkdir,
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

    pub fn render(&self,
                  colors: &Colors,
                  icons: &Icons) -> ColoredString {
        let strings = &[
            colors.colorize(icons.get_status(&self.index), &Elem::GitStatus { status: self.index }),
            ColoredString::from(" "),
            colors.colorize(icons.get_status(&self.workdir), &Elem::GitStatus { status: self.workdir })
        ];
        let res = ANSIStrings(strings).to_string();
        ColoredString::from(res)
    }
}
