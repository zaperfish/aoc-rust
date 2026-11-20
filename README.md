# 🎄 aoc-rust
[![crates.io](https://img.shields.io/crates/v/aoc-rust.svg)](https://crates.io/crates/aoc-rust)
[![license](https://img.shields.io/crates/l/aoc-rust.svg)](LICENSE)

A tiny, dependency free helper for solving [Advent of Code](https://adventofcode.com) in Rust.

It doesn’t scaffold anything for you: there are no generated files, templates or project layout to adopt. Scaffolding is easy to do with a small script, so the crate leaves it to you. You create your day files yourself, and the crate only adds the convenience around them: running and timing both parts, downloading inputs and submitting answers.

You write part_one and part_two, and one macro does the rest.

```rust
// src/bin/day01.rs
aoc_rust::solution!(2025, 1);
 
fn part_one(input: &str) -> Option<u64> { None }
 
fn part_two(_input: &str) -> Option<u64> { None }
```

Answers can be any type that implements `Display`, so days with text answers
can return `Option<String>`.

Running a day looks like this:

```
$ cargo run --release --bin day01 -- run --submit

2025 Day 01
  Part 1: 232                  14.21µs
  Part 2: unsolved
```

## Setup

Add it to the project you solve puzzles in and make sure the aoc-cli is installed:
 
```sh
cargo add aoc-rust
cargo install aoc-cli --version 0.12.0
```
> [!IMPORTANT]
> Make sure your session cookie is stored in `~/.adventofcode.session` or set in the
> `ADVENT_OF_CODE_SESSION` environment variable.

## Usage

Everything you need is in two macros:

- `solution!(year, day)`\
  Turns a file into a runnable day. It generates `main`, which reads the input,
  runs and times `part_one` and `part_two`, prints the answers, and handles the
  `run` and `download` commands.

- `debug!(...)`\
  Works like `eprintln!`, but is removed from release builds. Run without
  `--release` to see the output.

## Commands

```
Usage: cargo run [--release] --bin <DAY> -- <COMMAND> [OPTIONS]

Commands:
  run            Run both parts and print the answers
    --submit     Submit the answer of the highest solved part
    --example    Use the example input instead of your puzzle input
  download       Download the puzzle input for the day";
```

## Files
 
Inputs live next to your `Cargo.toml`:
 
```
my-adventofcode/
├── Cargo.toml
├── src/
│   └── bin/
│       ├── day01.rs       # aoc_rust::solution!(2025, 1);
│       └── day02.rs       # aoc_rust::solution!(2025, 2);
└── data/
    ├── inputs/
    │   ├── day01.txt      # your puzzle input (downloaded)
    │   └── day02.txt
    └── examples/
        ├── day01.txt      # the example from the puzzle text (paste it yourself)
        └── day02.txt
```

Make sure the directories exist before you start, since the crate doesn't create them for you.

```sh
mkdir -p data/inputs data/examples
```

## Hints
- Shorten the commands you type with an alias or a function of your shell.
- TODO: This repo includes small scripts I personally use to scaffold my project.
