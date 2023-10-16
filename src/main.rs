pub mod fuzz;
pub mod ndfa;

fn main() {
    let a = ndfa::Automata::from_regex(&"(aa|b)*".to_string());
    dbg!(fuzz::str_generator::generate_str(&a, 100));
}
