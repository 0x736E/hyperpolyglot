use clap::{App, Arg};
use lazy_static::lazy_static;
use regex::Regex;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    io::{self, Write},
    path::PathBuf,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use hyperpolyglot::{get_language_breakdown, get_language_info, Detection, LanguageType};

struct CLIOptions {
    condensed_output: bool,
    filter: Regex,
}

fn main() {
    let matches = get_cli().get_matches();
    let path = matches.value_of("PATH").unwrap();
    let breakdown = get_language_breakdown(path);

    let mut language_count: Vec<(&'static str, Vec<(Detection, PathBuf)>)> = breakdown
        .into_iter()
        .filter(|(language_name, _)| {
            match get_language_info(language_name).map(|l| &l.language_type) {
                Some(LanguageType::Markup) | Some(LanguageType::Programming) => true,
                _ => false,
            }
        })
        .collect();
    language_count.sort_by(|(_, a), (_, b)| b.len().cmp(&a.len()));
    print_language_split(&language_count);

    let cli_options = CLIOptions {
        condensed_output: matches.is_present("condensed"),
        filter: matches
            .value_of("filter")
            .map(|f| Regex::new(f).expect(&format!("Invalid Filter: {}", f)[..]))
            .unwrap_or(Regex::new("").unwrap()),
    };

    if matches.is_present("file-breakdown") {
        println!("");
        if let Err(_) = print_file_breakdown(&language_count, &cli_options) {
            std::process::exit(1);
        }
    }

    if matches.is_present("strategy-breakdown") {
        println!("");
        if let Err(_) = print_strategy_breakdown(&language_count, &cli_options) {
            std::process::exit(1);
        }
    }
}

fn get_cli<'a, 'b>() -> App<'a, 'b> {
    App::new("Hyperpolyglot")
        .version("0.1.0")
        .about("Get the programming language breakdown for a file.")
        .arg(Arg::with_name("PATH").index(1).default_value("."))
        .arg(
            Arg::with_name("file-breakdown")
                .short("b")
                .long("breakdown")
                .help("prints the language detected for each file it visits"),
        )
        .arg(
            Arg::with_name("strategy-breakdown")
                .short("s")
                .long("strategies")
                .help(
                    "Prints each strategy used and what files were determined using that strategy",
                ),
        )
        .arg(
            Arg::with_name("condensed")
                .short("c")
                .long("condensed")
                .help("Condenses the output for the breakdowns to only show the counts"),
        )
        .arg(
            Arg::with_name("filter").short("f").long("filter").help(
                "A regex that is used to filter the output for the file and streategy breakdown",
            ).takes_value(true),
        )
}

fn print_language_split(language_counts: &Vec<(&'static str, Vec<(Detection, PathBuf)>)>) {
    let total = language_counts
        .iter()
        .fold(0, |acc, (_, files)| acc + files.len()) as f64;
    for (language, files) in language_counts.iter() {
        let percentage = ((files.len() * 100) as f64) / total;
        println!("{:.2}% {}", percentage, language);
    }
}

fn print_file_breakdown(
    language_counts: &Vec<(&'static str, Vec<(Detection, PathBuf)>)>,
    options: &CLIOptions,
) -> Result<(), io::Error> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    for (language, breakdowns) in language_counts.iter() {
        if options.filter.is_match(language) {
            stdout.set_color(&TITLE_COLOR)?;
            write!(stdout, "{}", language)?;

            stdout.set_color(&DEFAULT_COLOR)?;
            writeln!(stdout, " ({})", breakdowns.len())?;
            if !options.condensed_output {
                for (_, file) in breakdowns.iter() {
                    let path = strip_relative_parts(file);
                    writeln!(stdout, "{}", path.display())?;
                }
                writeln!(stdout, "")?;
            }
        }
    }
    Ok(())
}

fn print_strategy_breakdown(
    language_counts: &Vec<(&'static str, Vec<(Detection, PathBuf)>)>,
    options: &CLIOptions,
) -> Result<(), io::Error> {
    let mut strategy_breakdown = HashMap::new();
    for (language, files) in language_counts.into_iter() {
        for (detection, file) in files.into_iter() {
            let files = strategy_breakdown
                .entry(detection.variant())
                .or_insert(BinaryHeap::new());
            files.push(Reverse((language, file)));
        }
    }

    let mut strategy_breakdowns: Vec<(String, BinaryHeap<Reverse<(&&str, &PathBuf)>>)> =
        strategy_breakdown.into_iter().collect();
    strategy_breakdowns.sort_by(|(_, a), (_, b)| b.len().cmp(&a.len()));

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    for (strategy, mut breakdowns) in strategy_breakdowns.into_iter() {
        if options.filter.is_match(&strategy[..]) {
            stdout.set_color(&TITLE_COLOR)?;
            write!(stdout, "{}", strategy)?;

            stdout.set_color(&DEFAULT_COLOR)?;
            writeln!(stdout, " ({})", breakdowns.len())?;
            if !options.condensed_output {
                while let Some(Reverse((language, file))) = breakdowns.pop() {
                    stdout.set_color(&DEFAULT_COLOR)?;
                    let path = strip_relative_parts(file);
                    write!(stdout, "{}", path.display())?;

                    stdout.set_color(&LANGUAGE_COLOR)?;
                    writeln!(stdout, " ({})", language)?;
                }
                writeln!(stdout, "")?;
            }
        }
    }
    Ok(())
}

fn strip_relative_parts<'a>(path: &'a PathBuf) -> &'a std::path::Path {
    if path.starts_with("./") {
        path.strip_prefix("./").unwrap()
    } else {
        path.as_path()
    }
}

lazy_static! {
    static ref TITLE_COLOR: ColorSpec = {
        let mut title_color = ColorSpec::new();
        title_color.set_fg(Some(Color::Magenta));
        title_color
    };
    static ref DEFAULT_COLOR: ColorSpec = ColorSpec::default();
    static ref LANGUAGE_COLOR: ColorSpec = {
        let mut language_color = ColorSpec::new();
        language_color.set_fg(Some(Color::Green));
        language_color
    };
}