use crate::{
    ConstantIndex, TermIndex,
    builtin::Builtin,
    program::{Instruction, Program},
};

#[derive(Debug, Clone)]
pub enum Value {
    Constant(ConstantIndex),
    Delay(TermIndex),
    Lambda(TermIndex),
    Construct {
        determinant: u16,
        values: Vec<Value>,
    },
    Builtin {
        builtin: Builtin,
        args: Vec<Value>,
    },
}

pub enum Frame {
    Force,
    ApplyLeftTerm,
    ApplyLeftValue(Value),
    ApplyRightValue(Value),
    Construct {
        determinant: u16,
        remaining: u32,
        collected: Vec<Value>,
    },
    Case {
        count: u32,
    },
}

pub fn run(program: Program<u32>) -> Option<Value> {
    let mut stack = Vec::new();
    let mut environment: Vec<Value> = Vec::new();
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
                args: Vec::new(),
            },
            Instruction::Construct {
                determinant,
                length,
            } => {
                if length != 0 {
                    stack.push(Frame::Construct {
                        determinant,
                        remaining: length - 1,
                        collected: Vec::new(),
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
                (Some(Frame::Force), Value::Builtin { builtin, args }) => {
                    todo!()
                }

                (Some(Frame::ApplyLeftTerm), value) => {
                    stack.push(Frame::ApplyRightValue(value));
                    index += 1;
                }
                (Some(Frame::ApplyRightValue(Value::Lambda(TermIndex(term_index)))), value) => {
                    environment.push(value);
                    index = term_index as usize;
                }
                (Some(Frame::ApplyRightValue(Value::Builtin { builtin, args })), value) => {
                    todo!()
                }
                (Some(Frame::ApplyLeftValue(value)), Value::Builtin { builtin, args }) => {
                    todo!()
                }
                (Some(Frame::ApplyLeftValue(value)), Value::Lambda(term_index)) => {
                    environment.push(value);
                    index = term_index.0 as usize;
                }

                (
                    Some(Frame::Construct {
                        determinant,
                        mut remaining,
                        mut collected,
                    }),
                    value,
                ) => {
                    collected.push(value);
                    if remaining == 0 {
                        ret = Value::Construct {
                            determinant,
                            values: collected,
                        };
                        continue;
                    }
                    remaining -= 1;
                    stack.push(Frame::Construct {
                        determinant,
                        remaining,
                        collected,
                    });
                    index += 1;
                }
                (Some(Frame::Case { count }), Value::Construct {
                    determinant,
                    values,
                }) if (determinant as u32) < count => {
                    todo!()
                },
                (None, value) => {
                    return Some(value);
                }
                _ => return None,
            };
            break;
        }
    }
}
