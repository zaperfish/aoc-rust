use std::env;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Macro Exports
// ---------------------------------------------------------------------------

/// Generates `main` for a day. Expects 'part_one' and 'part_two' functions.
#[macro_export]
macro_rules! solution {
    ($year:expr, $day:expr) => {
        use $crate::debug;

        fn main() {
            match $crate::run($year, $day, part_one, part_two) {
                Err(e) => eprintln!("error: {e}"),
                Ok(()) => {}
            };
        }
    };
}

/// 'eprintln!' which does not get compiled into release builds.
#[macro_export]
macro_rules! debug {
    ($($t:tt)*) => {{
        #[cfg(debug_assertions)]
        {
            eprintln!($($t)*);
        }
    }};
}

// ---------------------------------------------------------------------------
// EntryPoint
// ---------------------------------------------------------------------------

pub fn run<F1, F2, T1, T2>(year: u16, day: u8, part_one: F1, part_two: F2) -> Result<(), String>
where
    F1: Fn(&str) -> Option<T1>,
    T1: Display,
    F2: Fn(&str) -> Option<T2>,
    T2: Display,
{
    Cli::parse().command.execute(year, day, part_one, part_two)
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

struct Cli {
    command: CliCommand,
}

enum CliCommand {
    Run { submit: bool, example: bool },
    Download {},
}

const HELP: &str = "\
Usage: cargo run [--release] --bin <DAY> -- <COMMAND> [OPTIONS]

Commands:
  run            Run both parts and print the answers
    --submit     Submit the answer of the highest solved part
    --example    Use the example input instead of your puzzle input
  download       Download the puzzle input for the day";

impl Cli {
    fn parse() -> Self {
        match Self::try_parse() {
            Ok(cli) => cli,
            Err(e) => {
                println!("{e}");
                std::process::exit(2);
            }
        }
    }

    fn try_parse() -> Result<Self, String> {
        let mut args = env::args().skip(1);

        let command = match args.next().as_deref() {
            Some("run") => {
                let mut submit = false;
                let mut example = false;

                if args.any(|a| a == "--submit") {
                    submit = true;
                }
                if args.any(|a| a == "--example") {
                    example = true;
                }

                if submit && example {
                    return Err("--submit can't be used with --example".into());
                }

                CliCommand::Run { submit, example }
            }
            Some("download") => CliCommand::Download {},
            Some(_) => {
                print!("{HELP}");
                return Err("".into());
            }
            None => {
                print!("{HELP}");
                return Err("".into());
            }
        };

        Ok(Self { command })
    }
}

impl CliCommand {
    fn execute<F1, F2, T1, T2>(self, year: u16, day: u8, p1: F1, p2: F2) -> Result<(), String>
    where
        F1: Fn(&str) -> Option<T1>,
        T1: Display,
        F2: Fn(&str) -> Option<T2>,
        T2: Display,
    {
        match self {
            CliCommand::Run { submit, example } => {
                let input = get_input(day, example)?;
                let answers = Answers::solve(&input, &(p1, p2));
                answers.print(year, day);

                if submit {
                    let (part, answer) = answers.highest().ok_or("nothing to submit")?;
                    crate::submit(year, day, part, &answer)?;
                }
            }
            CliCommand::Download {} => download(year, day)?,
        }
        Ok(())
    }
}

fn download(year: u16, day: u8) -> Result<(), String> {
    let output = run_aoc_cli_output(
        Command::new("aoc")
            .args(["download", "--quiet", "--overwrite"])
            .args(["--year", &year.to_string()])
            .args(["--day", &day.to_string()])
            .args(["--input-only"])
            .args(["--input-file", &input_path(day).display().to_string()]),
    )?;

    if output.stderr.is_empty() {
        println!("Successfully downloaded input for day {day}");
    } else {
        return Err(format!(
            "download failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(())
}

fn submit(year: u16, day: u8, part: u8, answer: &str) -> Result<(), String> {
    println!("\nSubmitting {answer} for part {part}:");
    let _output = run_aoc_cli_output(
        Command::new("aoc")
            .args(["submit", "--quiet"])
            .args(["--year", &year.to_string()])
            .args(["--day", &day.to_string()])
            .arg(part.to_string())
            .arg(answer),
    )?;
    println!("{:?}", _output);
    Ok(())
}

fn run_aoc_cli_output(cmd: &mut Command) -> Result<Output, String> {
    let output = cmd.output().map_err(|e| {
        format!(
            "couldn't spawn aoc-cli (is it installed? \
            try `cargo install aoc-cli --version 0.12.0`): {e}"
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("aoc-cli exited with {}: {}", output.status, stderr));
    }

    Ok(output)
}

fn input_path(day: u8) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("data/inputs/day{day:02}.txt"))
}

fn example_path(day: u8) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("data/examples/day{day:02}.txt"))
}

fn get_input(day: u8, example: bool) -> Result<String, String> {
    std::fs::read_to_string(if !example {
        input_path(day)
    } else {
        example_path(day)
    })
    .map_err(|e| format!("failed to read from file: {e}"))
}

// ---------------------------------------------------------------------------
// Puzzle Solving
// ---------------------------------------------------------------------------

struct Answer {
    value: String,
    time: Duration,
}

struct Answers {
    part_one: Option<Answer>,
    part_two: Option<Answer>,
}

pub trait Solution {
    fn part_one(&self, input: &str) -> Option<String>;
    fn part_two(&self, input: &str) -> Option<String>;
}

impl Answers {
    fn solve<S>(input: &str, solution: &S) -> Self
    where
        S: Solution,
    {
        Self {
            part_one: timed(|| solution.part_one(input)),
            part_two: timed(|| solution.part_two(input)),
        }
    }

    fn highest(&self) -> Option<(u8, String)> {
        match self {
            Self {
                part_two: Some(Answer { value, .. }),
                ..
            } => Some((2, value.to_owned())),
            Self {
                part_one: Some(Answer { value, .. }),
                part_two: None,
            } => Some((1, value.to_owned())),
            Self {
                part_one: None,
                part_two: None,
            } => None,
        }
    }

    fn print(&self, year: u16, day: u8) {
        println!("{year} Day {day}:");
        match self.part_one {
            Some(ref a) => {
                println!("  Part 1: {:<20} {:?}", a.value, a.time);
            }
            None => {
                println!("  Part 1: unsolved");
            }
        };
        match self.part_two {
            Some(ref a) => {
                println!("  Part 2: {:<20} {:?}", a.value, a.time);
            }
            None => {
                println!("  Part 2: unsolved");
            }
        };
    }
}

impl<F1, F2, T1, T2> Solution for (F1, F2)
where
    F1: Fn(&str) -> Option<T1>,
    F2: Fn(&str) -> Option<T2>,
    T1: Display,
    T2: Display,
{
    fn part_one(&self, input: &str) -> Option<String> {
        self.0(input).map(|a| a.to_string())
    }

    fn part_two(&self, input: &str) -> Option<String> {
        self.1(input).map(|a| a.to_string())
    }
}

fn timed<F>(f: F) -> Option<Answer>
where
    F: FnOnce() -> Option<String>,
{
    let start = Instant::now();
    let value = f()?;
    let time = start.elapsed();
    Some(Answer { value, time })
}
