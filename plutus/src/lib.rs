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
//!   all items, both public and private.
//! - __Simple__: The internal implementation must be as simple as possible.
//! - __Performant__: The interpreter is architectured with performance in mind, by using
//!   principles from bytecode interpreters. However, performance is not a primary goal, and
//!   should not come at the cost of complexity.
//!
//! # Usage
//!
//! This crate's public API is very minimal. One may start by reading the documentation for [`Program`].
//!
//! Private items (identified by a 🔒) are also documented. They briefly explain the underlying
//! implementation.
//!
//! # Example
//!
//! ```rust
//! use plutus::{Program, Context, Budget};
//!
//! const PROGRAM: &str = "(program 1.0.0 [ [ (builtin addInteger) (con integer 2)] (con integer 2) ])";
//! const FOUR: &str = "(program 1.0.0 (con integer 4))";
//!
//! let arena = plutus::Arena::default();
//! let program: Program<String> = Program::from_str(PROGRAM, &arena).unwrap();
//! let program = program.into_de_bruijn().unwrap();
//!
//! let mut context = plutus::Context {
//!     model: &[0; 297], // Free execution
//!     budget: plutus::Budget { memory: u64::MAX, execution: u64::MAX }, // Maximum budget
//! };
//! let evaluated = program.evaluate(&mut context).unwrap();
//!
//! let four: Program<String> = Program::from_str(FOUR, &arena).unwrap();
//! let four = four.into_de_bruijn().unwrap();
//! assert_eq!(evaluated, four);
//! ```

use std::str::FromStr;

use crate::{builtin::Builtin, constant::Constant};

mod builtin;
mod constant;
pub use constant::Arena;
mod cost;
pub use cost::Context;
/// Script execution budget.
pub use ledger::alonzo::script::execution::Units as Budget;
mod flat;
mod lex;
mod machine;

pub(crate) use ledger::alonzo::script::{Data, data::Construct};

/// Reversed [De Bruijn index](https://en.wikipedia.org/wiki/De_Bruijn_index).
///
/// De Bruijn indices represent variables by the number of lambdas between their binding site and
/// their use site.
///
/// One should not to use this type directly.
///
/// # Details
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
/// There are two ways to obtain a `Program<T>`:
///  - Parsing it from its textual representation using the [`FromStr`] implementation.
///  - Decoding it from its `flat` binary representation using [`Program::from_flat`].
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
/// let arena = plutus::Arena::default();
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
pub struct Program<'a, T> {
    /// The version of the program.
    pub version: Version,
    /// Arena for all allocations during the program.
    ///
    /// This is a list of all constants used in the program. This is separate from the instructions
    /// so that the [`Instruction`] type is more compact, by referring to constants by their index
    /// instead.
    arena: &'a constant::Arena,
    /// Constant pool of the program.
    constants: Vec<Constant<'a>>,
    /// The instructions of the program.
    program: Vec<Instruction<T>>,
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

impl<'a, T: FromStr> Program<'a, T> {
    pub fn from_str(s: &str, arena: &'a constant::Arena) -> Result<Self, ParseError<T::Err>> {
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

        let mut program = Vec::new();
        let mut constants = Vec::new();
        let mut stack: Vec<(&str, Option<u32>)> = Vec::new();
        stack.push((term, None));

        while let Some(&(s, write_back)) = stack.last() {
            if s.is_empty() {
                if let Some(index) = write_back {
                    let next = program.len() as u32;
                    let Some(Instruction::Application(i)) = program.get_mut(index as usize) else {
                        unreachable!("index points to an application instruction");
                    };
                    *i = TermIndex(next);
                }

                stack.pop();
                continue;
            }

            let top = stack.last_mut().expect("stack is not empty");
            match s.as_bytes()[0] {
                b'(' => {
                    let (group, rest) = lex::stripped_group::<b'(', b')'>(&s[1..])
                        .ok_or(ParseError::UnmatchedDelimiter)?;
                    top.0 = rest;
                    let (keyword, rest) = lex::word(group);
                    match keyword {
                        "delay" => {
                            program.push(Instruction::Delay);
                            stack.push((rest, None));
                        }
                        "lam" => {
                            let (var, rest) = lex::word(rest);

                            program.push(Instruction::Lambda(
                                var.parse().map_err(ParseError::Variable)?,
                            ));
                            stack.push((rest.trim_start(), None));
                        }
                        "con" => {
                            let constant =
                                Constant::from_str(rest, arena).map_err(ParseError::Constant)?;
                            let index = ConstantIndex(constants.len() as u32);
                            constants.push(constant);
                            program.push(Instruction::Constant(index));
                        }
                        "force" => {
                            program.push(Instruction::Force);
                            stack.push((rest, None));
                        }
                        "error" => {
                            program.push(Instruction::Error);
                            stack.push((rest, None));
                        }
                        "builtin" => {
                            let builtin =
                                Builtin::from_str(rest).map_err(|_| ParseError::UnknownBuiltin)?;
                            program.push(Instruction::Builtin(builtin));
                        }
                        "constr" if version.minor > 0 => {
                            let (index_str, mut rest) = lex::word(rest);
                            let discriminant: u64 = index_str
                                .parse()
                                .map_err(|_| ParseError::ConstructDiscriminant)?;
                            let (discriminant, large_discriminant) =
                                if discriminant > u32::MAX as u64 {
                                    let index = constants.len() as u32;
                                    constants.push(Constant::Integer(
                                        arena.integer(rug::Integer::from(discriminant)),
                                    ));
                                    (index, true)
                                } else {
                                    (discriminant as u32, false)
                                };

                            let mut count = 0u16;
                            while !rest.is_empty() {
                                let (prefix, arg) =
                                    lex::right_term(rest).ok_or(ParseError::UnmatchedDelimiter)?;
                                rest = prefix;
                                stack.push((arg, None));
                                count += 1;
                            }
                            program.push(Instruction::Construct {
                                discriminant,
                                length: count,
                                large_discriminant,
                            });
                        }
                        "case" if version.minor > 0 => {
                            let mut count = 0u32;
                            let mut rest = rest;
                            while !rest.is_empty() {
                                let (prefix, arg) =
                                    lex::right_term(rest).ok_or(ParseError::UnmatchedDelimiter)?;
                                rest = prefix;
                                stack.push((arg, None));
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
                    top.0 = rest;
                    let mut rest = application;

                    let args_start = stack.len() + 1;
                    while !rest.is_empty() {
                        let (prefix, arg) =
                            lex::right_term(rest).ok_or(ParseError::UnmatchedDelimiter)?;
                        rest = prefix;
                        stack.push((arg, None));
                    }

                    if args_start >= stack.len() {
                        return Err(ParseError::Application);
                    }

                    for frame in stack[args_start..].iter_mut() {
                        frame.1 = Some(program.len() as u32);
                        program.push(Instruction::Application(TermIndex(0)));
                    }
                }
                _ => {
                    let (var, rest) = lex::word(s);
                    top.0 = rest;
                    program.push(Instruction::Variable(
                        var.parse().map_err(ParseError::Variable)?,
                    ));
                }
            };
        }
        Ok(Program {
            version,
            arena,
            constants,
            program,
        })
    }
}

impl<'a, T: PartialEq> Program<'a, T> {
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
    pub fn into_de_bruijn(self) -> Option<Program<'a, DeBruijn>> {
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
                    Instruction::Application(i) => {
                        increment_stack(&mut stack, 1);
                        Instruction::Application(i)
                    }

                    Instruction::Case { count: len } => {
                        increment_stack(&mut stack, len as u32);
                        Instruction::Case { count: len }
                    }
                    Instruction::Construct {
                        discriminant,
                        length: len,
                        large_discriminant,
                    } => {
                        if len > 0 {
                            increment_stack(&mut stack, len as u32 - 1);
                            Instruction::Construct {
                                discriminant,
                                length: len,
                                large_discriminant,
                            }
                        } else {
                            decrement_stack(&mut stack, &mut variables);
                            Instruction::Construct {
                                discriminant,
                                length: len,
                                large_discriminant,
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
                arena: self.arena,
                constants: self.constants,
                program,
            })
    }
}

impl<T, U> PartialEq<Program<'_, T>> for Program<'_, U>
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
                (Instruction::Application(_), Instruction::Application(_)) => true,
                (Instruction::Force, Instruction::Force) => true,
                (Instruction::Error, Instruction::Error) => true,
                (
                    Instruction::Construct {
                        discriminant: a_det,
                        length: a_len,
                        large_discriminant: a_large,
                    },
                    Instruction::Construct {
                        discriminant: b_det,
                        length: b_len,
                        large_discriminant: b_large,
                    },
                ) => {
                    a_len == b_len && {
                        a_large == b_large
                            && if *a_large {
                                self.constants[*a_det as usize] == other.constants[*b_det as usize]
                            } else {
                                a_det == b_det
                            }
                    }
                }
                (Instruction::Case { count: a_count }, Instruction::Case { count: b_count }) => {
                    a_count == b_count
                }
                _ => false,
            })
    }
}

impl<'a> Program<'a, DeBruijn> {
    /// Evaluate a `Program<DeBruijn>`, producing a `Program<DeBruijn>`, or `None` if evaluation
    /// failed.
    pub fn evaluate(self, context: &mut Context<'_>) -> Option<Self> {
        machine::run(self, context)
    }

    /// Decode a `Program<DeBruijn>` from its flat binary representation.
    pub fn from_flat(bytes: &[u8], arena: &'a constant::Arena) -> Option<Self> {
        let mut reader = flat::Reader::new(bytes);
        flat::decode_program(&mut reader, arena)
    }

    /// Encode a `Program<DeBruijn>` into its flat binary representation.
    ///
    /// Encoding can fail if the program contains constants that cannot yet be encoded in flat,
    /// such as `BLS12-381` related constants.
    pub fn to_flat(&self) -> Option<Vec<u8>> {
        let mut buffer = flat::Buffer::default();
        flat::Encode::encode(self, &mut buffer)?;
        Some(buffer.finish())
    }
}

/// An instruction in a `uplc` program.
///
/// Instead of containing pointers to their sub-terms, instructions are laid out in a linear
/// vector, like bytecode (`size_of::<Instruction<DeBruijn>>() == 8`). Each instruction knows how
/// many sub-terms it has. For example, `[(lam x x) (delay error)]` is a single "term", and is
/// represented with the following instructions:
/// ```ignore
/// use plutus::Instruction;
///
/// let instructions = vec![
///     Instruction::Application, // (Term) `[ ... ... ]` -- Expect two sub-terms
///     Instruction::Lambda(String::from("x")), // (Sub-term 1) `(lam x ...)` -- Expect one sub-term
///     Instruction::Variable(String::from("x")),// (Sub-term 1.1) x -- Expect zero sub-terms
///     Instruction::Delay,               // (Sub-term 2) `(delay ...)` -- Expect one sub-term
///     Instruction::Error,               // (Sub-term 2.1) `error` -- Expect zero sub-terms
/// ];
/// ```
///
/// # Limits
///
/// By using bounded numeric types, we limit the maximum size of programs:
///
/// - A program can have at most `u32::MAX` instructions, and `u32::MAX` constants.
/// - A `case` statement can have at most `u16::MAX` branches.
/// - A `construct` can have at most `u16::MAX` fields.
///
/// Generally, we use `u16` to quantify the number of sub-terms, and `u32` to quantify instructions
/// or constants.
#[derive(Debug, Copy, Clone, PartialEq)]
enum Instruction<T> {
    Variable(T),
    Delay,
    Lambda(T),
    /// The index of the second term in the application.
    Application(TermIndex),
    /// Index into the constants pool.
    Constant(ConstantIndex),
    Force,
    Error,
    Builtin(Builtin),
    Construct {
        discriminant: u32,
        length: u16,
        // TODO: Check if having discriminant always as a constant has minimal performance impact,
        // and is simpler.
        /// - `false`: the discriminant fits in 4 bytes, and is stored directly in the
        ///   `discriminant` field.
        /// - `true`: the discriminant is larger than 4 bytes, and the `discriminant` field
        ///   contains the index into the constants pool where the actual discriminant is stored.
        large_discriminant: bool,
    },
    Case {
        count: u16,
    },
}

/// Index of a term in the program.
///
/// This is used by `Value` to point to terms in the program.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TermIndex(u32);

/// Index of a constant in the constants pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ConstantIndex(u32);
