fn main() {
    let mut num_valid_passwords = 0;
    for puzzle_input in 128_392..=643_281 {
        if matches_complex_criteria(puzzle_input) {
            num_valid_passwords += 1;
        }
    }

    println!("{} valid passwords", num_valid_passwords);
}

fn matches_criteria(num: u32) -> bool {
    let digits = to_digits(num);

    let pairs: Vec<(u8, u8)> = digits
        .iter()
        .copied()
        .zip(digits.iter().copied().skip(1))
        .collect();

    pairs.iter().copied().all(are_monotonically_increasing)
        && pairs.iter().copied().any(are_duplicated)
}

fn matches_complex_criteria(num: u32) -> bool {
    let digits = to_digits(num);

    let pairs: Vec<(u8, u8)> = digits
        .iter()
        .copied()
        .zip(digits.iter().copied().skip(1))
        .collect();

    pairs.iter().copied().all(are_monotonically_increasing) && a_run_of_two_exists(digits)
}

fn to_digits(mut num: u32) -> Vec<u8> {
    let mut digits = Vec::new();

    if num == 0 {
        return vec![0];
    }

    while num > 0 {
        let digit = (num % 10) as u8;
        num /= 10;
        digits.push(digit);
    }

    digits.reverse();
    digits
}

fn are_monotonically_increasing(pair: (u8, u8)) -> bool {
    pair.0 <= pair.1
}

fn are_duplicated(pair: (u8, u8)) -> bool {
    pair.0 == pair.1
}

fn a_run_of_two_exists(digits: Vec<u8>) -> bool {
    let mut digits = digits.iter();
    if let Some(mut current_digit) = digits.next() {
        let mut num_in_run = 1;
        for digit in digits {
            if digit == current_digit {
                num_in_run += 1;
            } else {
                if num_in_run == 2 {
                    return true;
                }
                current_digit = digit;
                num_in_run = 1;
            }
        }
        return num_in_run == 2;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_match_the_criteria_() {
        assert_eq!(true, matches_criteria(111_111));
    }

    #[test]
    fn it_should_not_match_when_digits_decrease() {
        assert_eq!(false, matches_criteria(223_450));
    }

    #[test]
    fn it_should_not_match_when_there_is_not_at_least_two_adjacent_digits() {
        assert_eq!(false, matches_criteria(123_789));
    }

    #[test]
    fn it_should_match_the_complex_criteria() {
        assert_eq!(true, matches_complex_criteria(112_233));
    }

    #[test]
    fn it_should_not_match_when_there_are_no_exactly_two_digit_runs() {
        assert_eq!(false, matches_complex_criteria(123_444));
    }

    #[test]
    fn it_should_match_the_complex_criteria_even_if_there_are_runs_of_three_or_longer() {
        assert_eq!(true, matches_complex_criteria(111_122));
    }
}
