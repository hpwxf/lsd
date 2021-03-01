//! This module defines the [GitStatus] flag. To set it up from [ArgMatches], a [Config] and its
//! [Default] value, use the [configure_from](Configurable::configure_from) method.

use super::Configurable;

use crate::config_file::Config;

use clap::ArgMatches;
use serde::Deserialize;

/// The flag showing how git status is computed
#[derive(Clone, Debug, Copy, PartialEq, Eq, Deserialize)]
pub enum GitStatus {
    Flat,
    Recursive,
}

impl Configurable<Self> for GitStatus {
    /// Get a potential `GitStatus` variant from [ArgMatches].
    ///
    /// If any of the "flat" or "recursive" arguments is passed, this returns the
    /// corresponding `GitStatus` variant in a [Some]. If neither of them is passed, this returns
    /// [None].
    fn from_arg_matches(matches: &ArgMatches) -> Option<Self> {
        // println!("{:?}", matches);
        if matches.is_present("git") {
            // println!("DEBUG1");
            Some(Self::Flat)
        } else if matches.is_present("recursive") {
            // println!("DEBUG2");
            Some(Self::Recursive)
        } else {
            // println!("DEBUG3");
            None
        }
    }

    /// Get a potential `GitStatus` variant from a [Config].
    ///
    /// If the `Config::git_status` has value and is one of
    /// "flat" or "recursive", this returns the corresponding `GitStatus` variant in a [Some].
    /// Otherwise, this returns [None].
    fn from_config(config: &Config) -> Option<Self> {
        config.git_status
    }
}

/// The default value for `Display` is [Display::VisibleOnly].
impl Default for GitStatus {
    fn default() -> Self {
        GitStatus::Flat
    }
}

// FIXME TODO tests
