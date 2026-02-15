//! Evaluation of programs by a CEK machine.
//!
//! Defined in the [specification][spec] section 2.4.
//!
//! [spec]: https://plutus.cardano.intersectmbo.org/resources/plutus-core-spec.pdf

use std::rc::Rc;

use crate::{
    ConstantIndex, Context, DeBruijn, Instruction, Program, TermIndex, builtin::Builtin,
    constant::Constant,
};
use bvt::Vector;

pub mod bvt;

/// Represents a processed value in the CEK machine.
#[derive(Debug, Clone)]
pub(crate) enum Value<'a> {
    Constant(Constant<'a>),
    Delay {
        term: TermIndex,
        environment: Vector<Value<'a>>,
    },
    Lambda {
        term: TermIndex,
        environment: Vector<Value<'a>>,
    },
    Construct {
        discriminant: u32,
        large_discriminant: bool,
        values: Rc<[Value<'a>]>,
    },
    Builtin {
        builtin: Builtin,
        polymorphism: u8,
        args: Vec<Value<'a>>,
    },
}

impl<'a> Value<'a> {
    /// Discharge the value back into a program.
    ///
    /// Once a program is evaluated to a value, this value may still contain references to
    /// variables in its environment, which need to be discharged back into the program.
    ///
    /// This is defined in the [specification][spec] section 2.4.1.
    ///
    /// [spec]: https://plutus.cardano.intersectmbo.org/resources/plutus-core-spec.pdf
    fn discharge(self, mut program: Program<'a, DeBruijn>) -> Program<'a, u32> {
        enum DischargeValue<'a> {
            Constant(Constant<'a>),
            Term {
                term: TermIndex,
                // This is a number of terms, so u16 is sufficient.
                stack: Vec<Option<u32>>,
                environment: Vector<Value<'a>>,
            },
            Construct {
                discriminant: u32,
                large_discriminant: bool,
                values: Vec<DischargeValue<'a>>,
            },
            Builtin {
                builtin: Builtin,
                force_count: u8,
                args: Vec<DischargeValue<'a>>,
            },
        }

        impl<'a> From<Value<'a>> for DischargeValue<'a> {
            fn from(value: Value<'a>) -> Self {
                match value {
                    Value::Constant(constant) => DischargeValue::Constant(constant),
                    Value::Delay { term, environment } | Value::Lambda { term, environment } => {
                        DischargeValue::Term {
                            term,
                            stack: vec![None],
                            environment,
                        }
                    }
                    Value::Construct {
                        discriminant,
                        values,
                        large_discriminant,
                    } => DischargeValue::Construct {
                        discriminant,
                        values: values.iter().cloned().map(DischargeValue::from).collect(),
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

        struct Frame<'a> {
            value: DischargeValue<'a>,
            write_back: Option<u32>,
        }

        let mut value_stack = vec![Frame {
            value: DischargeValue::from(self),
            write_back: None,
        }];
        let mut instructions = Vec::new();
        while let Some(Frame { value, write_back }) = value_stack.pop() {
            match value {
                DischargeValue::Constant(constant) => {
                    instructions.push(Instruction::Constant({
                        let constant_index = ConstantIndex(program.constants.len() as u32);
                        program.constants.push(constant);
                        constant_index
                    }));
                }
                DischargeValue::Term {
                    term,
                    environment,
                    mut stack,
                } => {
                    // FIXME: a stack within a stack is a bit unfortunate.
                    // There is probably a more elegant way with the outer stack and a more complex
                    // frame.
                    let mut index = term.0 as usize;
                    while let Some(frame) = stack.pop() {
                        if let Some(write_back) = frame {
                            let index = instructions.len() as u32;
                            let Instruction::Application(wb) =
                                &mut instructions[write_back as usize]
                            else {
                                unreachable!(
                                    "write_back should point to an application instruction"
                                );
                            };
                            *wb = TermIndex(index);
                        }

                        match program.program[index] {
                            Instruction::Delay | Instruction::Force | Instruction::Lambda(_) => {
                                stack.push(frame);
                            }
                            Instruction::Variable(DeBruijn(var)) => {
                                if let Some(value) = environment.get(var as usize).cloned() {
                                    value_stack.push(Frame {
                                        value: DischargeValue::Term {
                                            term: TermIndex(index as u32 + 1),
                                            stack,
                                            environment,
                                        },
                                        write_back: None,
                                    });
                                    value_stack.push(Frame {
                                        value: DischargeValue::from(value),
                                        write_back: None,
                                    });
                                    break;
                                }
                            }
                            Instruction::Constant(_)
                            | Instruction::Error
                            | Instruction::Builtin(_) => {}
                            Instruction::Application(_) => {
                                stack.push(Some(instructions.len() as u32));
                                stack.push(None);
                            }
                            Instruction::Construct { length, .. } => {
                                stack.extend(std::iter::repeat_n(None, length as usize));
                            }
                            Instruction::Case { count } => {
                                stack.extend(std::iter::repeat_n(None, count as usize + 1));
                            }
                        }
                        instructions.push(map_var(program.program[index], |DeBruijn(var)| var));
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
                    value_stack.extend(
                        values
                            .into_iter()
                            .map(|v| Frame {
                                value: v,
                                write_back: None,
                            })
                            .rev(),
                    );
                }
                DischargeValue::Builtin {
                    builtin,
                    args,
                    force_count,
                } => {
                    let first_application_index = instructions.len();
                    instructions.extend(std::iter::repeat_n(
                        Instruction::Application(TermIndex(0)),
                        args.len(),
                    ));
                    instructions.extend(std::iter::repeat_n(
                        Instruction::Force,
                        force_count as usize,
                    ));
                    instructions.push(Instruction::Builtin(builtin));
                    let mut args = args.into_iter().rev();
                    if let Some(last) = args.next_back() {
                        let last_application_index = instructions.len() - force_count as usize - 2;
                        instructions[last_application_index] =
                            Instruction::Application(TermIndex(instructions.len() as u32));
                        value_stack.extend(args.enumerate().map(|(i, v)| Frame {
                            value: v,
                            write_back: Some((first_application_index + i) as u32),
                        }));
                        value_stack.push(Frame {
                            value: last,
                            write_back: None,
                        });
                    }
                }
            }
            if let Some(write_back) = write_back {
                let index = instructions.len() as u32;
                let Instruction::Application(wb) = &mut instructions[write_back as usize] else {
                    unreachable!("write_back should point to an application instruction");
                };
                *wb = TermIndex(index);
            }
        }

        Program {
            version: program.version,
            arena: program.arena,
            constants: program.constants,
            program: instructions,
        }
    }
}

/// Represents a frame of the CEK machine's stack.
///
/// Defined in the [specification][spec] figure 2.9.
///
/// [spec]: https://plutus.cardano.intersectmbo.org/resources/plutus-core-spec.pdf
#[derive(Debug)]
enum Frame<'a> {
    Force,
    ApplyLeftValue(Value<'a>),
    ApplyRightValue(Value<'a>),
    ApplyLeftTerm {
        environment: Vector<Value<'a>>,
        next: TermIndex,
    },
    Construct {
        remaining: u16,
        discriminant: u32,
        large_discriminant: bool,
        values: Vec<Value<'a>>,
        environment: Vector<Value<'a>>,
        next: TermIndex,
    },
    Case {
        count: u16,
        next: TermIndex,
        environment: Vector<Value<'a>>,
    },
}

/// Run the given program according to the CEK machine.
pub fn run<'a>(
    program: Program<'a, DeBruijn>,
    context: &mut Context<'_>,
) -> Option<Program<'a, u32>> {
    let base_costs = context.base()?;
    context.apply_no_args(&base_costs.startup)?;

    let mut stack = Vec::new();
    let mut environment: Vector<Value> = Vector::default();
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
            Instruction::Application(next) => {
                context.apply_no_args(&base_costs.application)?;
                index += 1;
                stack.push(Frame::ApplyLeftTerm {
                    environment: environment.clone(),
                    next,
                });
                continue;
            }
            Instruction::Constant(constant_index) => {
                context.apply_no_args(&base_costs.constant)?;
                Value::Constant(program.constants[constant_index.0 as usize])
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
                index += 1;
                if length != 0 {
                    stack.push(Frame::Construct {
                        remaining: length - 1,
                        environment: environment.clone(),
                        values: Vec::new(),
                        discriminant,
                        large_discriminant,
                        next: TermIndex(skip_terms(&program.program, index, 1) as u32),
                    });
                    continue;
                }

                Value::Construct {
                    discriminant,
                    large_discriminant,
                    values: Rc::new([]),
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
                        ret = builtin.apply(args, program.arena, context)?;
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
                        environment,
                        mut values,
                        next,
                    }),
                    value,
                ) => {
                    values.push(value);
                    if remaining == 0 {
                        ret = Value::Construct {
                            discriminant,
                            large_discriminant,
                            values: values.into(),
                        };
                        continue;
                    }
                    remaining -= 1;
                    index = next.0 as usize;
                    stack.push(Frame::Construct {
                        discriminant,
                        next: TermIndex(skip_terms(&program.program, index, 1) as u32),
                        large_discriminant,
                        remaining,
                        environment: environment.clone(),
                        values,
                    });
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

                    stack.extend(values.iter().cloned().map(Frame::ApplyLeftValue).rev());
                    index = skip_terms(&program.program, next.0 as usize, discriminant);
                    environment
                }
                (
                    Some(Frame::Case {
                        count,
                        next,
                        environment,
                    }),
                    Value::Constant(constant),
                ) => {
                    index = match constant {
                        Constant::Integer(integer) => {
                            let discriminant = integer.to_u16()?;
                            if discriminant >= count {
                                return None;
                            }
                            skip_terms(&program.program, next.0 as usize, discriminant as u64)
                        }
                        Constant::Unit => {
                            if count != 1 {
                                return None;
                            }
                            next.0 as usize
                        }
                        Constant::Boolean(bool) => {
                            let discriminant = if bool { 1 } else { 0 };
                            if !(1..=2).contains(&count) || discriminant >= count {
                                return None;
                            }
                            skip_terms(&program.program, next.0 as usize, discriminant as u64)
                        }
                        Constant::List(list) => {
                            if let Some(tail) = crate::builtin::list::tail(list) {
                                stack.push(Frame::ApplyLeftValue(Value::Constant(Constant::List(
                                    tail,
                                ))));
                            };
                            if let Some(head) = crate::builtin::list::head(list) {
                                stack.push(Frame::ApplyLeftValue(Value::Constant(head)));
                            };
                            let discriminant = if crate::builtin::list::null(list) {
                                1
                            } else {
                                0
                            };

                            if !(1..=2).contains(&count) || discriminant >= count {
                                return None;
                            }
                            skip_terms(&program.program, next.0 as usize, discriminant as u64)
                        }
                        Constant::Pair(first, second) => {
                            if count != 1 {
                                return None;
                            }
                            stack.push(Frame::ApplyLeftValue(Value::Constant(*second)));
                            stack.push(Frame::ApplyLeftValue(Value::Constant(*first)));
                            next.0 as usize
                        }
                        _ => return None,
                    };
                    environment
                }
                (None, value) => {
                    let program = value.discharge(program);
                    return Some(program);
                }
                _ => return None,
            };
        };
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
            Instruction::Application(_) => {
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

fn map_var<T, U>(instruction: Instruction<T>, f: impl FnOnce(T) -> U) -> Instruction<U> {
    match instruction {
        Instruction::Variable(var) => Instruction::Variable(f(var)),
        Instruction::Delay => Instruction::Delay,
        Instruction::Lambda(n) => Instruction::Lambda(f(n)),
        Instruction::Application(term) => Instruction::Application(term),
        Instruction::Constant(constant_index) => Instruction::Constant(constant_index),
        Instruction::Force => Instruction::Force,
        Instruction::Error => Instruction::Error,
        Instruction::Builtin(builtin) => Instruction::Builtin(builtin),
        Instruction::Construct {
            discriminant,
            large_discriminant,
            length,
        } => Instruction::Construct {
            discriminant,
            large_discriminant,
            length,
        },
        Instruction::Case { count } => Instruction::Case { count },
    }
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
            Instruction::Application(TermIndex(4)),
            Instruction::Lambda(0),
            Instruction::Constant(ConstantIndex(2)),
        ]
        .as_slice();
        assert_eq!(skip_terms(terms, 0, 1), 1);
        assert_eq!(skip_terms(terms, 0, 2), 3);
        assert_eq!(skip_terms(terms, 1, 1), 3);
    }
}
