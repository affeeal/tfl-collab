pub mod fuzz;

use fuzz::regex_generator::{self, RegexGenerator};

fn main() {
    let conf = fuzz::regex_generator::Config {
        max_lookahead_count: 2,
        star_height: 3,
        alphabet_size: 3,
        max_letter_count: 8,
    };

    let gen = RegexGenerator::new(&conf);

    let res = gen.generate(10);

    for c in res {
        println!("{:?}", c);
    }
}
