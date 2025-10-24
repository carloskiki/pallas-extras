use std::{num::NonZeroU16, str::FromStr};

use crate::{Version, builtin::Builtin, constant::Constant, lex};

mod evaluate;

#[derive(Debug)]
pub struct Program<T> {
    version: Version,
    constants: Vec<Constant>,
    pub(crate) program: Vec<Instruction<T>>,
}

/// Type of term parsed
///
/// That can be a group in parens, an application (brackets), or a variable (any identifier).
pub(crate) enum TermType {
    Group,
    Application,
    Variable,
}

impl<T: FromStr> FromStr for Program<T> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((program, "")) = lex::group::<b'(', b')'>(s.trim()) else {
            return Err(());
        };

        let content = program.strip_prefix("program").ok_or(())?.trim_start();
        let (version_str, term) = lex::word(content);
        let version = Version::from_str(version_str)?;
        if version.major != 1 {
            return Err(());
        }

        let mut constants = Vec::with_capacity(16);
        let mut program = Vec::with_capacity(128);
        let mut stack = Vec::with_capacity(32);
        stack.push(term);

        while let Some(&s) = stack.last() {
            if s.is_empty() {
                stack.pop();
                continue;
            }

            let (content, after_term, term_type) = lex::term(s).ok_or(())?;
            if let Some(s) = stack.last_mut() {
                *s = after_term;
            };

            match (content, term_type) {
                (content, TermType::Group) => {
                    let (keyword, rest) = lex::word(content);
                    match keyword {
                        "delay" => {
                            program.push(Instruction::Delay);
                            stack.push(rest);
                        }
                        "lam" => {
                            let (var, rest) = lex::word(rest);

                            program.push(Instruction::Lambda(var.parse().map_err(|_| ())?));
                            stack.push(rest.trim_start());
                        }
                        "con" => {
                            constants.push(Constant::from_str(rest)?);
                            let index = (constants.len() - 1) as u32;
                            program.push(Instruction::Constant(index));
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
                            let builtin = Builtin::from_str(rest).map_err(|_| ())?;
                            program.push(Instruction::Builtin(builtin));
                        }
                        "constr" if version.minor > 0 => {
                            let (index_str, mut rest) = lex::word(rest);
                            let determinant: u16 = index_str.parse().map_err(|_| ())?;
                            let mut count = 0u32;
                            while !rest.is_empty() {
                                let (prefix, arg) = lex::right_term(rest).ok_or(())?;
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
                                let (prefix, arg) = lex::right_term(rest).ok_or(())?;
                                rest = prefix;
                                stack.push(arg);
                                count += 1;
                            }
                            program.push(Instruction::Case { count: count - 1 });
                        }
                        _ => {
                            return Err(());
                        }
                    }
                }
                (content, TermType::Application) => {
                    let mut count = 0usize;
                    let mut rest = content;
                    while !rest.is_empty() {
                        let (prefix, arg) = lex::right_term(rest).ok_or(())?;
                        rest = prefix;
                        stack.push(arg);
                        count += 1;
                    }

                    count = count.saturating_sub(1);
                    if count == 0 {
                        return Err(());
                    }

                    for _ in 0..count {
                        program.push(Instruction::Application);
                    }
                }

                (var, TermType::Variable) => {
                    program.push(Instruction::Variable(var.parse().map_err(|_| ())?));
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

impl<T: PartialEq> Program<T> {
    pub fn into_de_bruijn(self) -> Option<Program<DeBruijn>> {
        let mut variables = Vec::with_capacity(16);
        let mut stack = Vec::with_capacity(32);
        stack.push((0, 0)); // (term count, variable count)

        self.program
            .into_iter()
            .map(|instr| {
                match instr {
                    // Decrease by 1
                    Instruction::Variable(v) => variables
                        .iter()
                        .rposition(|x| *x == v)
                        .map(|pos| {
                            Instruction::Variable(DeBruijn((variables.len() - 1 - pos) as u32))
                        })
                        .and_then(|var_instr| {
                            decrement_stack(&mut stack, &mut variables).then_some(var_instr)
                        }),
                    Instruction::Error => {
                        decrement_stack(&mut stack, &mut variables).then_some(Instruction::Error)
                    }
                    Instruction::Constant(c) => decrement_stack(&mut stack, &mut variables)
                        .then_some(Instruction::Constant(c)),
                    Instruction::Builtin(b) => decrement_stack(&mut stack, &mut variables)
                        .then_some(Instruction::Builtin(b)),

                    Instruction::Lambda(v) => {
                        variables.push(v);
                        stack.last_mut().map(|(depth, _)| {
                            *depth += 1;
                            Instruction::Lambda(DeBruijn(0u32))
                        })
                    }
                    Instruction::Application => {
                        increment_stack(&mut stack, 1).then_some(Instruction::Application)
                    }

                    Instruction::Case { count: len } => {
                        increment_stack(&mut stack, len).then_some(Instruction::Case { count: len })
                    }
                    Instruction::Construct {
                        determinant,
                        length: len,
                    } => {
                        if len > 0 {
                            increment_stack(&mut stack, len - 1).then_some(Instruction::Construct {
                                determinant,
                                length: len,
                            })
                        } else {
                            decrement_stack(&mut stack, &mut variables).then_some(
                                Instruction::Construct {
                                    determinant,
                                    length: len,
                                },
                            )
                        }
                    }

                    Instruction::Delay => Some(Instruction::Delay),
                    Instruction::Force => Some(Instruction::Force),
                }
            })
            .collect::<Option<Vec<_>>>()
            .map(|program| Program {
                version: self.version,
                constants: self.constants,
                program,
            })
    }
}

fn increment_stack(stack: &mut [(u32, u32)], count: u32) -> bool {
    let Some((term_count, _)) = stack.last_mut() else {
        return false;
    };
    *term_count += count;
    true
}

fn decrement_stack<T>(stack: &mut Vec<(u32, u32)>, variables: &mut Vec<T>) -> bool {
    if let Some((depth, var_count)) = stack.last_mut() {
        if *depth > 0 {
            *depth -= 1;
        } else {
            variables.truncate(variables.len() - *var_count as usize);
            stack.pop();
        }
        true
    } else {
        false
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Instruction<T> {
    Variable(T),
    Delay,
    Lambda(T),
    Application,
    // Index into the constants pool
    Constant(u32),
    Force,
    Error,
    Builtin(Builtin),
    // Should we support full u64 determinants?
    Construct { determinant: u16, length: u32 },
    Case { count: u32 },
}

impl<T> Instruction<T> {
    pub fn is_value(&self) -> bool {
        matches!(
            self,
            Instruction::Constant(_)
                | Instruction::Delay
                | Instruction::Lambda(_)
                | Instruction::Construct { .. }
                | Instruction::Builtin(_)
        )
    }
}

/// A De Bruijn index
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeBruijn(u32);

#[cfg(test)]
mod tests {
    use crate::program::DeBruijn;

    #[test]
    fn de_bruijn() {
        let program: super::Program<String> =
            "(program 1.0.0 (lam x (lam y [x y])))".parse().unwrap();
        let de_bruijn = program.into_de_bruijn().unwrap();
        assert_eq!(
            de_bruijn.program,
            vec![
                super::Instruction::Lambda(DeBruijn(0)),
                super::Instruction::Lambda(DeBruijn(0)),
                super::Instruction::Application,
                super::Instruction::Variable(DeBruijn(1)),
                super::Instruction::Variable(DeBruijn(0)),
            ]
        );
    }
}
