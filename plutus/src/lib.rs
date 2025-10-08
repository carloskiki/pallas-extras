use std::str::FromStr;

use crate::{builtin::Builtin, constant::Constant};

mod builtin;
mod constant;
mod data;
mod lex;

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

pub struct Program {
    version: Version,
    constants: Vec<Constant>,
    program: Vec<Instruction>,
}

/// Part of a program when parsing.
///
/// That can be a group in parens, an application (brackets), or a variable (any identifier).
enum TermType {
    Group,
    Application,
    Variable,
}

impl FromStr for Program {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((program, "")) = lex::group(s.trim()) else {
            return Err(());
        };

        let content = program.strip_prefix("program").ok_or(())?.trim_start();
        let (version_str, term) = lex::word(content);
        let version = Version::from_str(version_str)?;

        let mut variables = Vec::with_capacity(8);
        let mut constants = Vec::with_capacity(16);
        let mut program = Vec::with_capacity(128);
        let mut stack = Vec::with_capacity(32);
        stack.push((term, false));

        while let Some(&(s, _)) = stack.last() {
            if s.is_empty() {
                if let Some((_, true)) = stack.pop() {
                    variables.pop();
                };
                continue;
            }
            
            let (content, after_term, term_type) = lex::term(s).ok_or(())?;
            if let Some((s, _)) = stack.last_mut() {
                *s = after_term;
            };

            match (content, term_type) {
                (content, TermType::Group) => {
                    let (keyword, rest) = lex::word(content);
                    match keyword {
                        "delay" => {
                            program.push(Instruction::Delay);
                            stack.push((rest, false));
                        }
                        "lam" => {
                            let (var, rest) = s
                                .split_once(|c: char| c.is_whitespace() || c == '(')
                                .ok_or(())?;
                            if var.contains(|c: char| !c.is_ascii_alphanumeric()) {
                                return Err(());
                            }

                            program.push(Instruction::Lambda);
                            stack.push((rest.trim_start(), true));
                            variables.push(var);
                        }
                        "con" => {
                            constants.push(Constant::from_str(s)?);
                            let index = (constants.len() - 1) as u16;
                            program.push(Instruction::Constant(index));
                        }
                        "force" => {
                            program.push(Instruction::Force);
                            stack.push((rest, false));
                        }
                        "error" => {
                            program.push(Instruction::Error);
                            stack.push((rest, false));
                        }
                        "builtin" => {
                            let builtin = Builtin::from_str(s).map_err(|_| ())?;
                            program.push(Instruction::Builtin(builtin));
                        }
                        "constr" => {
                            let (index_str, mut rest) = lex::word(s);
                            let determinant: u16 = index_str.parse().map_err(|_| ())?;
                            let mut count = 0u8;
                            while !rest.is_empty() {
                                let (prefix, arg) = lex::right_term(rest).ok_or(())?;
                                rest = prefix;
                                stack.push((arg, false));
                                count += 1;
                            }
                            program.push(Instruction::Construct {
                                determinant,
                                len: count,
                            });
                        }
                        "case" => {
                            let mut count = 0u16;
                            let mut rest = s;
                            while !rest.is_empty() {
                                let (prefix, arg) = lex::right_term(rest).ok_or(())?;
                                rest = prefix;
                                stack.push((arg, false));
                                count += 1;
                            }
                            program.push(Instruction::Case { len: count - 1 });
                        }
                        _ => {
                            return Err(());
                        }
                    }
                }
                (content, TermType::Application) => {
                    program.push(Instruction::Application);
                    stack.push((content, false));
                }

                (var, TermType::Variable) => {
                    let index = variables.iter().rev().position(|v| *v == var).ok_or(())?;
                    program.push(Instruction::Variable(index as u16));
                }
            };
        }
        Ok(Program {
            version,
            constants,
            program,
        })
    }
}

pub enum Instruction {
    // De Bruijn index
    Variable(u16),
    Delay,
    Lambda,
    Application,
    // Index into the constants pool
    Constant(u16),
    Force,
    Error,
    Builtin(Builtin),
    // Should we support full u64 determinants?
    Construct { determinant: u16, len: u8 },
    Case { len: u16 },
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
