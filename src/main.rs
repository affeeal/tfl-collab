pub mod fuzz;

fn main() {
    let conf = fuzz::regex_generator::Config {
        max_lookahead_count: 2,
        star_height: 1,
        alphabet_size: 5,
        max_letter_count: 8,
    };

    let res = fuzz::regex_generator::generate(&conf, 10);

    for c in res {
        println!("{:?}", c);
    }
}
