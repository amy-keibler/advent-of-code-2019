use thiserror::Error;

use std::collections::VecDeque;
use std::convert::TryFrom;

#[derive(Debug, PartialEq, Error)]
pub enum ExecutionError {
    #[error("Unsupported operation code {code} found at position {index}")]
    InvalidOperationCode { index: usize, code: i32 },
    #[error("Operation attempted to index position {index}, but program has the length of {program_length}")]
    IndexOutsideOfProgram { index: i32, program_length: usize },
    #[error("Invalid operation index found for operation at position {index}")]
    InvalidOperationIndex { index: i32 },
    #[error("No input available for operation at position {index}")]
    InvalidRequestForInput { index: usize },
}

#[derive(Debug, PartialEq)]
enum Operation {
    Add(ParameterMode, ParameterMode),
    Multiply(ParameterMode, ParameterMode),
    StoreInput,
    ProduceOutput(ParameterMode),
    JumpIfTrue(ParameterMode, ParameterMode),
    JumpIfFalse(ParameterMode, ParameterMode),
    LessThan(ParameterMode, ParameterMode),
    EqualTo(ParameterMode, ParameterMode),
    Terminate,
}

impl Operation {
    fn number_of_parameters(&self) -> usize {
        match self {
            Self::Add(_, _) | Self::Multiply(_, _) | Self::LessThan(_, _) | Self::EqualTo(_, _) => {
                3
            }
            Self::JumpIfTrue(_, _) | Self::JumpIfFalse(_, _) => 2,
            Self::StoreInput | Self::ProduceOutput(_) => 1,
            Self::Terminate => 0,
        }
    }
}

impl TryFrom<i32> for Operation {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        let code = value % 100;
        let mut parameter_modes = value / 100;
        match code {
            1 => {
                let left_mode = extract_parameter_mode(&mut parameter_modes)?;
                let right_mode = extract_parameter_mode(&mut parameter_modes)?;
                Ok(Self::Add(left_mode, right_mode))
            }
            2 => {
                let left_mode = extract_parameter_mode(&mut parameter_modes)?;
                let right_mode = extract_parameter_mode(&mut parameter_modes)?;
                Ok(Self::Multiply(left_mode, right_mode))
            }
            3 => Ok(Self::StoreInput),
            4 => {
                let mode = extract_parameter_mode(&mut parameter_modes)?;
                Ok(Self::ProduceOutput(mode))
            }
            5 => {
                let true_mode = extract_parameter_mode(&mut parameter_modes)?;
                let jump_mode = extract_parameter_mode(&mut parameter_modes)?;
                Ok(Self::JumpIfTrue(true_mode, jump_mode))
            }
            6 => {
                let false_mode = extract_parameter_mode(&mut parameter_modes)?;
                let jump_mode = extract_parameter_mode(&mut parameter_modes)?;
                Ok(Self::JumpIfFalse(false_mode, jump_mode))
            }
            7 => {
                let left_mode = extract_parameter_mode(&mut parameter_modes)?;
                let right_mode = extract_parameter_mode(&mut parameter_modes)?;
                Ok(Self::LessThan(left_mode, right_mode))
            }
            8 => {
                let left_mode = extract_parameter_mode(&mut parameter_modes)?;
                let right_mode = extract_parameter_mode(&mut parameter_modes)?;
                Ok(Self::EqualTo(left_mode, right_mode))
            }
            99 => Ok(Self::Terminate),
            n => Err(format!("Invalid Operation {}", n)),
        }
    }
}

fn extract_parameter_mode(parameter_modes: &mut i32) -> Result<ParameterMode, String> {
    let parameter_mode = ParameterMode::try_from(*parameter_modes % 10)?;
    *parameter_modes /= 10;

    Ok(parameter_mode)
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum ParameterMode {
    Position,
    Immediate,
}

impl TryFrom<i32> for ParameterMode {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Position),
            1 => Ok(Self::Immediate),
            n => Err(format!("Invalid Parameter Mode {}", n)),
        }
    }
}

#[derive(Debug, PartialEq)]
enum ExecutionStatus {
    Ongoing,
    Terminated,
}

pub struct IntcodeComputer {
    program_counter: usize,
    memory: Vec<i32>,
    input: VecDeque<i32>,
    output: VecDeque<i32>,
}

impl IntcodeComputer {
    pub fn new(memory: Vec<i32>) -> IntcodeComputer {
        IntcodeComputer {
            program_counter: 0,
            memory,
            input: VecDeque::new(),
            output: VecDeque::new(),
        }
    }
    pub fn new_with_input(memory: Vec<i32>, input: VecDeque<i32>) -> IntcodeComputer {
        IntcodeComputer {
            program_counter: 0,
            memory,
            input,
            output: VecDeque::new(),
        }
    }

    pub fn execute(&mut self) -> Result<VecDeque<i32>, ExecutionError> {
        loop {
            let operation_code =
                self.memory
                    .get(self.program_counter)
                    .copied()
                    .ok_or_else(|| ExecutionError::InvalidOperationIndex {
                        index: self.program_counter as i32,
                    })?;
            let operation = Operation::try_from(operation_code).map_err(|_| {
                ExecutionError::InvalidOperationCode {
                    index: self.program_counter,
                    code: operation_code,
                }
            })?;
            let result = self.perform_operation(operation)?;
            if ExecutionStatus::Terminated == result {
                return Ok(self.output.clone());
            }
        }
    }

    fn perform_operation(
        &mut self,
        operation: Operation,
    ) -> Result<ExecutionStatus, ExecutionError> {
        match operation {
            Operation::Add(left_mode, right_mode) => {
                self.perform_function(left_mode, right_mode, std::ops::Add::add)?;
            }
            Operation::Multiply(left_mode, right_mode) => {
                self.perform_function(left_mode, right_mode, std::ops::Mul::mul)?;
            }
            Operation::StoreInput => {
                if let Some(input) = self.input.pop_front() {
                    let output_index =
                        self.fetch_parameter(ParameterMode::Immediate, self.program_counter + 1)?;
                    self.set_memory(output_index, input)?;
                } else {
                    return Err(ExecutionError::InvalidRequestForInput {
                        index: self.program_counter,
                    });
                }
            }
            Operation::ProduceOutput(mode) => {
                let output = self.fetch_parameter(mode, self.program_counter + 1)?;
                self.output.push_back(output);
            }
            Operation::JumpIfTrue(true_mode, jump_mode) => {
                return self.perform_jump(true_mode, jump_mode, |value| value != 0)
            }
            Operation::JumpIfFalse(false_mode, jump_mode) => {
                return self.perform_jump(false_mode, jump_mode, |value| value == 0)
            }
            Operation::LessThan(left_mode, right_mode) => {
                self.perform_function(
                    left_mode,
                    right_mode,
                    wrap_boolean_fn(|left, right| left < right),
                )?;
            }
            Operation::EqualTo(left_mode, right_mode) => {
                self.perform_function(
                    left_mode,
                    right_mode,
                    wrap_boolean_fn(|left, right| left == right),
                )?;
            }
            Operation::Terminate => {
                self.program_counter += 1 + operation.number_of_parameters();
                return Ok(ExecutionStatus::Terminated);
            }
        };
        self.program_counter += 1 + operation.number_of_parameters();
        Ok(ExecutionStatus::Ongoing)
    }

    fn fetch_parameter(&self, mode: ParameterMode, index: usize) -> Result<i32, ExecutionError> {
        let value = self.memory.get(index).copied().ok_or_else(|| {
            ExecutionError::IndexOutsideOfProgram {
                index: index as i32,
                program_length: self.memory.len(),
            }
        })?;
        match mode {
            ParameterMode::Position => {
                if value < 0 {
                    return Err(ExecutionError::IndexOutsideOfProgram {
                        index: value,
                        program_length: self.memory.len(),
                    });
                }
                self.fetch_parameter(ParameterMode::Immediate, value as usize)
            }
            ParameterMode::Immediate => Ok(value),
        }
    }

    fn perform_function(
        &mut self,
        left_mode: ParameterMode,
        right_mode: ParameterMode,
        operation: impl FnOnce(i32, i32) -> i32,
    ) -> Result<(), ExecutionError> {
        let left = self.fetch_parameter(left_mode, self.program_counter + 1)?;
        let right = self.fetch_parameter(right_mode, self.program_counter + 2)?;
        let output_index = self
            .memory
            .get(self.program_counter + 3)
            .copied()
            .ok_or_else(|| ExecutionError::IndexOutsideOfProgram {
                index: (self.program_counter + 3) as i32,
                program_length: self.memory.len(),
            })?;
        self.set_memory(output_index, operation(left, right))
    }

    fn perform_jump(
        &mut self,
        true_mode: ParameterMode,
        jump_mode: ParameterMode,
        operation: impl FnOnce(i32) -> bool,
    ) -> Result<ExecutionStatus, ExecutionError> {
        let truth_value = self.fetch_parameter(true_mode, self.program_counter + 1)?;
        if operation(truth_value) {
            let new_program_counter = self.fetch_parameter(jump_mode, self.program_counter + 2)?;
            if new_program_counter < 0 {
                return Err(ExecutionError::InvalidOperationIndex {
                    index: new_program_counter,
                });
            }
            self.program_counter = new_program_counter as usize;
            return Ok(ExecutionStatus::Ongoing);
        }

        self.program_counter += 1 + 2;
        Ok(ExecutionStatus::Ongoing)
    }

    fn set_memory(&mut self, index: i32, value: i32) -> Result<(), ExecutionError> {
        if index < 0 {
            return Err(ExecutionError::IndexOutsideOfProgram {
                index,
                program_length: self.memory.len(),
            });
        }
        self.memory
            .get_mut(index as usize)
            .map(|output| *output = value)
            .ok_or_else(|| ExecutionError::IndexOutsideOfProgram {
                index,
                program_length: self.memory.len(),
            })
    }
}

fn wrap_boolean_fn(to_wrap: impl Fn(i32, i32) -> bool) -> impl Fn(i32, i32) -> i32 {
    move |left, right| {
        if to_wrap(left, right) {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_properly_convert_operations() {
        assert_eq!(
            Ok(Operation::Add(
                ParameterMode::Immediate,
                ParameterMode::Position
            )),
            Operation::try_from(101)
        );
        assert_eq!(
            Ok(Operation::Multiply(
                ParameterMode::Position,
                ParameterMode::Immediate
            )),
            Operation::try_from(1002)
        );
        assert_eq!(Ok(Operation::StoreInput), Operation::try_from(3));
        assert_eq!(
            Ok(Operation::ProduceOutput(ParameterMode::Position)),
            Operation::try_from(4)
        );
        assert_eq!(
            Ok(Operation::JumpIfTrue(
                ParameterMode::Immediate,
                ParameterMode::Position
            )),
            Operation::try_from(105)
        );
        assert_eq!(
            Ok(Operation::JumpIfFalse(
                ParameterMode::Immediate,
                ParameterMode::Position
            )),
            Operation::try_from(106)
        );
        assert_eq!(
            Ok(Operation::LessThan(
                ParameterMode::Immediate,
                ParameterMode::Position
            )),
            Operation::try_from(107)
        );
        assert_eq!(
            Ok(Operation::EqualTo(
                ParameterMode::Immediate,
                ParameterMode::Position
            )),
            Operation::try_from(108)
        );
        assert_eq!(Ok(Operation::Terminate), Operation::try_from(99));
    }

    #[test]
    fn it_should_fail_for_an_invalid_operation_code() {
        assert_eq!(
            Err(String::from("Invalid Operation 10")),
            Operation::try_from(10)
        );
        assert_eq!(
            Err(String::from("Invalid Parameter Mode 3")),
            Operation::try_from(301)
        );
    }

    fn setup_computer(memory: Vec<i32>) -> IntcodeComputer {
        IntcodeComputer {
            program_counter: 0,
            memory,
            input: VecDeque::new(),
            output: VecDeque::new(),
        }
    }

    #[test]
    fn it_should_perform_an_addition() {
        let mut computer = setup_computer(vec![1, 0, 0, 3]);
        let status = computer
            .perform_operation(Operation::Add(
                ParameterMode::Position,
                ParameterMode::Position,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![1, 0, 0, 2], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_perform_an_addition_in_immediate_mode() {
        let mut computer = setup_computer(vec![1101, 2, 2, 3]);
        let status = computer
            .perform_operation(Operation::Add(
                ParameterMode::Immediate,
                ParameterMode::Immediate,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![1101, 2, 2, 4], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_perform_a_multiplication() {
        let mut computer = setup_computer(vec![2, 0, 0, 3]);
        let status = computer
            .perform_operation(Operation::Multiply(
                ParameterMode::Position,
                ParameterMode::Position,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![2, 0, 0, 4], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_perform_a_multiplication_in_immediate_mode() {
        let mut computer = setup_computer(vec![1102, 3, 3, 3]);
        let status = computer
            .perform_operation(Operation::Multiply(
                ParameterMode::Immediate,
                ParameterMode::Immediate,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![1102, 3, 3, 9], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_retrieve_input() {
        let mut computer = IntcodeComputer {
            program_counter: 0,
            memory: vec![3, 3, 0, 0],
            input: vec![5].into(),
            output: VecDeque::new(),
        };
        let status = computer
            .perform_operation(Operation::StoreInput)
            .expect("Failed to execute operation");
        assert_eq!(vec![3, 3, 0, 5], computer.memory);
        assert_eq!(VecDeque::new(), computer.input);
        assert_eq!(2, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_produce_output() {
        let mut computer = IntcodeComputer {
            program_counter: 0,
            memory: vec![4, 3, 0, 5],
            input: VecDeque::new(),
            output: VecDeque::new(),
        };
        let status = computer
            .perform_operation(Operation::ProduceOutput(ParameterMode::Position))
            .expect("Failed to execute operation");
        assert_eq!(vec![4, 3, 0, 5], computer.memory);
        assert_eq!(VecDeque::from(vec![5]), computer.output);
        assert_eq!(2, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_produce_output_in_immediate_mode() {
        let mut computer = IntcodeComputer {
            program_counter: 0,
            memory: vec![4, 3, 0, 5],
            input: VecDeque::new(),
            output: VecDeque::new(),
        };
        let status = computer
            .perform_operation(Operation::ProduceOutput(ParameterMode::Immediate))
            .expect("Failed to execute operation");
        assert_eq!(vec![4, 3, 0, 5], computer.memory);
        assert_eq!(VecDeque::from(vec![3]), computer.output);
        assert_eq!(2, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_jump_if_true() {
        // perform jump
        let mut computer = setup_computer(vec![5, 1, 3, 2]);
        let status = computer
            .perform_operation(Operation::JumpIfTrue(
                ParameterMode::Position,
                ParameterMode::Position,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![5, 1, 3, 2], computer.memory);
        assert_eq!(2, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);

        // no jump performed
        let mut computer = setup_computer(vec![5, 4, 3, 2, 0]);
        let status = computer
            .perform_operation(Operation::JumpIfTrue(
                ParameterMode::Position,
                ParameterMode::Position,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![5, 4, 3, 2, 0], computer.memory);
        assert_eq!(3, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_jump_if_true_in_immediate_mode() {
        // perform jump
        let mut computer = setup_computer(vec![1105, 1, 4, 2, 0]);
        let status = computer
            .perform_operation(Operation::JumpIfTrue(
                ParameterMode::Immediate,
                ParameterMode::Immediate,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![1105, 1, 4, 2, 0], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);

        // no jump performed
        let mut computer = setup_computer(vec![1105, 0, 4, 2, 0]);
        let status = computer
            .perform_operation(Operation::JumpIfTrue(
                ParameterMode::Immediate,
                ParameterMode::Immediate,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![1105, 0, 4, 2, 0], computer.memory);
        assert_eq!(3, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_jump_if_false() {
        // perform jump
        let mut computer = setup_computer(vec![6, 4, 2, 2, 0]);
        let status = computer
            .perform_operation(Operation::JumpIfFalse(
                ParameterMode::Position,
                ParameterMode::Position,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![6, 4, 2, 2, 0], computer.memory);
        assert_eq!(2, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);

        // no jump performed
        let mut computer = setup_computer(vec![6, 4, 2, 2, 1]);
        let status = computer
            .perform_operation(Operation::JumpIfFalse(
                ParameterMode::Position,
                ParameterMode::Position,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![6, 4, 2, 2, 1], computer.memory);
        assert_eq!(3, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_jump_if_false_in_immediate_mode() {
        // perform jump
        let mut computer = setup_computer(vec![1106, 0, 4, 2]);
        let status = computer
            .perform_operation(Operation::JumpIfFalse(
                ParameterMode::Immediate,
                ParameterMode::Immediate,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![1106, 0, 4, 2], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);

        // no jump performed
        let mut computer = setup_computer(vec![1106, 1, 3, 2]);
        let status = computer
            .perform_operation(Operation::JumpIfFalse(
                ParameterMode::Immediate,
                ParameterMode::Immediate,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![1106, 1, 3, 2], computer.memory);
        assert_eq!(3, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_evaluate_less_than() {
        // less than
        let mut computer = setup_computer(vec![7, 4, 5, 6, 0, 1, 0]);
        let status = computer
            .perform_operation(Operation::LessThan(
                ParameterMode::Position,
                ParameterMode::Position,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![7, 4, 5, 6, 0, 1, 1], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);

        // not less than
        let mut computer = setup_computer(vec![7, 4, 5, 6, 1, 0, 1]);
        let status = computer
            .perform_operation(Operation::LessThan(
                ParameterMode::Position,
                ParameterMode::Position,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![7, 4, 5, 6, 1, 0, 0], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_evaluate_less_than_in_immediate_mode() {
        // less than
        let mut computer = setup_computer(vec![1107, 0, 1, 4, 0]);
        let status = computer
            .perform_operation(Operation::LessThan(
                ParameterMode::Immediate,
                ParameterMode::Immediate,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![1107, 0, 1, 4, 1], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);

        // not less than
        let mut computer = setup_computer(vec![1107, 1, 0, 4, 1]);
        let status = computer
            .perform_operation(Operation::LessThan(
                ParameterMode::Immediate,
                ParameterMode::Immediate,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![1107, 1, 0, 4, 0], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_evaluate_equal_to() {
        // equal to
        let mut computer = setup_computer(vec![8, 4, 5, 6, 0, 0, 0]);
        let status = computer
            .perform_operation(Operation::EqualTo(
                ParameterMode::Position,
                ParameterMode::Position,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![8, 4, 5, 6, 0, 0, 1], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);

        // not equal to
        let mut computer = setup_computer(vec![8, 4, 5, 6, 1, 0, 1]);
        let status = computer
            .perform_operation(Operation::EqualTo(
                ParameterMode::Position,
                ParameterMode::Position,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![8, 4, 5, 6, 1, 0, 0], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_evaluate_equal_to_in_immediate_mode() {
        // equal to
        let mut computer = setup_computer(vec![1108, 0, 0, 4, 0]);
        let status = computer
            .perform_operation(Operation::EqualTo(
                ParameterMode::Immediate,
                ParameterMode::Immediate,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![1108, 0, 0, 4, 1], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);

        // not equal to
        let mut computer = setup_computer(vec![1108, 1, 0, 4, 1]);
        let status = computer
            .perform_operation(Operation::EqualTo(
                ParameterMode::Immediate,
                ParameterMode::Immediate,
            ))
            .expect("Failed to execute operation");
        assert_eq!(vec![1108, 1, 0, 4, 0], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionStatus::Ongoing, status);
    }

    #[test]
    fn it_should_terminate_a_program() {
        let mut computer = setup_computer(vec![99]);
        let status = computer
            .perform_operation(Operation::Terminate)
            .expect("Failed to execute operation");
        assert_eq!(vec![99], computer.memory);
        assert_eq!(1, computer.program_counter);
        assert_eq!(ExecutionStatus::Terminated, status);
    }

    #[test]
    fn it_should_fail_for_indexing_outside_of_a_program() {
        let mut computer = setup_computer(vec![1, 5, 2, 3]);
        let failure = computer
            .perform_operation(Operation::Add(
                ParameterMode::Position,
                ParameterMode::Immediate,
            ))
            .expect_err("Failed to fail operation");
        assert_eq!(vec![1, 5, 2, 3], computer.memory);
        assert_eq!(0, computer.program_counter);
        assert_eq!(
            ExecutionError::IndexOutsideOfProgram {
                index: 5,
                program_length: 4
            },
            failure
        );

        let mut computer = setup_computer(vec![1, -5, 2, 3]);
        let failure = computer
            .perform_operation(Operation::Add(
                ParameterMode::Position,
                ParameterMode::Immediate,
            ))
            .expect_err("Failed to fail operation");
        assert_eq!(vec![1, -5, 2, 3], computer.memory);
        assert_eq!(0, computer.program_counter);
        assert_eq!(
            ExecutionError::IndexOutsideOfProgram {
                index: -5,
                program_length: 4
            },
            failure
        );
    }

    #[test]
    fn it_should_fail_for_invalid_operation_index() {
        let mut computer = setup_computer(vec![1, 0, 0, 3]);
        let failure = computer.execute().expect_err("Failed to fail operation");
        assert_eq!(vec![1, 0, 0, 2], computer.memory);
        assert_eq!(4, computer.program_counter);
        assert_eq!(ExecutionError::InvalidOperationIndex { index: 4 }, failure);
    }

    #[test]
    fn it_should_fail_for_invalid_request_for_input() {
        let mut computer = IntcodeComputer {
            program_counter: 0,
            memory: vec![3, 3, 0, 0],
            input: VecDeque::new(),
            output: VecDeque::new(),
        };
        let failure = computer
            .perform_operation(Operation::StoreInput)
            .expect_err("Failed to execute operation");
        assert_eq!(vec![3, 3, 0, 0], computer.memory);
        assert_eq!(VecDeque::new(), computer.input);
        assert_eq!(0, computer.program_counter);
        assert_eq!(ExecutionError::InvalidRequestForInput { index: 0 }, failure);
    }

    #[test]
    fn it_should_fail_for_negative_operation_codes() {
        let mut computer = setup_computer(vec![-1, 0, 0, 3]);
        let failure = computer.execute().expect_err("Failed to fail operation");
        assert_eq!(vec![-1, 0, 0, 3], computer.memory);
        assert_eq!(0, computer.program_counter);
        assert_eq!(
            ExecutionError::InvalidOperationCode { index: 0, code: -1 },
            failure
        );
    }

    #[test]
    fn it_should_execute_an_entire_program() {
        let mut computer = setup_computer(vec![1, 0, 0, 3, 4, 3, 99]);
        let output = computer.execute().expect("Failed to execute program");
        assert_eq!(VecDeque::from(vec![2]), output);
    }

    #[test]
    fn it_should_test_the_puzzle_examples() {
        // Here are several programs that take one input, compare it to the value 8, and then produce one output
        /*
                For example, here are several programs that take one input, compare it to the value 8, and then produce one output:

                3,9,8,9,10,9,4,9,99,-1,8 - Using position mode, consider whether the input is equal to 8; output 1 (if it is) or 0 (if it is not).
                3,9,7,9,10,9,4,9,99,-1,8 - Using position mode, consider whether the input is less than 8; output 1 (if it is) or 0 (if it is not).
                3,3,1108,-1,8,3,4,3,99 - Using immediate mode, consider whether the input is equal to 8; output 1 (if it is) or 0 (if it is not).
                3,3,1107,-1,8,3,4,3,99 - Using immediate mode, consider whether the input is less than 8; output 1 (if it is) or 0 (if it is not).

                Here are some jump tests that take an input, then output 0 if the input was zero or 1 if the input was non-zero:

                3,12,6,12,15,1,13,14,13,4,13,99,-1,0,1,9 (using position mode)
                3,3,1105,-1,9,1101,0,0,12,4,12,99,1 (using immediate mode)
                Here's a larger example:
        */

        // The program will then output 999 if the input value is below 8, output 1000 if the input value is equal to 8, or output 1001 if the input value is greater than 8.
        let mut computer = IntcodeComputer::new_with_input(
            vec![
                3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36,
                98, 0, 0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000,
                1, 20, 4, 20, 1105, 1, 46, 98, 99,
            ],
            VecDeque::from(vec![7]),
        );
        let output = computer.execute().expect("Failed to execute program");
        assert_eq!(VecDeque::from(vec![999]), output);

        let mut computer = IntcodeComputer::new_with_input(
            vec![
                3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36,
                98, 0, 0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000,
                1, 20, 4, 20, 1105, 1, 46, 98, 99,
            ],
            VecDeque::from(vec![8]),
        );
        let output = computer.execute().expect("Failed to execute program");
        assert_eq!(VecDeque::from(vec![1000]), output);

        let mut computer = IntcodeComputer::new_with_input(
            vec![
                3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31, 1106, 0, 36,
                98, 0, 0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104, 999, 1105, 1, 46, 1101, 1000,
                1, 20, 4, 20, 1105, 1, 46, 98, 99,
            ],
            VecDeque::from(vec![9]),
        );
        let output = computer.execute().expect("Failed to execute program");
        assert_eq!(VecDeque::from(vec![1001]), output);
    }
}
