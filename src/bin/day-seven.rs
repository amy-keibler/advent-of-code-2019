use anyhow::anyhow;

use std::collections::VecDeque;

use advent_of_code::intcode_computer::{ExecutionError, IntcodeComputer};
use advent_of_code::permutations::PermutationsIterator;

fn main() {
    let puzzle_input = include_str!("../../data/day-seven-input.txt");
    let program: Vec<i32> = puzzle_input
        .split(',')
        .flat_map(|l| i32::from_str_radix(l, 10).into_iter())
        .collect();
    let (value, setting) =
        maximize_amplifier_output(program).expect("Failed to execute phase settings");
    println!("Got {}, for setting {:?}", value, setting);
}

fn maximize_amplifier_output(program: Vec<i32>) -> Result<(i32, [PhaseSetting; 5]), anyhow::Error> {
    PermutationsIterator::from(vec![
        PhaseSetting::Zero,
        PhaseSetting::One,
        PhaseSetting::Two,
        PhaseSetting::Three,
        PhaseSetting::Four,
    ])
    .map(|p| [p[0], p[1], p[2], p[3], p[4]])
    .map(|p| (evaluate_sequence_for_program(program.clone(), p), p))
    .filter_map(|(value, p)| {
        if let Ok(value) = value {
            Some((value, p))
        } else {
            None
        }
    })
    .max_by_key(|(value, _)| value.clone())
    .ok_or_else(|| anyhow!("Did not get a maximum value"))
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum PhaseSetting {
    Zero,
    One,
    Two,
    Three,
    Four,
}

impl PhaseSetting {
    fn value(&self) -> i32 {
        match self {
            PhaseSetting::Zero => 0,
            PhaseSetting::One => 1,
            PhaseSetting::Two => 2,
            PhaseSetting::Three => 3,
            PhaseSetting::Four => 4,
        }
    }
}

fn evaluate_sequence_for_program(
    program: Vec<i32>,
    phase_sequence: [PhaseSetting; 5],
) -> Result<i32, anyhow::Error> {
    let mut transferred_output = 0;

    for phase_setting in phase_sequence.iter().map(PhaseSetting::value) {
        let mut computer = IntcodeComputer::new_with_input(
            program.clone(),
            VecDeque::from(vec![phase_setting, transferred_output]),
        );
        transferred_output = computer
            .execute()?
            .pop_front()
            .ok_or_else(|| anyhow!("Did not get output from program execution"))?;
    }
    Ok(transferred_output)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_should_calculate_signals_for_inputs() {
        let program = vec![
            3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
        ];
        let phase_sequence = [
            PhaseSetting::Four,
            PhaseSetting::Three,
            PhaseSetting::Two,
            PhaseSetting::One,
            PhaseSetting::Zero,
        ];
        assert_eq!(
            43210,
            evaluate_sequence_for_program(program, phase_sequence)
                .expect("Failed to evaluate sequence")
        );

        let program = vec![
            3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
            99, 0, 0,
        ];
        let phase_sequence = [
            PhaseSetting::Zero,
            PhaseSetting::One,
            PhaseSetting::Two,
            PhaseSetting::Three,
            PhaseSetting::Four,
        ];
        assert_eq!(
            54321,
            evaluate_sequence_for_program(program, phase_sequence)
                .expect("Failed to evaluate sequence")
        );

        let program = vec![
            3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
            33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
        ];
        let phase_sequence = [
            PhaseSetting::One,
            PhaseSetting::Zero,
            PhaseSetting::Four,
            PhaseSetting::Three,
            PhaseSetting::Two,
        ];
        assert_eq!(
            65210,
            evaluate_sequence_for_program(program, phase_sequence)
                .expect("Failed to evaluate sequence")
        );
    }

    #[test]
    fn it_should_maximize_amplifier_output() {
        let program = vec![
            3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
        ];
        let phase_sequence = [
            PhaseSetting::Four,
            PhaseSetting::Three,
            PhaseSetting::Two,
            PhaseSetting::One,
            PhaseSetting::Zero,
        ];
        assert_eq!(
            (43210, phase_sequence),
            maximize_amplifier_output(program).expect("Failed to amplify output")
        );

        let program = vec![
            3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
            99, 0, 0,
        ];
        let phase_sequence = [
            PhaseSetting::Zero,
            PhaseSetting::One,
            PhaseSetting::Two,
            PhaseSetting::Three,
            PhaseSetting::Four,
        ];
        assert_eq!(
            (54321, phase_sequence),
            maximize_amplifier_output(program).expect("Failed to amplify output")
        );

        let program = vec![
            3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
            33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
        ];
        let phase_sequence = [
            PhaseSetting::One,
            PhaseSetting::Zero,
            PhaseSetting::Four,
            PhaseSetting::Three,
            PhaseSetting::Two,
        ];
        assert_eq!(
            (65210, phase_sequence),
            maximize_amplifier_output(program).expect("Failed to amplify output")
        );
    }
}
