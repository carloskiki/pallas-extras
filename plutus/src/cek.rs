use crate::{
    ConstantIndex, TermIndex, ValueIndex,
    builtin::Builtin,
    program::{Instruction, Program},
};

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Constant(ConstantIndex),
    Delay(TermIndex),
    Lambda(TermIndex),
    Construct {
        determinant: u16,
        values: Vec<Value>,
    },
    Builtin {
        builtin: Builtin,
        polymorphism: u8,
        args: Vec<Value>,
    },
}

impl Value {
    pub fn take(&mut self) -> Value {
        std::mem::replace(self, Value::Constant(ConstantIndex(0)))
    }
}

pub enum Frame {
    Force,
    ApplyLeftTerm,
    ApplyLeftValue(ValueIndex),
    ApplyRightValue(ValueIndex),
    Construct { remaining: u16, value: ValueIndex },
    Case { count: u16 },
}

pub fn run(mut program: Program<u32>) -> Option<Value> {
    let mut stack = Vec::new();
    let mut environment: Vec<Value> = Vec::new();
    let mut values: Vec<Value> = Vec::new();
    let mut index = 0;

    loop {
        let mut ret = match program.program[index] {
            Instruction::Variable(var) => environment.get(var as usize)?.clone(),
            Instruction::Delay => Value::Delay(TermIndex(index as u32 + 1)),
            Instruction::Lambda(_) => Value::Lambda(TermIndex(index as u32 + 1)),
            Instruction::Application => {
                stack.push(Frame::ApplyLeftTerm);
                index += 1;
                continue;
            }
            Instruction::Constant(constant_index) => Value::Constant(ConstantIndex(constant_index)),
            Instruction::Force => {
                stack.push(Frame::Force);
                index += 1;
                continue;
            }
            Instruction::Error => {
                return None;
            }
            Instruction::Builtin(builtin) => Value::Builtin {
                builtin,
                polymorphism: builtin.quantifiers(),
                args: Vec::new(),
            },
            Instruction::Construct {
                determinant,
                length,
            } => {
                let construct = Value::Construct {
                    determinant,
                    values: Vec::new(),
                };

                if length != 0 {
                    let construct_index = ValueIndex(values.len() as u32);
                    values.push(construct);
                    stack.push(Frame::Construct {
                        remaining: length - 1,
                        value: construct_index,
                    });
                    index += 1;
                    continue;
                }

                construct
            }
            Instruction::Case { count } => {
                stack.push(Frame::Case { count });
                index += 1;
                continue;
            }
        };

        loop {
            match (stack.pop(), ret) {
                (Some(Frame::Force), Value::Delay(term)) => {
                    index = term.0 as usize;
                }
                (
                    Some(Frame::Force),
                    Value::Builtin {
                        builtin,
                        polymorphism,
                        args,
                    },
                ) if polymorphism > 0 => {
                    ret = Value::Builtin {
                        builtin,
                        polymorphism: polymorphism - 1,
                        args,
                    };
                    continue;
                }

                (Some(Frame::ApplyLeftTerm), value) => {
                    let value_index = ValueIndex(values.len() as u32);
                    values.push(value);
                    stack.push(Frame::ApplyRightValue(value_index));
                    index += 1;
                }
                (Some(Frame::ApplyRightValue(value_index)), value) => {
                    match &mut values[value_index.0 as usize] {
                        Value::Lambda(term_index) => {
                            environment.push(value);
                            index = term_index.0 as usize;
                        }
                        Value::Builtin {
                            builtin,
                            args,
                            polymorphism: 0,
                        } => {
                            args.push(value);
                            if args.len() == builtin.arity() as usize {
                                // All arguments collected, apply the builtin.
                                ret = match builtin.apply(args, &mut program.constants) {
                                    Some(result) => result,
                                    None => return None,
                                };
                                continue;
                            } else {
                                // Still need more arguments.
                                ret = values[value_index.0 as usize].take();
                                continue;
                            }
                        }
                        _ => return None,
                    }
                }
                (
                    Some(Frame::ApplyLeftValue(value)),
                    Value::Builtin {
                        builtin,
                        mut args,
                        polymorphism: 0,
                    },
                ) => {
                    args.push(values[value.0 as usize].take());
                    if args.len() == builtin.arity() as usize {
                        ret = match builtin.apply(&mut args, &mut program.constants) {
                            Some(result) => result,
                            None => return None,
                        };
                        continue;
                    } else {
                        ret = Value::Builtin {
                            builtin,
                            polymorphism: 0,
                            args,
                        };
                        continue;
                    }
                }
                (Some(Frame::ApplyLeftValue(value_index)), Value::Lambda(term_index)) => {
                    // This path is unlikely (only happens with "case").
                    let value = values[value_index.0 as usize].clone();
                    environment.push(value);
                    index = term_index.0 as usize;
                }

                (
                    Some(Frame::Construct {
                        mut remaining,
                        value: value_index,
                    }),
                    value,
                ) => {
                    let Value::Construct { values, .. } = &mut values[value_index.0 as usize]
                    else {
                        unreachable!("Expected construct value");
                    };
                    values.push(value);

                    if remaining == 0 {
                        ret = values[value_index.0 as usize].take();
                        continue;
                    }
                    remaining -= 1;
                    stack.push(Frame::Construct {
                        remaining,
                        value: value_index,
                    });
                    index += 1;
                }
                (
                    Some(Frame::Case { count }),
                    Value::Construct {
                        determinant,
                        values: construct_values,
                    },
                ) if determinant < count => {
                    index = skip_terms(&program.program, index, determinant)?;
                    let first_index = construct_values.len() as u32;
                    values.extend(construct_values);
                    stack.extend(
                        (first_index..values.len() as u32)
                            .map(|i| Frame::ApplyLeftValue(ValueIndex(i))),
                    )
                }
                (None, value) => {
                    return Some(value);
                }
                _ => return None,
            };
            break;
        }
    }
}

fn skip_terms(terms: &[Instruction<u32>], mut index: usize, count: u16) -> Option<usize> {
    let mut remaining = count;
    while remaining > 0 {
        match terms.get(index)? {
            Instruction::Delay | Instruction::Lambda(_) | Instruction::Force => {}
            Instruction::Variable(_)
            | Instruction::Constant(_)
            | Instruction::Error
            | Instruction::Builtin(_) => {
                remaining -= 1;
            }
            Instruction::Application => {
                remaining += 1;
            }
            Instruction::Construct { length: 0, .. } => {
                remaining -= 1;
            }
            Instruction::Construct { length, .. } => {
                remaining += *length - 1;
            }
            Instruction::Case { count } => {
                remaining += *count;
            }
        }
        index += 1;
    }
    Some(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_terms() {
        let terms = [
            Instruction::Constant(0),
            Instruction::Delay,
            Instruction::Constant(1),
            Instruction::Application,
            Instruction::Lambda(0),
            Instruction::Constant(2),
        ]
        .as_slice();
        assert_eq!(skip_terms(terms, 0, 1), Some(1));
        assert_eq!(skip_terms(terms, 0, 2), Some(3));
        assert_eq!(skip_terms(terms, 0, 3), None);
        assert_eq!(skip_terms(terms, 0, 4), None);
        assert_eq!(skip_terms(terms, 1, 1), Some(3));
    }
}
