#![deny(clippy::undocumented_unsafe_blocks)]

use std::str::FromStr;

mod builtin;
mod constant;
mod data;
mod lex;
pub mod flat;
pub mod program;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TermIndex(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ConstantIndex(u32);

/// A De Bruijn index
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeBruijn(pub u32);

#[derive(Debug)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl FromStr for Version {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');
        let major = parts.next().and_then(|p| p.parse().ok()).ok_or(())?;
        let minor = parts.next().and_then(|p| p.parse().ok()).ok_or(())?;
        let patch = parts.next().and_then(|p| p.parse().ok()).ok_or(())?;
        if parts.next().is_some() {
            return Err(());
        }
        Ok(Version {
            major,
            minor,
            patch,
        })
    }
}
