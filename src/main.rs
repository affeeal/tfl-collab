pub mod fuzz;
pub mod ndfa;

fn main() {
    let r1 = "(ab)*|ba";

    let a1 = dbg!(ndfa::Automata::from_regex(&r1.to_string()));
    let mut generator = fuzz::str_generator::StringGenerator::from_automata(&a1);
    generator.gen_strings(&5);
}
