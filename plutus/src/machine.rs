//! Evaluation of programs according to the CEK machine defined in the [specification][spec]
//! section 2.4.
//!
//! [spec]: https://plutus.cardano.intersectmbo.org/resources/plutus-core-spec.pdf

use crate::{
    ConstantIndex, Context, DeBruijn, Instruction, Program, TermIndex, builtin::Builtin,
    constant::Constant,
};

pub mod bvt;

/// Represents a processed value in the CEK machine.
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
        discriminant: u32,
        large_discriminant: bool,
        values: Vec<Value>,
    },
    Builtin {
        builtin: Builtin,
        polymorphism: u8,
        args: Vec<Value>,
    },
}

impl Value {
    /// Discharge the value back into a program.
    ///
    /// Once a program is evaluated to a value, this value may still contain references to
    /// variables in its environment, which need to be discharged back into the program.
    ///
    /// This is defined in the [specification][spec] section 2.4.1.
    ///
    /// [spec]: https://plutus.cardano.intersectmbo.org/resources/plutus-core-spec.pdf
    fn discharge(self, mut program: Program<DeBruijn>) -> Program<DeBruijn> {
        enum DischargeValue {
            Constant(ConstantIndex),
            Term {
                term: TermIndex,
                // This is a number of terms, so u16 is sufficient.
                remaining: u16,
                environment: Vec<Value>,
            },
            Construct {
                discriminant: u32,
                large_discriminant: bool,
                values: Vec<DischargeValue>,
            },
            Builtin {
                builtin: Builtin,
                force_count: u8,
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
                        discriminant,
                        values,
                        large_discriminant,
                    } => DischargeValue::Construct {
                        discriminant,
                        values: values.into_iter().map(DischargeValue::from).collect(),
                        large_discriminant,
                    },
                    Value::Builtin {
                        builtin,
                        args,
                        polymorphism,
                    } => DischargeValue::Builtin {
                        builtin,
                        force_count: builtin.quantifiers() - polymorphism,
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
                    mut remaining,
                } => {
                    let mut index = term.0 as usize;
                    while remaining > 0 {
                        match program.program[index] {
                            Instruction::Delay | Instruction::Force | Instruction::Lambda(_) => {}
                            Instruction::Variable(DeBruijn(var)) => {
                                remaining -= 1;
                                if let Some(value) = environment.get(var as usize).cloned() {
                                    value_stack.push(DischargeValue::Term {
                                        term: TermIndex(index as u32 + 1),
                                        remaining,
                                        environment,
                                    });
                                    value_stack.push(DischargeValue::from(value));
                                    break;
                                }
                            }
                            Instruction::Constant(_)
                            | Instruction::Error
                            | Instruction::Builtin(_)
                            | Instruction::Construct { length: 0, .. } => {
                                remaining -= 1;
                            }
                            Instruction::Application => {
                                remaining += 1;
                            }
                            Instruction::Construct { length, .. } => {
                                remaining += length - 1;
                            }
                            Instruction::Case { count } => {
                                remaining += count;
                            }
                        }
                        instructions.push(program.program[index]);
                        index += 1;
                    }
                }
                DischargeValue::Construct {
                    discriminant,
                    large_discriminant,
                    values,
                } => {
                    instructions.push(Instruction::Construct {
                        discriminant,
                        large_discriminant,
                        length: values.len() as u16,
                    });
                    value_stack.extend(values.into_iter().rev());
                }
                DischargeValue::Builtin {
                    builtin,
                    args,
                    force_count,
                } => {
                    instructions.extend(std::iter::repeat_n(Instruction::Application, args.len()));
                    instructions.extend(std::iter::repeat_n(
                        Instruction::Force,
                        force_count as usize,
                    ));
                    instructions.push(Instruction::Builtin(builtin));
                    value_stack.extend(args.into_iter().rev());
                }
            }
        }
        program.program = instructions;
        program
    }
}

/// Represents a frame of the CEK machine's stack.
///
/// Defined in the [specification][spec] figure 2.9.
///
/// [spec]: https://plutus.cardano.intersectmbo.org/resources/plutus-core-spec.pdf
pub enum Frame {
    Force,
    ApplyLeftValue(Value),
    ApplyRightValue(Value),
    ApplyLeftTerm {
        environment: Vec<Value>,
        next: TermIndex,
    },
    Construct {
        remaining: u16,
        discriminant: u32,
        large_discriminant: bool,
        environment_len: u16,
        environment_and_values: Vec<Value>,
    },
    Case {
        count: u16,
        next: TermIndex,
        environment: Vec<Value>,
    },
}

// Some ideas to make this faster:
// - Find a way to avoid cloning the environment so much. We can probably use prefixes for this,
// and only clone some of the time if we need to pop off the env.
// - Find a way to not store `next` and not `skip_terms` so much.
// - Don't clone constants all the time, only clone if they come from the environment.
// - Make builtins borrow by default.

/// Run the given program according to the CEK machine.
pub fn run(mut program: Program<DeBruijn>, context: &mut Context<'_>) -> Option<Program<DeBruijn>> {
    let base_costs = context.base()?;
    context.apply_no_args(&base_costs.startup)?;

    let mut stack = Vec::new();
    let mut environment: Vec<Value> = Vec::new();
    let mut index = 0;

    loop {
        let mut ret = match program.program[index] {
            Instruction::Variable(var) => {
                context.apply_no_args(&base_costs.variable)?;
                environment
                    .get(var.0 as usize)
                    .expect("variable exists")
                    .clone()
            }
            Instruction::Delay => {
                context.apply_no_args(&base_costs.delay)?;
                Value::Delay {
                    term: TermIndex(index as u32),
                    environment,
                }
            }
            Instruction::Lambda(_) => {
                context.apply_no_args(&base_costs.lambda)?;
                Value::Lambda {
                    term: TermIndex(index as u32),
                    environment,
                }
            }
            Instruction::Application => {
                context.apply_no_args(&base_costs.application)?;
                index += 1;
                stack.push(Frame::ApplyLeftTerm {
                    environment: environment.clone(),
                    next: TermIndex(skip_terms(&program.program, index, 1) as u32),
                });
                continue;
            }
            Instruction::Constant(constant_index) => {
                context.apply_no_args(&base_costs.constant)?;
                Value::Constant(constant_index)
            }
            Instruction::Force => {
                context.apply_no_args(&base_costs.force)?;
                stack.push(Frame::Force);
                index += 1;
                continue;
            }
            Instruction::Error => {
                return None;
            }
            Instruction::Builtin(builtin) => {
                context.apply_no_args(&base_costs.builtin)?;
                Value::Builtin {
                    builtin,
                    polymorphism: builtin.quantifiers(),
                    args: Vec::new(),
                }
            }
            Instruction::Construct {
                discriminant,
                large_discriminant,
                length,
            } => {
                context.apply_no_args(&context.datatypes()?.construct)?;
                if length != 0 {
                    stack.push(Frame::Construct {
                        remaining: length - 1,
                        environment_len: environment.len() as u16,
                        environment_and_values: environment.clone(),
                        discriminant,
                        large_discriminant,
                    });
                    index += 1;
                    continue;
                }

                Value::Construct {
                    discriminant,
                    large_discriminant,
                    values: Vec::new(),
                }
            }
            Instruction::Case { count } => {
                context.apply_no_args(&context.datatypes()?.case)?;
                index += 1;
                stack.push(Frame::Case {
                    count,
                    environment: environment.clone(),
                    next: TermIndex(skip_terms(&program.program, index, 1) as u32),
                });
                continue;
            }
        };

        environment = loop {
            break match (stack.pop(), ret) {
                (Some(Frame::Force), Value::Delay { term, environment }) => {
                    index = term.0 as usize + 1;
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

                (Some(Frame::ApplyLeftTerm { environment, next }), value) => {
                    stack.push(Frame::ApplyRightValue(value));
                    index = next.0 as usize;
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
                    index = term.0 as usize + 1;
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
                        ret = builtin.apply(args, &mut program.constants, context)?;
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
                        discriminant,
                        large_discriminant,
                        mut remaining,
                        environment_len,
                        mut environment_and_values,
                    }),
                    value,
                ) => {
                    environment_and_values.push(value);
                    if remaining == 0 {
                        ret = Value::Construct {
                            discriminant,
                            large_discriminant,
                            values: environment_and_values
                                .drain(environment_len as usize..)
                                .collect(),
                        };
                        continue;
                    }
                    remaining -= 1;
                    let environment = environment_and_values[..environment_len as usize].to_vec();
                    stack.push(Frame::Construct {
                        discriminant,
                        large_discriminant,
                        remaining,
                        environment_len,
                        environment_and_values,
                    });
                    index = skip_terms(&program.program, index, 1);
                    environment
                }
                (
                    Some(Frame::Case {
                        count,
                        environment,
                        next,
                    }),
                    Value::Construct {
                        discriminant,
                        large_discriminant,
                        values,
                    },
                ) if discriminant < count as u32 || large_discriminant => {
                    let discriminant = if large_discriminant {
                        let Constant::Integer(discriminant) =
                            &program.constants[discriminant as usize]
                        else {
                            panic!("large discriminant did not point to an integer constant");
                        };
                        discriminant.to_u64().expect("discriminant fits in u64")
                    } else {
                        discriminant as u64
                    };

                    stack.extend(values.into_iter().map(Frame::ApplyLeftValue).rev());
                    index = skip_terms(&program.program, next.0 as usize, discriminant);
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

/// Skip over `count` terms in the instruction list, starting from `index`.
fn skip_terms<T>(terms: &[Instruction<T>], mut index: usize, count: u64) -> usize {
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
                remaining += length as u64 - 1;
            }
            Instruction::Case { count } => {
                remaining += count as u64;
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
