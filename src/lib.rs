use std::{fmt::Display, io::Write, path::Path, time::Duration};

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
        #[arg(short, long, default_value_t = false, conflicts_with = "example")]
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
    SubmissionResponse,
    InputError(String),
    Download(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoHome => write!(f, "could not find HOME"),
            Error::NoSession => write!(
                f,
                "could not find session cookie\n\
                hint: set the AOC_SESSION env var, or put the cookie in ~/.adventofcode.session"
            ),
            Error::Http(e) => write!(f, "http: {}", format!("{e}")),
            Error::NoSolution => write!(f, "did not solve any part"),
            Error::SubmissionResponse => write!(f, "unable to extract submission response message"),
            Error::InputError(e) => write!(f, "{e}"),
            Error::Download(e) => write!(f, "unable to get input: {e}"),
        }
    }
}

#[macro_export]
macro_rules! solution {
    ($year:expr, $day:expr) => {
        fn main() {
            match aoc_rust::run($year, $day, part_one, part_two) {
                Ok(()) => {}
                Err(e) => eprintln!("error: {e}"),
            };
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

fn print_result(
    year: u16,
    day: u8,
    p1: Option<&(String, Duration)>,
    p2: Option<&(String, Duration)>,
) {
    println!("{year} Day {day}:");
    let p1res = if let Some((ans, dur)) = p1 {
        format!("{:<20} {:.2?}", &ans, dur)
    } else {
        "unsolved".to_string()
    };
    println!("  Part 1: {p1res}");
    let p2res = if let Some((ans, dur)) = p2 {
        format!("{:<20} {:.2?}", &ans, dur)
    } else {
        "unsolved".to_string()
    };
    println!("  Part 2: {p2res}");
}

pub fn run<F1, F2, T1, T2>(year: u16, day: u8, p1: F1, p2: F2) -> Result<(), Error>
where
    F1: Fn(&str) -> Option<T1>,
    T1: Display,
    F2: Fn(&str) -> Option<T2>,
    T2: Display,
{
    let cli = Cli::parse();

    match cli.command {
        Command::Run {
            submit,
            bench,
            example,
        } => {
            let input_path = if !example {
                Path::new(INPUTS_PATH).join(format!("day{day:02}.txt"))
            } else {
                Path::new(EXAMPLES_PATH).join(format!("day{day:02}.txt"))
            };
            let input = get_input(&input_path)?;

            let p1res = run_part(p1, &input);
            let p2res = run_part(p2, &input);

            print_result(year, day, p1res.as_ref(), p2res.as_ref());

            if submit {
                if let Some((ref ans, _)) = p2res {
                    crate::submit(year, day, 1, &ans.to_string())?;
                } else if let Some((ref ans, _)) = p1res {
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
    std::fs::read_to_string(path)
        .map_err(|e| Error::InputError(format!("{e} \"{}\"", path.display())))
}

fn download(year: u16, day: u8) -> Result<(), Error> {
    let path = Path::new(INPUTS_PATH).join(format!("day{day:02}.txt"));
    let input = fetch_input(year, day)?;
    std::fs::write(path, input).map_err(|e| Error::Download(format!("{e}")))?;
    Ok(())
}

fn submit(year: u16, day: u8, part: u8, ans: &str) -> Result<(), Error> {
    println!("\nSubmitting {ans} for {year} day {day} part {part}... ");
    std::io::stdout().flush().unwrap();
    let response_msg = post_answer(year, day, part, ans)?;
    println!("{response_msg}");
    Ok(())
}

fn post_answer(year: u16, day: u8, part: u8, ans: &str) -> Result<String, Error> {
    let text = client()?
        .post(format!("{AOC_BASE_URL}/{year}/day/{day}/answer"))
        .form(&[("level", part.to_string()), ("answer", ans.to_string())])
        .send()
        .map_err(|e| Error::Http(e))?
        .error_for_status()
        .map_err(|e| Error::Http(e))?
        .text()
        .map_err(|e| Error::Http(e))?;

    let msg = text
        .split_once("<article>")
        .and_then(|(_, rest)| rest.split_once("</article>"))
        .map(|(article, _)| strip_tags(article))
        .ok_or(Error::SubmissionResponse)?;

    Ok(msg.to_owned())
}

fn strip_tags(html: &str) -> String {
    let mut stripped = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => stripped.push(c),
            _ => {}
        }
    }
    stripped.trim().to_owned()
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
    headers.insert(COOKIE, cookie);
    headers.insert(USER_AGENT, user_agent);

    Ok(Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(5))
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
    Ok("github.com/zaperfish/aoc-rust @ unknown".to_owned())
}
