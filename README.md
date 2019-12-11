# advent-of-code-2019

![Rust test status badge](https://github.com/amy-keibler/advent-of-code-2019/workflows/Rust/badge.svg)

A rust implementation of the [Advent of Code 2019](https://adventofcode.com/2019) puzzle solutions

## Project Layout

Folder         | Purpose
---------------|--------
`src/bin/*.rs` | contains the individual files for each day's problem (usually left in the state required to solve the second part of the puzzle)
`src/*.rs`     | contains the common modules that are used for multiple days' puzzles

## Project Execution

Each day is compiled as a separate binary, so executing day three's puzzle would be `cargo run --bin day-three`. Tests can be run via the usual `cargo test`.
