use ansi_term::{ANSIString, Colour, Style};
use lscolors::{Indicator, LsColors};
use std::collections::HashMap;
use std::path::Path;

#[allow(dead_code)]
#[derive(strum::EnumIter)] // for tests
#[derive(Hash, Debug, Eq, PartialEq, Clone)]
pub enum Elem {
    /// Node type
    File {
        exec: bool,
        uid: bool,
    },
    SymLink,
    BrokenSymLink,
    Dir {
        uid: bool,
    },
    Pipe,
    BlockDevice,
    CharDevice,
    Socket,
    Special,

    /// Permissions
    Read,
    Write,
    Exec,
    ExecSticky,
    NoAccess,

    /// Last Time Modified
    DayOld,
    HourOld,
    Older,

    /// User / Group Name
    User,
    Group,

    /// File Size
    NonFile,
    FileLarge,
    FileMedium,
    FileSmall,

    /// INode
    INode {
        valid: bool,
    },

    Links {
        valid: bool,
    },

    TreeEdge,

    #[cfg(all(
        feature = "git",
        not(any(
            all(target_os = "linux", target_arch = "arm"),
            all(windows, target_arch = "x86", target_env = "gnu")
        ))
    ))]
    GitStatus {
        status: crate::git::GitStatus,
    },
}

impl Elem {
    pub fn has_suid(&self) -> bool {
        matches!(self, Elem::Dir { uid: true } | Elem::File { uid: true, .. })
    }
}

pub type ColoredString<'a> = ANSIString<'a>;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum Theme {
    NoColor,
    Default,
    NoLscolors,
}

pub struct Colors {
    colors: Option<HashMap<Elem, Colour>>,
    lscolors: Option<LsColors>,
}

impl Colors {
    pub fn new(theme: Theme) -> Self {
        let colors = match theme {
            Theme::NoColor => None,
            Theme::Default => Some(Self::get_light_theme_colour_map()),
            Theme::NoLscolors => Some(Self::get_light_theme_colour_map()),
        };
        let lscolors = match theme {
            Theme::NoColor => None,
            Theme::Default => Some(LsColors::from_env().unwrap_or_default()),
            Theme::NoLscolors => None,
        };

        Self { colors, lscolors }
    }

    pub fn colorize<'a>(&self, input: String, elem: &Elem) -> ColoredString<'a> {
        self.style(elem).paint(input)
    }

    pub fn colorize_using_path<'a>(
        &self,
        input: String,
        path: &Path,
        elem: &Elem,
    ) -> ColoredString<'a> {
        let style_from_path = self.style_from_path(path);
        match style_from_path {
            Some(style_from_path) => style_from_path.paint(input),
            None => self.colorize(input, elem),
        }
    }

    fn style_from_path(&self, path: &Path) -> Option<Style> {
        match &self.lscolors {
            Some(lscolors) => lscolors
                .style_for_path(path)
                .map(lscolors::Style::to_ansi_term_style),
            None => None,
        }
    }

    fn style(&self, elem: &Elem) -> Style {
        match &self.lscolors {
            Some(lscolors) => match self.get_indicator_from_elem(elem) {
                Some(style) => {
                    let style = lscolors.style_for_indicator(style);
                    style
                        .map(lscolors::Style::to_ansi_term_style)
                        .unwrap_or_default()
                }
                None => self.style_default(elem),
            },
            None => self.style_default(elem),
        }
    }

    fn style_default(&self, elem: &Elem) -> Style {
        if let Some(ref colors) = self.colors {
            let style_fg = Style::default().fg(colors[elem]);
            if elem.has_suid() {
                style_fg.on(Colour::Fixed(124)) // Red3
            } else {
                style_fg
            }
        } else {
            Style::default()
        }
    }

    fn get_indicator_from_elem(&self, elem: &Elem) -> Option<Indicator> {
        let indicator_string = match elem {
            Elem::File { exec, uid } => match (exec, uid) {
                (_, true) => None,
                (true, false) => Some("ex"),
                (false, false) => Some("fi"),
            },
            Elem::Dir { uid } => {
                if *uid {
                    None
                } else {
                    Some("di")
                }
            }
            Elem::SymLink => Some("ln"),
            Elem::Pipe => Some("pi"),
            Elem::Socket => Some("so"),
            Elem::BlockDevice => Some("bd"),
            Elem::CharDevice => Some("cd"),
            Elem::BrokenSymLink => Some("or"),
            Elem::INode { valid } => match valid {
                true => Some("so"),
                false => Some("no"),
            },
            Elem::Links { valid } => match valid {
                true => Some("so"),
                false => Some("no"),
            },
            _ => None,
        };

        match indicator_string {
            Some(ids) => Indicator::from(ids),
            None => None,
        }
    }

    // You can find the table for each color, code, and display at:
    //
    //https://jonasjacek.github.io/colors/
    fn get_light_theme_colour_map() -> HashMap<Elem, Colour> {
        let mut m = HashMap::new();
        // User / Group
        m.insert(Elem::User, Colour::Fixed(230)); // Cornsilk1
        m.insert(Elem::Group, Colour::Fixed(187)); // LightYellow3

        // Permissions
        m.insert(Elem::Read, Colour::Green);
        m.insert(Elem::Write, Colour::Yellow);
        m.insert(Elem::Exec, Colour::Red);
        m.insert(Elem::ExecSticky, Colour::Purple);
        m.insert(Elem::NoAccess, Colour::Fixed(245)); // Grey

        // File Types
        m.insert(
            Elem::File {
                exec: false,
                uid: false,
            },
            Colour::Fixed(184),
        ); // Yellow3
        m.insert(
            Elem::File {
                exec: false,
                uid: true,
            },
            Colour::Fixed(184),
        ); // Yellow3
        m.insert(
            Elem::File {
                exec: true,
                uid: false,
            },
            Colour::Fixed(40),
        ); // Green3
        m.insert(
            Elem::File {
                exec: true,
                uid: true,
            },
            Colour::Fixed(40),
        ); // Green3
        m.insert(Elem::Dir { uid: true }, Colour::Fixed(33)); // DodgerBlue1
        m.insert(Elem::Dir { uid: false }, Colour::Fixed(33)); // DodgerBlue1
        m.insert(Elem::Pipe, Colour::Fixed(44)); // DarkTurquoise
        m.insert(Elem::SymLink, Colour::Fixed(44)); // DarkTurquoise
        m.insert(Elem::BrokenSymLink, Colour::Fixed(124)); // Red3
        m.insert(Elem::BlockDevice, Colour::Fixed(44)); // DarkTurquoise
        m.insert(Elem::CharDevice, Colour::Fixed(172)); // Orange3
        m.insert(Elem::Socket, Colour::Fixed(44)); // DarkTurquoise
        m.insert(Elem::Special, Colour::Fixed(44)); // DarkTurquoise

        // Last Time Modified
        m.insert(Elem::HourOld, Colour::Fixed(40)); // Green3
        m.insert(Elem::DayOld, Colour::Fixed(42)); // SpringGreen2
        m.insert(Elem::Older, Colour::Fixed(36)); // DarkCyan

        // Last Time Modified
        m.insert(Elem::NonFile, Colour::Fixed(245)); // Grey
        m.insert(Elem::FileSmall, Colour::Fixed(229)); // Wheat1
        m.insert(Elem::FileMedium, Colour::Fixed(216)); // LightSalmon1
        m.insert(Elem::FileLarge, Colour::Fixed(172)); // Orange3

        // INode
        m.insert(Elem::INode { valid: true }, Colour::Fixed(13)); // Pink
        m.insert(Elem::INode { valid: false }, Colour::Fixed(245)); // Grey
        m.insert(Elem::Links { valid: true }, Colour::Fixed(13));
        m.insert(Elem::Links { valid: false }, Colour::Fixed(245));

        // TODO add this after we can use file to configure theme
        // m.insert(Elem::TreeEdge, Colour::Fixed(44)); // DarkTurquoise

        // GitStatus
        #[cfg(all(
            feature = "git",
            not(any(
                all(target_os = "linux", target_arch = "arm"),
                all(windows, target_arch = "x86", target_env = "gnu")
            ))
        ))]
        {
            m.insert(
                Elem::GitStatus {
                    status: crate::git::GitStatus::Default,
                },
                Colour::White,
            );
            m.insert(
                Elem::GitStatus {
                    status: crate::git::GitStatus::Unmodified,
                },
                Colour::White,
            );
            m.insert(
                Elem::GitStatus {
                    status: crate::git::GitStatus::Ignored,
                },
                Colour::Fixed(245),
            ); // Grey
            m.insert(
                Elem::GitStatus {
                    status: crate::git::GitStatus::NewInIndex,
                },
                Colour::Green,
            );
            m.insert(
                Elem::GitStatus {
                    status: crate::git::GitStatus::NewInWorkdir,
                },
                Colour::White,
            );
            m.insert(
                Elem::GitStatus {
                    status: crate::git::GitStatus::Typechange,
                },
                Colour::White,
            );
            m.insert(
                Elem::GitStatus {
                    status: crate::git::GitStatus::Deleted,
                },
                Colour::Red,
            );
            m.insert(
                Elem::GitStatus {
                    status: crate::git::GitStatus::Renamed,
                },
                Colour::Fixed(172),
            ); // Orange3
            m.insert(
                Elem::GitStatus {
                    status: crate::git::GitStatus::Modified,
                },
                Colour::Blue,
            );
            m.insert(
                Elem::GitStatus {
                    status: crate::git::GitStatus::Conflicted,
                },
                Colour::Red,
            );
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_elem_map_completeness() {
        let m = Colors::get_light_theme_colour_map();
        for elem in Elem::iter() {
            assert!(m.contains_key(&elem));
        }
    }

    #[cfg(all(
        feature = "git",
        not(any(
            all(target_os = "linux", target_arch = "arm"),
            all(windows, target_arch = "x86", target_env = "gnu")
        ))
    ))]
    #[test]
    fn test_git_status_map_completeness() {
        let m = Colors::get_light_theme_colour_map();
        for status in crate::git::GitStatus::iter() {
            let elem = Elem::GitStatus { status };
            assert!(m.contains_key(&elem));
        }
    }
}
