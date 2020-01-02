use nom::bytes::complete::tag;
use nom::bytes::complete::take_while1;
use nom::character::complete::one_of;
use nom::error::{ParseError, VerboseError};
use nom::multi::separated_list;
use nom::IResult;
use std::collections::{HashMap, HashSet};

fn main() {
    let puzzle_input = include_str!("../../data/day-three-input.txt");
    let mut puzzle_input = puzzle_input.lines();
    let first_wire = puzzle_input.next().expect("Failed to get first wire");
    let second_wire = puzzle_input.next().expect("Failed to get second wire");

    let first_wire = parse(first_wire).expect("Failed to parse first wire");
    let second_wire = parse(second_wire).expect("Failed to parse second wire");

    let position =
        closest_intersection(&first_wire, &second_wire).expect("Failed to get any intersections");
    println!(
        "Got a position of {:?} with a distance of {}",
        position,
        position.distance_from_origin()
    );

    let delay = lowest_delay_of_intersections(&first_wire, &second_wire)
        .expect("Failed to get any intersections");
    println!("Got a delay of {}", delay);
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq)]
struct PathSegment {
    direction: Direction,
    distance: u32,
}

fn parse(path: &str) -> Result<Vec<PathSegment>, String> {
    parse_path(path)
        .map(|(_, p)| p)
        .map_err(|e: nom::Err<VerboseError<&str>>| format!("{:#?}", e))
}

fn parse_path<'a, E: ParseError<&'a str>>(path: &'a str) -> IResult<&'a str, Vec<PathSegment>, E> {
    separated_list(tag(","), parse_path_segment)(&path)
}

fn parse_path_segment<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, PathSegment, E> {
    let (input, direction) = parse_direction(input)?;
    let (input, distance) = parse_distance(input)?;

    Ok((
        input,
        PathSegment {
            direction,
            distance,
        },
    ))
}

fn parse_direction<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Direction, E> {
    let (input, direction) = one_of("UDLR")(input)?;
    let direction = match direction {
        'U' => Direction::Up,
        'D' => Direction::Down,
        'R' => Direction::Right,
        'L' => Direction::Left,
        _ => unreachable!(),
    };
    Ok((input, direction))
}

fn parse_distance<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, u32, E> {
    let (input, distance) = take_while1(|i| char::is_digit(i, 10))(input)?;

    Ok((
        input,
        distance.parse::<u32>().unwrap_or_else(|_| {
            panic!(
                "Should have been able to get a value from all digits {}",
                distance
            )
        }),
    ))
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Position(i32, i32);

impl Position {
    fn distance_from_origin(self) -> u32 {
        (i32::abs(self.0) + i32::abs(self.1)) as u32
    }

    fn adjacent_position(self, direction: Direction) -> Position {
        match direction {
            Direction::Up => Position(self.0, self.1 + 1),
            Direction::Down => Position(self.0, self.1 - 1),
            Direction::Left => Position(self.0 - 1, self.1),
            Direction::Right => Position(self.0 + 1, self.1),
        }
    }
}

fn closest_intersection(
    first_wire: &[PathSegment],
    second_wire: &[PathSegment],
) -> Option<Position> {
    let first_positions = wire_to_positions(first_wire);
    let second_positions = wire_to_positions(second_wire);

    first_positions
        .keys()
        .cloned()
        .collect::<HashSet<Position>>()
        .intersection(
            &second_positions
                .keys()
                .cloned()
                .collect::<HashSet<Position>>(),
        )
        .min_by_key(|p| p.distance_from_origin())
        .copied()
}

fn lowest_delay_of_intersections(
    first_wire: &[PathSegment],
    second_wire: &[PathSegment],
) -> Option<u32> {
    let first_positions = wire_to_positions(first_wire);
    let second_positions = wire_to_positions(second_wire);

    first_positions
        .keys()
        .cloned()
        .collect::<HashSet<Position>>()
        .intersection(
            &second_positions
                .keys()
                .cloned()
                .collect::<HashSet<Position>>(),
        )
        .flat_map(
            |p| match (first_positions.get(p), second_positions.get(p)) {
                (Some(first_steps), Some(second_steps)) => Some(first_steps + second_steps),
                _ => None,
            },
        )
        .min()
}

/// assumes an initial position of <0, 0>
fn wire_to_positions(wire: &[PathSegment]) -> HashMap<Position, u32> {
    let mut positions = HashMap::new();
    let mut current_position = Position(0, 0);
    let mut current_step = 0;
    for wire_segment in wire {
        for _ in 0..wire_segment.distance {
            current_position = current_position.adjacent_position(wire_segment.direction);
            current_step += 1;
            positions.entry(current_position).or_insert(current_step);
        }
    }
    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_parse_a_simple_path() {
        assert_eq!(
            Ok(vec![
                PathSegment {
                    direction: Direction::Up,
                    distance: 2,
                },
                PathSegment {
                    direction: Direction::Right,
                    distance: 2,
                },
            ]),
            parse("U2,R2")
        );
    }

    #[test]
    fn wires_that_do_not_overlap_should_not_have_a_closest_position() {
        assert_eq!(None, closest_intersection(&Vec::new(), &Vec::new()));
    }

    #[test]
    fn a_single_simple_intersection_should_return_a_position() {
        let first_wire = vec![
            PathSegment {
                direction: Direction::Up,
                distance: 2,
            },
            PathSegment {
                direction: Direction::Right,
                distance: 2,
            },
        ];
        let second_wire = vec![
            PathSegment {
                direction: Direction::Right,
                distance: 2,
            },
            PathSegment {
                direction: Direction::Up,
                distance: 2,
            },
        ];
        assert_eq!(
            Some(Position(2, 2)),
            closest_intersection(&first_wire, &second_wire)
        );
    }

    #[test]
    fn it_should_handle_complex_cases() {
        let first_wire =
            parse("R75,D30,R83,U83,L12,D49,R71,U7,L72").expect("Failed to parse first wire");
        let second_wire =
            parse("U62,R66,U55,R34,D71,R55,D58,R83").expect("Failed to parse second wire");
        assert_eq!(
            159,
            closest_intersection(&first_wire, &second_wire)
                .unwrap()
                .distance_from_origin()
        );

        let first_wire = parse("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51")
            .expect("Failed to parse first wire");
        let second_wire =
            parse("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7").expect("Failed to parse first wire");
        assert_eq!(
            135,
            closest_intersection(&first_wire, &second_wire)
                .unwrap()
                .distance_from_origin()
        );
    }

    #[test]
    fn it_should_handle_complex_signal_delay_cases() {
        let first_wire =
            parse("R75,D30,R83,U83,L12,D49,R71,U7,L72").expect("Failed to parse first wire");
        let second_wire =
            parse("U62,R66,U55,R34,D71,R55,D58,R83").expect("Failed to parse second wire");
        assert_eq!(
            610,
            lowest_delay_of_intersections(&first_wire, &second_wire).unwrap()
        );

        let first_wire = parse("R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51")
            .expect("Failed to parse first wire");
        let second_wire =
            parse("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7").expect("Failed to parse first wire");
        assert_eq!(
            410,
            lowest_delay_of_intersections(&first_wire, &second_wire).unwrap()
        );
    }
}
