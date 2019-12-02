fn main() {
    let puzzle_input = include_str!("../../data/day-one-input.txt");
    let required_fuel: u32 = puzzle_input
        .lines()
        .flat_map(|l| u32::from_str_radix(l, 10).into_iter())
        .map(fuel_for_module_including_fuel)
        .sum();
    println!("Requires {} fuel", required_fuel);
}

fn fuel_for_module(mass: u32) -> u32 {
    (mass / 3).checked_sub(2).unwrap_or_default()
}

fn fuel_for_module_including_fuel(mass: u32) -> u32 {
    let module_fuel = fuel_for_module(mass);
    if module_fuel > 0 {
        return module_fuel + fuel_for_module_including_fuel(module_fuel);
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_handle_a_simple_example() {
        assert_eq!(2, fuel_for_module(12));
    }

    #[test]
    fn it_should_handle_a_rounding_example() {
        assert_eq!(2, fuel_for_module(14));
    }

    #[test]
    fn it_should_more_realistic_examples() {
        assert_eq!(654, fuel_for_module(1969));
        assert_eq!(33583, fuel_for_module(100_756));
    }

    #[test]
    fn it_should_treat_negative_fuel_as_zero() {
        assert_eq!(0, fuel_for_module(3));
    }

    #[test]
    fn it_should_handle_a_simple_module_plus_fuel_example() {
        assert_eq!(2, fuel_for_module_including_fuel(12));
    }

    #[test]
    fn it_should_more_realistic_module_plus_fuel_examples() {
        assert_eq!(966, fuel_for_module_including_fuel(1969));
        assert_eq!(50346, fuel_for_module_including_fuel(100_756));
    }
}
