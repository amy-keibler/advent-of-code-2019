fn main() {
    let puzzle_input = include_str!("../../data/day-two-input.txt");
    let program: Vec<u32> = puzzle_input
        .split(',')
        .flat_map(|l| u32::from_str_radix(l, 10).into_iter())
        .collect();

    // reset program
    for i in 0..100 {
        for j in 0..100 {
            let mut program = program.clone();
            program[1] = i;
            program[2] = j;

            if Ok(19_690_720) == execute_program(&mut program) {
                println!("Inputs of {} and {} produce the output", i, j);
                return;
            }
        }
    }
    println!("Failed to find input");
}

#[derive(Debug, PartialEq)]
enum ExecutionError {
    InvalidOperationCode { index: usize, code: u32 },
    IndexOutsideOfProgram { index: usize, program_length: usize },
    IncompleteOperationData { index: usize },
    InvalidOperationIndex { index: usize },
}

fn execute_program(program: &mut Vec<u32>) -> Result<u32, ExecutionError> {
    let mut program_counter: usize = 0;
    loop {
        match program.get(program_counter) {
            Some(1) => {
                perform_operation(program, program_counter, std::ops::Add::add)?;
            }
            Some(2) => {
                perform_operation(program, program_counter, std::ops::Mul::mul)?;
            }
            Some(99) => {
                return program
                    .get(0)
                    .copied()
                    .ok_or(ExecutionError::IndexOutsideOfProgram {
                        index: program_counter,
                        program_length: program.len(),
                    })
            }
            Some(op) => {
                return Err(ExecutionError::InvalidOperationCode {
                    index: program_counter,
                    code: *op,
                })
            }
            None => {
                return Err(ExecutionError::IndexOutsideOfProgram {
                    index: program_counter,
                    program_length: program.len(),
                })
            }
        }
        program_counter += 4;
    }
}

fn perform_operation(
    program: &mut Vec<u32>,
    program_counter: usize,
    operation: impl FnOnce(u32, u32) -> u32,
) -> Result<(), ExecutionError> {
    if program_counter + 3 < program.len() {
        if let (Some(left), Some(right), output) = (
            program.get(program[program_counter + 1] as usize).copied(),
            program.get(program[program_counter + 2] as usize).copied(),
            program[program_counter + 3],
        ) {
            program[output as usize] = operation(left, right)
        } else {
            return Err(ExecutionError::InvalidOperationIndex {
                index: program_counter,
            });
        }
    } else {
        return Err(ExecutionError::IncompleteOperationData {
            index: program_counter,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_handle_an_invalid_operation() {
        let mut program_input = vec![3, 0, 0, 0, 99];
        let program_output = execute_program(&mut program_input);
        assert_eq!(
            (
                vec![3, 0, 0, 0, 99],
                Err(ExecutionError::InvalidOperationCode { index: 0, code: 3 })
            ),
            (program_input, program_output)
        );
    }

    #[test]
    fn it_should_handle_an_invalid_reference() {
        let mut program_input = vec![1, 0, 0, 0];
        let program_output = execute_program(&mut program_input);
        assert_eq!(
            (
                vec![2, 0, 0, 0],
                Err(ExecutionError::IndexOutsideOfProgram {
                    index: 4,
                    program_length: 4
                })
            ),
            (program_input, program_output)
        );
    }

    #[test]
    fn it_should_handle_invalid_data_for_an_operation() {
        let mut program_input = vec![1, 0, 0];
        let program_output = execute_program(&mut program_input);
        assert_eq!(
            (
                vec![1, 0, 0],
                Err(ExecutionError::IncompleteOperationData { index: 0 })
            ),
            (program_input, program_output)
        );
    }

    #[test]
    fn it_should_handle_invalid_indexes_for_an_operation() {
        let mut program_input = vec![1, 5, 0, 0];
        let program_output = execute_program(&mut program_input);
        assert_eq!(
            (
                vec![1, 5, 0, 0],
                Err(ExecutionError::InvalidOperationIndex { index: 0 })
            ),
            (program_input, program_output)
        );
    }

    #[test]
    fn it_should_add() {
        let mut program_input = vec![1, 0, 0, 0, 99];
        let program_output = execute_program(&mut program_input);
        assert_eq!(
            (vec![2, 0, 0, 0, 99], Ok(2)),
            (program_input, program_output)
        );
    }

    #[test]
    fn it_should_multiply() {
        let mut program_input = vec![2, 3, 0, 3, 99];
        let program_output = execute_program(&mut program_input);
        assert_eq!(
            (vec![2, 3, 0, 6, 99], Ok(2)),
            (program_input, program_output)
        );
    }

    #[test]
    fn it_should_handle_complex_cases() {
        let mut program_input = vec![2, 4, 4, 5, 99, 0];
        let program_output = execute_program(&mut program_input);
        assert_eq!(
            (vec![2, 4, 4, 5, 99, 9801], Ok(2)),
            (program_input, program_output)
        );

        let mut program_input = vec![1, 1, 1, 4, 99, 5, 6, 0, 99];
        let program_output = execute_program(&mut program_input);
        assert_eq!(
            (vec![30, 1, 1, 4, 2, 5, 6, 0, 99], Ok(30)),
            (program_input, program_output)
        );

        let mut program_input = vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50];
        let program_output = execute_program(&mut program_input);
        assert_eq!(
            (vec![3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50], Ok(3500)),
            (program_input, program_output)
        );
    }
}
