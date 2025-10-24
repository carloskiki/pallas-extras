use std::str::FromStr;

mod builtin;
mod constant;
mod data;
mod lex;
mod cek;
pub mod program;

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

pub enum BuiltinType {
    Integer,
    Bytes,
    String,
    Unit,
    Boolean,
    List,
    Pair = 0b0110,
    // TypeApplication = 0b0111, Probably only for decoding
    Data = 0b1000,
    BLSG1Element,
    BLSG2Element,
    BLSMlResult,
    Array,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TermIndex(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ConstantIndex(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ValueIndex(u32);
