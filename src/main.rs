pub mod fuzz;

fn main() {
    let conf = fuzz::regex_generator::Config {
        max_lookahead_count: 2,
        star_height: 1,
        alphabet_size: 3,
        max_letter_count: 5,
    };

    let res = fuzz::regex_generator::generate(&conf, 5);

    for c in res {
        println!("{:?}", c);
    }
}
