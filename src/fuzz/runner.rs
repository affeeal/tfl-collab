use crate::fuzz::str_generator;

use super::regex_generator::{self, RegexGenerator};
use fancy_regex::Regex;
use log::{error, info};

pub fn run_tests(regex_count: usize, strs_count: usize) {
    let cfg = regex_generator::Config {
        max_lookahead_count: 4,
        star_height: 2,
        alphabet_size: 4,
        max_letter_count: 8,
    };

    let generator = RegexGenerator::new(&cfg);

    let regexes = generator.generate(regex_count);

    for r in regexes {
        info!("starting tests for regex {}...", r);
        info!("creating automata...");
        let automata = crate::convertor::gen_rec(&r).unwrap();
        info!("generating strings...");
        let strs = str_generator::generate_str(&automata, strs_count);
        info!("strings generation successfully");
        info!("running tests...");
        let mut regex = "".to_string();
        automata
            .to_regex()
            .unwrap_or_else(|| "^$".to_string())
            .chars()
            .for_each(|c| match c {
                'ε' => regex.push_str("(.?)"),
                _ => regex.push(c),
            });
        let with_lookahead = Regex::new(&r).unwrap();
        let without_lookahead = Regex::new(&regex).unwrap();
        for str in strs {
            if with_lookahead.is_match(&str).unwrap() != without_lookahead.is_match(&str).unwrap() {
                error!("\t failed with string: '{}'", str);
            } else {
                info!("\t string: '{}' OK", str);
            }
        }
    }
}
