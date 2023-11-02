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

    if let Some(expr) = cli.regex {
        regex = expr;
    }

    if let Some(c) = cli.regex_count {
        regex_count = c;
    }

    if let Some(c) = cli.string_count {
        string_count = c
    }

    if let Some(h) = cli.star_height {
        cfg.star_height = h
    }

    if let Some(c) = cli.lookahead_count {
        cfg.max_letter_count = c
    }

    if let Some(c) = cli.letter_count {
        cfg.max_letter_count = c
    }

    if regex.is_empty() {
        runner::run_tests(regex_count, string_count, &cfg);
    } else {
        runner::run_tests_for_regex(&regex, string_count);
    }
}
