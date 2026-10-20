// SPDX-License-Identifier: GPL-3.0-or-later
//! Kindle Series Enabler: shows the Library's "Group series" switch in
//! marketplaces where Amazon doesn't offer it (docs/decisions).

pub mod appreg;
pub mod commands;
pub mod grouping;
pub mod state;

/// Errors carry a message for the user; the binary prints it.
pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;
