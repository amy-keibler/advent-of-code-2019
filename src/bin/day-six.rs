use anyhow::anyhow;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while1;
use nom::combinator::all_consuming;
use nom::error::{ParseError, VerboseError};
use nom::multi::separated_list;
use nom::IResult;

use std::collections::{HashMap, HashSet, VecDeque};

fn main() -> Result<(), terminator::Terminator> {
    let puzzle_input = include_str!("../../data/day-six-input.txt");
    let orbits = parse(puzzle_input)?;

    let start = "YOU";
    let end = "SAN";

    println!(
        "Minimum transfers from {} to {} is {}",
        start,
        end,
        minimum_transfers(orbits, start, end)
    );
    Ok(())
}

#[derive(Debug, PartialEq)]
struct Orbit<'a> {
    orbited: &'a str,
    orbiting: &'a str,
}

fn parse(input: &str) -> Result<Vec<Orbit>, anyhow::Error> {
    parse_orbits(input)
        .map(|(_, o)| o)
        .map_err(|e: nom::Err<VerboseError<&str>>| anyhow!("{:?}", e))
}

fn parse_orbits<'a, E: ParseError<&'a str>>(orbits: &'a str) -> IResult<&'a str, Vec<Orbit>, E> {
    all_consuming(separated_list(tag("\n"), parse_orbit))(&orbits)
}

fn parse_orbit<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Orbit, E> {
    let (input, orbited) = parse_planet(input)?;
    let (input, _) = tag(")")(input)?;
    let (input, orbiting) = parse_planet(input)?;

    Ok((input, Orbit { orbited, orbiting }))
}

fn parse_planet<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    let (input, name) = take_while1(char::is_alphanumeric)(input)?;

    Ok((input, name))
}

fn orbit_checksum(orbits: Vec<Orbit>) -> u32 {
    let orbit_graph: HashMap<&str, &str> = orbits
        .into_iter()
        .map(|o| (o.orbiting, o.orbited))
        .collect();

    let mut checksum = 0;

    for orbiting in orbit_graph.keys() {
        let mut visited_planets = HashSet::<&str>::new();
        visited_planets.insert(orbiting);
        let mut current_key = orbit_graph.get(orbiting);
        while let Some(key) = current_key {
            if visited_planets.contains(key) {
                break;
            } else {
                visited_planets.insert(key);
                current_key = orbit_graph.get(key);
                checksum += 1;
            }
        }
    }

    checksum
}

fn build_orbit_chain<'a>(orbits: &Vec<Orbit<'a>>, start: &'a str) -> VecDeque<&'a str> {
    let orbit_graph: HashMap<&str, &str> = orbits
        .into_iter()
        .map(|o| (o.orbiting, o.orbited))
        .collect();

    let mut current_chain: VecDeque<&'a str> = VecDeque::new();

    build_chain(&orbit_graph, &mut current_chain, start);

    current_chain
}

/// Currently cannot handle cycles
fn build_chain<'a>(
    orbit_graph: &HashMap<&'a str, &'a str>,
    current_chain: &mut VecDeque<&'a str>,
    planet: &'a str,
) {
    if let Some(orbited) = orbit_graph.get(planet) {
        current_chain.push_front(orbited);
        build_chain(orbit_graph, current_chain, orbited);
    }
}

fn minimum_transfers(orbits: Vec<Orbit>, start: &str, end: &str) -> u32 {
    let mut chain_from_start = build_orbit_chain(&orbits, start);
    let mut chain_from_end = build_orbit_chain(&orbits, end);
    remove_common_prefix(&mut chain_from_start, &mut chain_from_end);

    (chain_from_start.len() + chain_from_end.len()) as u32
}

fn remove_common_prefix<'a>(
    chain_from_start: &mut VecDeque<&'a str>,
    chain_from_end: &mut VecDeque<&'a str>,
) {
    if chain_from_start.front() == chain_from_end.front() {
        chain_from_start.pop_front();
        chain_from_end.pop_front();
        remove_common_prefix(chain_from_start, chain_from_end);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_parse_orbits() {
        let input = r#"COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L"#;
        let output = parse(input).expect("Failed to parse orbits");
        assert_eq!(
            vec![
                Orbit {
                    orbited: "COM",
                    orbiting: "B"
                },
                Orbit {
                    orbited: "B",
                    orbiting: "C"
                },
                Orbit {
                    orbited: "C",
                    orbiting: "D"
                },
                Orbit {
                    orbited: "D",
                    orbiting: "E"
                },
                Orbit {
                    orbited: "E",
                    orbiting: "F"
                },
                Orbit {
                    orbited: "B",
                    orbiting: "G"
                },
                Orbit {
                    orbited: "G",
                    orbiting: "H"
                },
                Orbit {
                    orbited: "D",
                    orbiting: "I"
                },
                Orbit {
                    orbited: "E",
                    orbiting: "J"
                },
                Orbit {
                    orbited: "J",
                    orbiting: "K"
                },
                Orbit {
                    orbited: "K",
                    orbiting: "L"
                },
            ],
            output
        );
        assert_eq!(
            vec![Orbit {
                orbited: "123",
                orbiting: "456"
            },],
            parse("123)456").expect("Failed to parse orbits")
        );
    }

    #[test]
    fn it_fails_to_parse_invalid_input() {
        assert_eq!(true, dbg!(parse("A(B")).is_err());
        assert_eq!(true, dbg!(parse("ABC")).is_err());
        assert_eq!(true, dbg!(parse("ABC)")).is_err());
    }

    #[test]
    fn it_can_checksum_orbits() {
        assert_eq!(
            42,
            orbit_checksum(vec![
                Orbit {
                    orbited: "COM",
                    orbiting: "B"
                },
                Orbit {
                    orbited: "B",
                    orbiting: "C"
                },
                Orbit {
                    orbited: "C",
                    orbiting: "D"
                },
                Orbit {
                    orbited: "D",
                    orbiting: "E"
                },
                Orbit {
                    orbited: "E",
                    orbiting: "F"
                },
                Orbit {
                    orbited: "B",
                    orbiting: "G"
                },
                Orbit {
                    orbited: "G",
                    orbiting: "H"
                },
                Orbit {
                    orbited: "D",
                    orbiting: "I"
                },
                Orbit {
                    orbited: "E",
                    orbiting: "J"
                },
                Orbit {
                    orbited: "J",
                    orbiting: "K"
                },
                Orbit {
                    orbited: "K",
                    orbiting: "L"
                },
            ])
        );
    }

    #[test]
    fn it_can_handle_cycles() {
        assert_eq!(
            6,
            orbit_checksum(vec![
                Orbit {
                    orbited: "A",
                    orbiting: "B"
                },
                Orbit {
                    orbited: "B",
                    orbiting: "C"
                },
                Orbit {
                    orbited: "C",
                    orbiting: "A"
                },
            ])
        );

        assert_eq!(
            6,
            orbit_checksum(vec![
                Orbit {
                    orbited: "Start",
                    orbiting: "A"
                },
                Orbit {
                    orbited: "A",
                    orbiting: "B"
                },
                Orbit {
                    orbited: "B",
                    orbiting: "C"
                },
                Orbit {
                    orbited: "C",
                    orbiting: "A"
                },
            ])
        );
    }

    #[test]
    fn it_can_build_an_orbit_chain() {
        assert_eq!(
            VecDeque::from(vec!["C", "B", "A"]),
            build_orbit_chain(
                &vec![
                    Orbit {
                        orbiting: "Start",
                        orbited: "A"
                    },
                    Orbit {
                        orbiting: "A",
                        orbited: "B"
                    },
                    Orbit {
                        orbiting: "B",
                        orbited: "C"
                    }
                ],
                "Start"
            )
        );
    }

    #[test]
    fn it_can_find_the_minimum_transfers() {
        let input = r#"COM)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L
K)YOU
I)SAN"#;
        let output = parse(input).expect("Failed to parse orbits");
        assert_eq!(4, minimum_transfers(output, "YOU", "SAN"));
    }
}
