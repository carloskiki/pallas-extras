use std::str::FromStr;

use crate::{ConstantIndex, DeBruijn, Version, builtin::Builtin, constant::Constant, lex};

pub(crate) mod evaluate;

#[derive(Debug)]
pub struct Program<T> {
    version: Version,
    pub(crate) constants: Vec<Constant>,
    pub(crate) program: Vec<Instruction<T>>,
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
                            let builtin = Builtin::from_str(rest).map_err(|_| ())?;
                            program.push(Instruction::Builtin(builtin));
                        }
                        "constr" if version.minor > 0 => {
                            let (index_str, mut rest) = lex::word(rest);
                            let determinant: u32 = index_str.parse().map_err(|_| ())?;
                            let mut count = 0u16;
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
        stack.push(1); // (term count, variable count)

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
                        increment_stack(&mut stack, len);
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
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Instruction<T> {
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
    Case { count: u32 },
}

/// Type of term parsed
///
/// That can be a group in parens, an application (brackets), or a variable (any identifier).
pub(crate) enum TermType {
    Group,
    Application,
    Variable,
}

#[cfg(test)]
mod tests {
    use crate::DeBruijn;

    #[test]
    fn de_bruijn() {
        let program: super::Program<String> =
            "(program 1.0.0 (lam x (lam y [x y])))".parse().unwrap();
        let de_bruijn = program.into_de_bruijn().unwrap();
        assert_eq!(
            de_bruijn.program,
            vec![
                super::Instruction::Lambda(DeBruijn(0)),
                super::Instruction::Lambda(DeBruijn(1)),
                super::Instruction::Application,
                super::Instruction::Variable(DeBruijn(0)),
                super::Instruction::Variable(DeBruijn(1)),
            ]
        );
    }
}
