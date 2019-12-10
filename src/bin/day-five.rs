use std::collections::VecDeque;

use advent_of_code::intcode_computer::{ExecutionError, IntcodeComputer};

fn main() {
    let puzzle_input = include_str!("../../data/day-five-input.txt");
    let program: Vec<i32> = puzzle_input
        .split(',')
        .flat_map(|l| i32::from_str_radix(l, 10).into_iter())
        .collect();
    let output = run_diagnostic(program, VecDeque::from(vec![5]));
    println!("Result: {:?}", output);
}

#[derive(Debug, PartialEq)]
enum DiagnosticResult {
    EmptyOutput,
    Success { code: i32, output: VecDeque<i32> },
    Failure { code: i32, output: VecDeque<i32> },
    Error(ExecutionError),
}

fn run_diagnostic(program: Vec<i32>, input: VecDeque<i32>) -> DiagnosticResult {
    let mut computer = IntcodeComputer::new_with_input(program, input);
    computer
        .execute()
        .map(|mut output| {
            if let Some(diagnostic_code) = output.pop_back() {
                if output.iter().copied().all(|x| x == 0) {
                    DiagnosticResult::Success {
                        code: diagnostic_code,
                        output,
                    }
                } else {
                    DiagnosticResult::Failure {
                        code: diagnostic_code,
                        output,
                    }
                }
            } else {
                DiagnosticResult::EmptyOutput
            }
        })
        .unwrap_or_else(DiagnosticResult::Error)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_should_perform_a_successful_diagnostic() {
        let program = vec![3, 2, 0, 7, 104, 42, 99, 0];
        assert_eq!(
            DiagnosticResult::Success {
                code: 42,
                output: VecDeque::from(vec![0])
            },
            run_diagnostic(program, VecDeque::from(vec![4]))
        );
    }

    #[test]
    fn it_should_perform_an_unsuccessful_diagnostic() {
        let program = vec![3, 2, 0, 7, 104, 42, 99, 1];
        assert_eq!(
            DiagnosticResult::Failure {
                code: 42,
                output: VecDeque::from(vec![1])
            },
            run_diagnostic(program, VecDeque::from(vec![4]))
        );
    }

    #[test]
    fn it_should_proxy_execution_errors() {
        assert_eq!(
            DiagnosticResult::Error(ExecutionError::IndexOutsideOfProgram {
                index: 3,
                program_length: 3
            }),
            run_diagnostic(vec![1, 0, 0], VecDeque::new())
        );
    }

    #[test]
    fn it_should_handle_empty_output() {
        let program = vec![3, 2, 0, 7, 104, 42, 99, 0];
        assert_eq!(
            DiagnosticResult::EmptyOutput,
            run_diagnostic(program, VecDeque::from(vec![99]))
        );
    }
}
