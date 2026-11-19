use std::{fmt::Display, path::Path, time::Duration};

use ::reqwest::header::COOKIE;
use clap::{Parser, Subcommand};
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue, USER_AGENT},
};

const AOC_BASE_URL: &str = "https://adventofcode.com";
const INPUTS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/inputs");
const EXAMPLES_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/examples");

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Run {
        #[arg(short, long, default_value_t = false)]
        submit: bool,

        #[arg(long, default_value_t = false)]
        bench: bool,

        #[arg(long, default_value_t = false)]
        example: bool,
    },
    Download {},
}

pub enum Error {
    NoHome,
    NoSession,
    Http(reqwest::Error),
    NoSolution,
}

#[macro_export]
macro_rules! solution {
    ($year:expr, $day:expr) => {
        fn main() {
            aoc_rust::run($year, $day, part_one, part_two);
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($($t:tt)*) => {{
        #[cfg(debug_assertions)]
        {
            eprintln!($($t)*);
        }
    }};
}

pub fn run(
    year: u16,
    day: u8,
    p1: impl Fn(&str) -> Option<u64>,
    p2: impl Fn(&str) -> Option<u64>,
) -> Result<(), Error> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run {
            submit,
            bench,
            example,
        } => {
            let input_path = if !example {
                Path::new(INPUTS_PATH).join(format!("/day{day:02}.txt"))
            } else {
                Path::new(EXAMPLES_PATH).join(format!("/day{day:02}.txt"))
            };
            let input = get_input(&input_path)?;
            let p1res = run_part(p1, &input);
            let p2res = run_part(p2, &input);

            println!(
                "Part 1 : {}",
                if let Some((ref ans, dur)) = p1res {
                    format!("Part 1 : {:<20} {:?}", &ans, dur)
                } else {
                    "unsolved".to_string()
                }
            );

            println!(
                "Part 2 : {}",
                if let Some((ref ans, dur)) = p2res {
                    format!("Part 1 : {:<20} {:?}", &ans, dur)
                } else {
                    "unsolved".to_string()
                }
            );

            if submit {
                if let Some((ans, _)) = p2res {
                    crate::submit(year, day, 1, &ans.to_string())?;
                } else if let Some((ans, _)) = p1res {
                    crate::submit(year, day, 2, &ans.to_string())?;
                } else {
                    return Err(Error::NoSolution);
                }
            }
        }
        Command::Download {} => download(year, day)?,
    }
    Ok(())
}

fn get_input(path: &Path) -> Result<String, Error> {
    Ok("A".to_owned())
}

fn download(year: u16, day: u8) -> Result<(), Error> {
    let input = fetch_input(year, day)?;
    std::fs::write(
        Path::new(INPUTS_PATH).join(format!("day{day:02}.txt")),
        input,
    )
    .map_err(|_| Error::NoSession)?;
    Ok(())
}

fn submit(year: u16, day: u8, part: u8, ans: &str) -> Result<(), Error> {
    let res = client()?
        .post(format!("{AOC_BASE_URL}/{year}/day/{day}/input"))
        .form(&[("level", part.to_string()), ("answer", ans.to_string())])
        .send()
        .map_err(|e| Error::Http(e))?
        .error_for_status()
        .map_err(|e| Error::Http(e))?
        .text()
        .map_err(|e| Error::Http(e))?;

    println!("{:?}", res);
    Ok(())
}

pub fn run_part<F, T>(f: F, input: &str) -> Option<(String, Duration)>
where
    F: Fn(&str) -> Option<T>,
    T: Display,
{
    let start = std::time::Instant::now();
    let res = match f(input) {
        Some(res) => res,
        None => return None,
    };
    let elapsed = start.elapsed();
    Some((res.to_string(), elapsed))
}

fn fetch_input(year: u16, day: u8) -> Result<String, Error> {
    let input = client()?
        .get(format!("{AOC_BASE_URL}/{year}/day/{day}/input"))
        .send()
        .map_err(|e| Error::Http(e))?
        .error_for_status()
        .map_err(|e| Error::Http(e))?
        .text()
        .map_err(|e| Error::Http(e))?;
    Ok(input)
}

fn client() -> Result<Client, Error> {
    let user_agent = HeaderValue::from_str(&get_user_agent()?).unwrap();

    let session = get_session()?;
    let mut cookie = HeaderValue::from_str(&format!("session={}", session.trim())).unwrap();
    cookie.set_sensitive(true);

    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, cookie).unwrap();
    headers.insert(USER_AGENT, user_agent).unwrap();

    Ok(Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(1))
        .https_only(true)
        .build()
        .map_err(|e| Error::Http(e))?)
}

fn get_session() -> Result<String, Error> {
    if let Ok(s) = std::env::var("AOC_SESSION") {
        return Ok(s);
    }
    let home = std::env::var("HOME").map_err(|_| Error::NoHome)?;
    let path = Path::new(&home).join(".adventofcode.session");
    let s = std::fs::read_to_string(&path).map_err(|_| Error::NoSession)?;
    Ok(s.trim().to_owned())
}

fn get_user_agent() -> Result<String, Error> {
    if let Ok(ua) = std::env::var("AOC_USER_AGENT") {
        return Ok(ua);
    }
    Ok("github.com/zaperfish/aocrust @ unknown".to_owned())
}
