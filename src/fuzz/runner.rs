use crate::fuzz::str_generator;

use super::regex_generator::{self, RegexGenerator};
use fancy_regex::Regex;
use log::{error, info};

pub fn run_tests(regex_count: usize, strs_count: usize, cfg: &regex_generator::Config) {
    let generator = RegexGenerator::new(cfg);

    let regexes = generator.generate(regex_count);

    for r in regexes {
        run_tests_for_regex(&r, strs_count)
    }
}

pub fn run_tests_for_regex(r: &str, strs_count: usize) {
    info!("starting tests for regex {}...", r);
    info!("creating automata...");
    let automata = crate::convertor::gen_rec(r).unwrap();
    info!(
        "generated regex: {}",
        automata.to_regex().unwrap_or_else(|| "^$".to_string())
    );
    let mut str_gen = str_generator::StringGenerator::from_automata(&automata);
    info!("generating strings...");
    let strs = str_gen.gen_strs(strs_count);
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

    let with_lookahead = Regex::new(r).unwrap();
    let without_lookahead = Regex::new(&regex).unwrap();

    for str in strs {
        let lhs = with_lookahead.is_match(&str);
        let rhs = without_lookahead.is_match(&str);
		
        if let Err(e) = lhs {
            error!("got err: {}", e);
            continue;
        }

        if let Err(e) = rhs {
            error!("got err: {}", e);
            continue;
        }

        let lhs = lhs.unwrap();
        let rhs = rhs.unwrap();
		
        if lhs != rhs {
            error!("\t failed with string: '{}'", str);
        } else {
            info!("\t string: '{}' OK", str);
        }
    }
}
