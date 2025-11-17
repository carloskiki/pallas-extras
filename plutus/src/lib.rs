#![deny(clippy::undocumented_unsafe_blocks)]
//! Plutus-core interpreter implementation.
//!
//! This library provides functionality to parse (textual and flat) untyped plutus-core (uplc)
//! programs, and evaluate them. It also supports encoding uplc programs into the flat format.
//!
//! # Goals
//! This implementation aims to achieve the following, listed in order of priority:
//! - __Correct__: The interpreter must accurately follow the uplc specification, and be on
//!   par with the [`IntersectMBO/plutus`](https://github.com/IntersectMBO/plutus)
//!   implementation.
//! - __Well-documented__: Both the library API _and_ the internal implementation must be
//!   well-documented, to facilitate understanding and maintenance. This includes documenting
//!   all items, both public and private, and writing clear code over succinct code.
//! - __Simple__: The internal implementation must be as simple as possible.
//! - __Performant__: The interpreter is architectured with performance in mind, by using
//!   principles from bytecode interpreters. However, performance is not a primary goal, and
//!   should not come at the cost of complexity.
//!
//! # Usage
//!
//! This crate's public API is very minimal. One may start by reading the documentation for [`Program`].
//!
//! # Example
//!
//! ```rust
//! use plutus::Program;
//!
//! const PROGRAM: &str = "(program 1.0.0 [ [ (builtin addInteger) (con integer 2)] (con integer 2) ])";
//! const FOUR: &str = "(program 1.0.0 (con integer 4))";
//!
//! let program: Program<String> = PROGRAM.parse().unwrap();
//! let program = program.into_de_bruijn().unwrap();
//! let evaluated = program.evaluate().unwrap();
//!
//! let four: Program<String> = FOUR.parse().unwrap();
//! let four = four.into_de_bruijn().unwrap();
//! assert_eq!(evaluated, four);
//! ```

use std::str::FromStr;

use crate::{builtin::Builtin, constant::Constant};

mod builtin;
mod constant;
mod data;
mod evaluate;
mod flat;
mod lex;

/// Reversed [De Bruijn index](https://en.wikipedia.org/wiki/De_Bruijn_index).
///
/// De Bruijn indices represent variables by the number of lambdas between their binding site and
/// their use site.
///
/// One should not to use this type directly.
///
/// # Details
/// _One does not need to understand this to use the library!_
///
///  This type represents _reversed_ De Bruijn indices starting from `0`, where `0`
/// represents the outermost variable. The reason behind this choice is that it makes indexing
/// variables in a stack much simpler, since the variable is simply the index into the stack.
///
/// `flat` encoding uses traditional De Bruijn indices starting at `1`, so the conversion is done
/// at the flat encoding/decoding boundary.
///
/// ## Example
///
/// We cover the same example in both traditional and reversed De Bruijn indices. We use `λ` to denote
/// a lambda abstraction, and `[]` to denote application.
///
/// A __normal__ De Bruijn index representation, starting from `0`:
/// ```txt
///  ╭──────────────────╮
///  │   ╭──╮ ╭──╮      │
///  ▼   ▼  │ ▼  │      │
/// [λ. [λ. 0 λ. 0] [λ. 1 0]]
///                  ▲    │
///                  ╰────╯
/// ```
///
/// A __reversed__ De Bruijn index representation, starting from `0` (as used in this library):
/// ```txt
///  ╭──────────────────╮
///  │   ╭──╮ ╭──╮      │
///  ▼   ▼  │ ▼  │      │
/// [λ. [λ. 1 λ. 1] [λ. 0 1]]
///                  ▲    │
///                  ╰────╯
/// ```
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeBruijn(pub u32);

/// Program [version](https://en.wikipedia.org/wiki/Software_versioning).
///
/// Currently, only versions `1.0.0` and `1.1.0` are supported.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    /// Major version component.
    pub major: u64,
    /// Minor version component.
    pub minor: u64,
    /// Patch version component.
    pub patch: u64,
}

/// An untyped plutus core (`uplc`) program.
///
/// This type represents a parsed uplc program, parameterized over the variable representation
/// `T`. This can be anything, although the only known use cases are [`String`] and [`DeBruijn`].
///
/// There are two main ways to obtain a `Program<T>`:
///  - Parsing it from its textual representation using the [`FromStr`] implementation.
///  - Decoding it from its flat binary representation using [`Program::from_flat`].
///
/// # Parsing
///
/// To parse a program from its textual representation, use the [`FromStr`] implementation.
/// Technically, type parameter `T` can be any type that implements [`FromStr`], but using
/// [`String`] is the only known use case.
///
/// ```rust
/// use plutus::Program;
///
/// const PROGRAM: &str = "(program 1.0.0 (lam x x))";
/// let program: plutus::Program<String> = PROGRAM.parse().unwrap();
/// ```
///
/// A program can then be converted into a `Program<DeBruijn>` using [`Program::into_de_bruijn`].
///
/// # Flat encoding and decoding
///
/// To decode a program from its flat binary representation, use [`Program::from_flat`]. This
/// produces a `Program<DeBruijn>`, since the `flat` codec only supports De Bruijn indices.
///
/// A program can also be encoded into its `flat` binary representation using [`Program::to_flat`].
///
/// # Evaluating
///
/// Evaluation is only supported for `Program<DeBruijn>`, by calling [`Program::evaluate`].
/// This produces another `Program<DeBruijn>`, representing the evaluated program.
#[derive(Debug)]
pub struct Program<T> {
    /// The version of the program.
    pub version: Version,
    /// The constants pool of the program.
    ///
    /// This is a list of all constants used in the program. This is separate from the instructions
    /// so that the [`Instruction`] type is more compact, by referring to constants by their index
    /// instead.
    constants: Vec<Constant>,
    /// The instructions of the program.
    program: Vec<Instruction<T>>,
}

impl<T: FromStr> FromStr for Program<T> {
    type Err = ParseError<T::Err>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (program, "") =
            lex::group::<b'(', b')'>(s.trim()).ok_or(ParseError::UnmatchedDelimiter)?
        else {
            return Err(ParseError::TrailingContent);
        };

        let content = program
            .strip_prefix("program")
            .ok_or(ParseError::ProgramKeyword)?
            .trim_start();

        let (version_str, term) = lex::word(content);
        let version = {
            let mut parts = version_str.split('.');
            let major = parts
                .next()
                .and_then(|p| p.parse().ok())
                .ok_or(ParseError::Version)?;
            let minor = parts
                .next()
                .and_then(|p| p.parse().ok())
                .ok_or(ParseError::Version)?;
            let patch = parts
                .next()
                .and_then(|p| p.parse().ok())
                .ok_or(ParseError::Version)?;
            if parts.next().is_some() {
                return Err(ParseError::Version);
            }
            Version {
                major,
                minor,
                patch,
            }
        };
        if (version
            != Version {
                major: 1,
                minor: 0,
                patch: 0,
            }
            && version
                != Version {
                    major: 1,
                    minor: 1,
                    patch: 0,
                })
        {
            return Err(ParseError::Version);
        }

        let mut constants = Vec::new();
        let mut program = Vec::new();
        let mut stack = Vec::new();
        stack.push(term);

        while let Some(&s) = stack.last() {
            if s.is_empty() {
                stack.pop();
                continue;
            }

            let top = stack.last_mut().expect("stack is not empty");
            match s.as_bytes()[0] {
                b'(' => {
                    let (group, rest) = lex::stripped_group::<b'(', b')'>(&s[1..])
                        .ok_or(ParseError::UnmatchedDelimiter)?;
                    *top = rest;
                    let (keyword, rest) = lex::word(group);
                    match keyword {
                        "delay" => {
                            program.push(Instruction::Delay);
                            stack.push(rest);
                        }
                        "lam" => {
                            let (var, rest) = lex::word(rest);

                            program.push(Instruction::Lambda(
                                var.parse().map_err(ParseError::Variable)?,
                            ));
                            stack.push(rest.trim_start());
                        }
                        "con" => {
                            constants.push(Constant::from_str(rest)?);
                            let index = (constants.len() - 1) as u32;
                            program.push(Instruction::Constant(ConstantIndex(index)));
                        }
                        "force" => {
                            program.push(Instruction::Force);
                            stack.push(rest);
                        }
                        "error" => {
                            program.push(Instruction::Error);
                            stack.push(rest);
                        }
                        "builtin" => {
                            let builtin =
                                Builtin::from_str(rest).map_err(|_| ParseError::UnknownBuiltin)?;
                            program.push(Instruction::Builtin(builtin));
                        }
                        "constr" if version.minor > 0 => {
                            let (index_str, mut rest) = lex::word(rest);
                            let determinant: u32 = index_str
                                .parse()
                                .map_err(|_| ParseError::ConstructDiscriminant)?;
                            let mut count = 0u16;
                            while !rest.is_empty() {
                                let (prefix, arg) =
                                    lex::right_term(rest).ok_or(ParseError::UnmatchedDelimiter)?;
                                rest = prefix;
                                stack.push(arg);
                                count += 1;
                            }
                            program.push(Instruction::Construct {
                                determinant,
                                length: count,
                            });
                        }
                        "case" if version.minor > 0 => {
                            let mut count = 0u32;
                            let mut rest = rest;
                            while !rest.is_empty() {
                                let (prefix, arg) =
                                    lex::right_term(rest).ok_or(ParseError::UnmatchedDelimiter)?;
                                rest = prefix;
                                stack.push(arg);
                                count += 1;
                            }
                            program.push(Instruction::Case {
                                count: count as u16 - 1,
                            });
                        }
                        _ => {
                            return Err(ParseError::UnknownKeyword);
                        }
                    }
                }
                b'[' => {
                    let (application, rest) = lex::stripped_group::<b'[', b']'>(&s[1..])
                        .ok_or(ParseError::UnmatchedDelimiter)?;
                    *top = rest;
                    let mut count = 0usize;
                    let mut rest = application;
                    while !rest.is_empty() {
                        let (prefix, arg) =
                            lex::right_term(rest).ok_or(ParseError::UnmatchedDelimiter)?;
                        rest = prefix;
                        stack.push(arg);
                        count += 1;
                    }

                    count = count.saturating_sub(1);
                    if count == 0 {
                        return Err(ParseError::Application);
                    }

                    for _ in 0..count {
                        program.push(Instruction::Application);
                    }
                }
                _ => {
                    let (var, rest) = lex::word(s);
                    *top = rest;
                    program.push(Instruction::Variable(
                        var.parse().map_err(ParseError::Variable)?,
                    ));
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

/// Errors that can occur when parsing a `Program<T>`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum ParseError<E> {
    /// There is trailing content after the program.
    #[error("trailing content after program")]
    TrailingContent,
    /// The program does not start with the `program` keyword.
    #[error("missing 'program' keyword")]
    ProgramKeyword,
    /// Version is invalid. Only 1.0.0 and 1.1.0 are supported.
    #[error("invalid program version")]
    Version,
    /// `(` without a matching `)` or `[` without a matching `]`.
    #[error("unmatched delimiter")]
    UnmatchedDelimiter,
    /// Variable parsing error.
    #[error("variable parsing error")]
    Variable(#[source] E),
    /// Unknown keyword.
    #[error("unknown keyword")]
    UnknownKeyword,
    /// Unknown builtin function.
    #[error("unknown builtin function")]
    UnknownBuiltin,
    /// Constant parsing error.
    #[error("constant parsing error")]
    Constant(#[from] constant::ParseError),
    /// A constructor with length zero.
    #[error("constructor with length zero")]
    ConstructDiscriminant,
    /// An application with less than two arguments.
    #[error("application with less than two arguments")]
    Application,
}

impl<T: PartialEq> Program<T> {
    /// Convert any `Program<T: PartialEq>` into a `Program<DeBruijn>`, using reversed De Bruijn
    /// indices.
    ///
    /// # Example
    /// ```rust
    /// use plutus::Program;
    ///
    /// const PROGRAM_A: &str = "(program 1.0.0 (lam x (lam y [x y])))";
    /// const PROGRAM_B: &str = "(program 1.0.0 (lam hello (lam world [hello world])))";
    ///
    /// let program_a: Program<String> = PROGRAM_A.parse().unwrap();
    /// let program_b: Program<String> = PROGRAM_B.parse().unwrap();
    ///
    /// assert_ne!(program_a, program_b);
    ///
    /// let de_bruijn_a = program_a.into_de_bruijn().unwrap();
    /// let de_bruijn_b = program_b.into_de_bruijn().unwrap();
    ///
    /// assert_eq!(de_bruijn_a, de_bruijn_b);
    /// ```
    pub fn into_de_bruijn(self) -> Option<Program<DeBruijn>> {
        fn increment_stack(stack: &mut [u32], count: u32) {
            *stack.last_mut().expect("stack is not empty") += count;
        }

        fn decrement_stack<T>(stack: &mut Vec<u32>, variables: &mut Vec<T>) {
            *stack.last_mut().expect("stack is not empty") -= 1;
            while let Some(0) = stack.last() {
                stack.pop();
                variables.pop();
            }
        }

        let mut variables = Vec::with_capacity(16);
        let mut stack = Vec::with_capacity(32);
        stack.push(1);

        self.program
            .into_iter()
            .map(|instr| {
                Some(match instr {
                    Instruction::Variable(v) => {
                        let position = variables.iter().rposition(|x| *x == v)?;
                        decrement_stack(&mut stack, &mut variables);
                        Instruction::Variable(DeBruijn(position as u32))
                    }
                    Instruction::Error => {
                        decrement_stack(&mut stack, &mut variables);
                        Instruction::Error
                    }
                    Instruction::Constant(c) => {
                        decrement_stack(&mut stack, &mut variables);
                        Instruction::Constant(c)
                    }
                    Instruction::Builtin(b) => {
                        decrement_stack(&mut stack, &mut variables);
                        Instruction::Builtin(b)
                    }
                    Instruction::Lambda(v) => {
                        let index = variables.len();
                        variables.push(v);
                        *stack.last_mut().expect("stack is not empty") -= 1;
                        stack.push(1);

                        Instruction::Lambda(DeBruijn(index as u32))
                    }
                    Instruction::Application => {
                        increment_stack(&mut stack, 1);
                        Instruction::Application
                    }

                    Instruction::Case { count: len } => {
                        increment_stack(&mut stack, len as u32);
                        Instruction::Case { count: len }
                    }
                    Instruction::Construct {
                        determinant,
                        length: len,
                    } => {
                        if len > 0 {
                            increment_stack(&mut stack, len as u32 - 1);
                            Instruction::Construct {
                                determinant,
                                length: len,
                            }
                        } else {
                            decrement_stack(&mut stack, &mut variables);
                            Instruction::Construct {
                                determinant,
                                length: len,
                            }
                        }
                    }

                    Instruction::Delay => Instruction::Delay,
                    Instruction::Force => Instruction::Force,
                })
            })
            .collect::<Option<Vec<_>>>()
            .map(|program| Program {
                version: self.version,
                constants: self.constants,
                program,
            })
    }
}

impl<T, U> PartialEq<Program<T>> for Program<U>
where
    U: PartialEq<T>,
{
    fn eq(&self, other: &Program<T>) -> bool {
        self.program
            .iter()
            .zip(other.program.iter())
            .all(|(a, b)| match (a, b) {
                (Instruction::Variable(a), Instruction::Variable(b)) => a == b,
                (Instruction::Lambda(a), Instruction::Lambda(b)) => a == b,
                (Instruction::Builtin(a), Instruction::Builtin(b)) => a == b,
                (Instruction::Constant(a), Instruction::Constant(b)) => {
                    self.constants[a.0 as usize] == other.constants[b.0 as usize]
                }
                (Instruction::Delay, Instruction::Delay) => true,
                (Instruction::Application, Instruction::Application) => true,
                (Instruction::Force, Instruction::Force) => true,
                (Instruction::Error, Instruction::Error) => true,
                (
                    Instruction::Construct {
                        determinant: a_det,
                        length: a_len,
                    },
                    Instruction::Construct {
                        determinant: b_det,
                        length: b_len,
                    },
                ) => a_det == b_det && a_len == b_len,
                (Instruction::Case { count: a_count }, Instruction::Case { count: b_count }) => {
                    a_count == b_count
                }
                _ => false,
            })
    }
}

impl Program<DeBruijn> {
    pub fn evaluate(self) -> Option<Self> {
        evaluate::run(self)
    }

    pub fn from_flat(bytes: &[u8]) -> Option<Self> {
        let mut reader = flat::Reader::new(bytes);
        flat::Decode::decode(&mut reader)
    }

    pub fn to_flat(&self) -> Option<Vec<u8>> {
        let mut buffer = flat::Buffer::default();
        flat::Encode::encode(self, &mut buffer)?;
        Some(buffer.finish())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Instruction<T> {
    Variable(T),
    Delay,
    Lambda(T),
    Application,
    // Index into the constants pool
    Constant(ConstantIndex),
    Force,
    Error,
    Builtin(Builtin),
    // Should we support full u64 determinants?
    Construct { determinant: u32, length: u16 },
    Case { count: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TermIndex(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ConstantIndex(u32);
