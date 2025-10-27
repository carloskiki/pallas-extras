use crate::program::Program;
use crate::{ConstantIndex, DeBruijn, TermIndex, builtin::Builtin, program::Instruction};

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Constant(ConstantIndex),
    Delay {
        term: TermIndex,
        environment: Vec<Value>,
    },
    Lambda {
        term: TermIndex,
        environment: Vec<Value>,
    },
    Construct {
        determinant: u32,
        values: Vec<Value>,
    },
    Builtin {
        builtin: Builtin,
        polymorphism: u8,
        args: Vec<Value>,
    },
}

impl Value {
    fn discharge(self, mut program: Program<DeBruijn>) -> Program<DeBruijn> {
        enum DischargeValue {
            Constant(ConstantIndex),
            Term {
                term: TermIndex,
                remaining: u32,
                environment: Vec<Value>,
            },
            Construct {
                determinant: u32,
                values: Vec<DischargeValue>,
            },
            Builtin {
                builtin: Builtin,
                args: Vec<DischargeValue>,
            },
        }

        impl From<Value> for DischargeValue {
            fn from(value: Value) -> Self {
                match value {
                    Value::Constant(constant_index) => DischargeValue::Constant(constant_index),
                    Value::Delay { term, environment } | Value::Lambda { term, environment } => {
                        DischargeValue::Term {
                            term,
                            remaining: 1,
                            environment,
                        }
                    }
                    Value::Construct {
                        determinant,
                        values,
                    } => DischargeValue::Construct {
                        determinant,
                        values: values.into_iter().map(DischargeValue::from).collect(),
                    },
                    Value::Builtin { builtin, args, .. } => DischargeValue::Builtin {
                        builtin,
                        args: args.into_iter().map(DischargeValue::from).collect(),
                    },
                }
            }
        }
        
        let mut value_stack = vec![DischargeValue::from(self)];
        let mut instructions = Vec::new();
        while let Some(value) = value_stack.pop() {
            match value {
                DischargeValue::Constant(constant_index) => {
                    instructions.push(Instruction::Constant(constant_index));
                }
                DischargeValue::Term {
                    term,
                    environment,
                    remaining,
                } => {
                    // TODO: this is wrong
                    let mut remaining = remaining;
                    let mut index = term.0 as usize;
                    while remaining > 0 {
                        match program.program[index] {
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
                                remaining += length as u32 - 1;
                            }
                            Instruction::Case { count } => {
                                remaining += count;
                            }
                        }
                        if let Instruction::Variable(var) = program.program[index] {
                            let value = environment[environment.len() - var.0 as usize].clone();
                            value_stack.push(DischargeValue::Term {
                                term: TermIndex(index as u32 + 1),
                                remaining,
                                environment,
                            });
                            value_stack.push(DischargeValue::from(value));
                            break;
                        } else {
                            index += 1;
                            instructions.push(program.program[term.0 as usize]);
                        }
                    }
                }
                DischargeValue::Construct {
                    determinant,
                    values,
                } => {
                    instructions.push(Instruction::Construct {
                        determinant,
                        length: values.len() as u16,
                    });
                    value_stack.extend(values.into_iter().rev());
                }
                DischargeValue::Builtin { builtin, args, .. } => {
                    instructions.extend(std::iter::repeat_n(Instruction::Application, args.len()));
                    instructions.push(Instruction::Builtin(builtin));
                    value_stack.extend(args.into_iter().rev());
                }
            }
        }
        program.program = instructions;
        program
    }
}

pub enum Frame {
    Force,
    ApplyLeftValue(Value),
    ApplyRightValue(Value),
    ApplyLeftTerm {
        environment: Vec<Value>,
    },
    Construct {
        remaining: u16,
        determinant: u32,
        environment_len: u32,
        environment_and_values: Vec<Value>,
    },
    Case {
        count: u32,
        environment: Vec<Value>,
    },
}

pub fn run(mut program: Program<DeBruijn>) -> Option<Program<DeBruijn>> {
    let mut stack = Vec::new();
    let mut environment: Vec<Value> = Vec::new();
    let mut index = 0;

    loop {
        let mut ret = match program.program[index] {
            Instruction::Variable(var) => {
                environment.get(environment.len() - var.0 as usize)?.clone()
            }
            Instruction::Delay => Value::Delay {
                term: TermIndex(index as u32),
                environment,
            },
            Instruction::Lambda(_) => Value::Lambda {
                term: TermIndex(index as u32),
                environment,
            },
            Instruction::Application => {
                stack.push(Frame::ApplyLeftTerm {
                    environment: environment.clone(),
                });
                index += 1;
                continue;
            }
            Instruction::Constant(constant_index) => Value::Constant(constant_index),
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
                if length != 0 {
                    stack.push(Frame::Construct {
                        remaining: length - 1,
                        environment_len: environment.len() as u32,
                        environment_and_values: environment.clone(),
                        determinant,
                    });
                    index += 1;
                    continue;
                }

                Value::Construct {
                    determinant,
                    values: Vec::new(),
                }
            }
            Instruction::Case { count } => {
                stack.push(Frame::Case {
                    count,
                    environment: environment.clone(),
                });
                index += 1;
                continue;
            }
        };

        environment = loop {
            break match (stack.pop(), ret) {
                (Some(Frame::Force), Value::Delay { term, environment }) => {
                    index = term.0 as usize;
                    environment
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

                (Some(Frame::ApplyLeftTerm { environment }), value) => {
                    stack.push(Frame::ApplyRightValue(value));
                    index += 1;
                    environment
                }
                (
                    Some(Frame::ApplyRightValue(Value::Lambda {
                        term,
                        mut environment,
                    })),
                    value,
                )
                | (
                    Some(Frame::ApplyLeftValue(value)),
                    Value::Lambda {
                        term,
                        mut environment,
                    },
                ) => {
                    environment.push(value);
                    index = term.0 as usize;
                    environment
                }
                (
                    Some(Frame::ApplyRightValue(Value::Builtin {
                        builtin,
                        polymorphism: 0,
                        mut args,
                    })),
                    value,
                )
                | (
                    Some(Frame::ApplyLeftValue(value)),
                    Value::Builtin {
                        builtin,
                        mut args,
                        polymorphism: 0,
                    },
                ) => {
                    args.push(value);
                    if args.len() == builtin.arity() as usize {
                        ret = builtin.apply(args, &mut program.constants)?;
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
                (
                    Some(Frame::Construct {
                        determinant,
                        mut remaining,
                        environment_len,
                        mut environment_and_values,
                    }),
                    value,
                ) => {
                    environment_and_values.push(value);
                    if remaining == 0 {
                        ret = Value::Construct {
                            determinant,
                            values: environment_and_values
                                .drain(environment_len as usize..)
                                .collect(),
                        };
                        continue;
                    }
                    remaining -= 1;
                    let environment = environment_and_values[..environment_len as usize].to_vec();
                    stack.push(Frame::Construct {
                        determinant,
                        remaining,
                        environment_len,
                        environment_and_values,
                    });
                    index += 1;
                    environment
                }
                (
                    Some(Frame::Case { count, environment }),
                    Value::Construct {
                        determinant,
                        values,
                    },
                ) if determinant < count => {
                    index = skip_terms(&program.program, index, determinant);
                    stack.extend(values.into_iter().map(Frame::ApplyLeftValue));
                    environment
                }
                (None, value) => {
                    return Some(value.discharge(program));
                }
                _ => return None,
            };
        }
    }
}

fn skip_terms<T>(terms: &[Instruction<T>], mut index: usize, count: u32) -> usize {
    let mut remaining = count;
    while remaining > 0 {
        match terms[index] {
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
                remaining += length as u32 - 1;
            }
            Instruction::Case { count } => {
                remaining += count;
            }
        }
        index += 1;
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_terms() {
        let terms = [
            Instruction::Constant(ConstantIndex(0)),
            Instruction::Delay,
            Instruction::Constant(ConstantIndex(1)),
            Instruction::Application,
            Instruction::Lambda(0),
            Instruction::Constant(ConstantIndex(2)),
        ]
        .as_slice();
        assert_eq!(skip_terms(terms, 0, 1), 1);
        assert_eq!(skip_terms(terms, 0, 2), 3);
        assert_eq!(skip_terms(terms, 1, 1), 3);
    }
}
