# 🎄 aoc-rust
 
A tiny, dependency free helper for solving [Advent of Code](https://adventofcode.com) in Rust.

It doesn’t scaffold anything for you: there are no generated files, templates or project layout to adopt. Scaffolding is easy to do with a small script, so the crate leaves it to you. You create your day files yourself, and the crate only adds the convenience around them: running and timing both parts, downloading inputs and submitting answers.

You write part_one and part_two, and one macro does the rest.

Running a day looks like this:
```
2015 Day 01
  Part 1: 232                  14.21µs
  Part 2: unsolved
```

## Usage

Add it to the project you solve puzzles in:
And make sure the aoc-cli is installed:
 
```sh
cargo add aoc-rust
cargo install aoc-cli --version 0.12.0
```

Everything you need is in two macros:

- **`solution!(year, day)`**\
  Turns a file into a runnable day. It generates `main`, which reads the input,
  runs and times `part_one` and `part_two`, prints the answers, and handles the
  `run` and `download` commands.

- **`debug!(...)`**\
  Works like `eprintln!`, but is removed from release builds. Run without
  `--release` to see the output.

Each day is its own binary:
 
```rust
// src/bin/day01.rs
aoc_rust::solution!(2015, 1);
 
fn part_one(input: &str) -> Option<u64> {
    debug!("Some debug output");
    Some(input.lines().count() as u64)
}
 
fn part_two(_input: &str) -> Option<u64> {
    None // not solved yet
}
```

Answers can be any type that implements `Display`, so days with text answers
can return `Option<String>`.

## Commands

```
$ cargo run --release --bin day01 -- run --submit
```

```
Usage: day01 <COMMAND>
 
Commands:
  run       [--submit] [--example] Run both parts and print the answers
  download                         Download the puzzle input for the day
```

## Files
 
Inputs live next to your `Cargo.toml`:
 
```
data/
├── inputs/day01.txt     # your puzzle input (downloaded)
├── examples/day01.txt   # the example from the puzzle text (paste it yourself)
└── puzzles/day01.md     # the puzzle as md file (downloaded)
```

Make sure the directories exist before you start, since the crate doesn't create them for you.

```sh
mkdir -p data/inputs data/examples data/puzzles
```
