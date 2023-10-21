use fuzz::runner;

pub mod convertor;
pub mod fuzz;
pub mod ndfa;
pub mod parser;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(value_parser, long)]
    regex_count: Option<usize>,
    #[clap(value_parser, long)]
    string_count: Option<usize>,
    #[clap(value_parser, long)]
    regex: Option<String>,
}

fn main() {
    env_logger::init();
    let cli = Args::parse();

    let mut regex = "".to_string();
    let mut regex_count = 50;
    let mut string_count = 10;

    match cli.regex {
        Some(expr) => regex = expr,
        None => {}
    }

    match cli.regex_count {
        Some(c) => regex_count = c,
        None => {}
    }

    match cli.string_count {
        Some(c) => string_count = c,
        None => {}
    }

    if !regex.is_empty() {
        runner::run_tests_for_regex(&regex, string_count);
    } else {
        runner::run_tests(regex_count, string_count);
    }
}
