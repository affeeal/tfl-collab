use fuzz::{regex_generator, runner};

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
    #[clap(value_parser, long)]
    lookahead_count: Option<usize>,
    #[clap(value_parser, long)]
    star_height: Option<usize>,
    #[clap(value_parser, long)]
    letter_count: Option<usize>,
}

fn main() {
    env_logger::init();
    let cli = Args::parse();

    let mut regex = "".to_string();
    let mut regex_count = 50;
    let mut string_count = 10;
    let mut cfg = regex_generator::Config {
        max_lookahead_count: 4,
        star_height: 2,
        alphabet_size: 3,
        max_letter_count: 10,
    };

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

    match cli.star_height {
        Some(h) => cfg.star_height = h,
        None => {}
    }

    match cli.lookahead_count {
        Some(c) => cfg.max_letter_count = c,
        None => {}
    }

    match cli.letter_count {
        Some(c) => cfg.max_letter_count = c,
        None => {}
    }

    if !regex.is_empty() {
        runner::run_tests_for_regex(&regex, string_count);
    } else {
        runner::run_tests(regex_count, string_count, &cfg);
    }
}
