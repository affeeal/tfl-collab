pub mod fuzz;
pub mod ndfa;

fn main() {
    let r1 = "a*|b(aa|b)*";

    let _a1 = dbg!(ndfa::Automata::from_regex(&r1.to_string()));
    let _em = ndfa::Automata::new_empty();
    let _ep = ndfa::Automata::new_epsilon();

    let mut generator = fuzz::str_generator::StringGenerator::from_automata(&_a1);
    generator.gen_strs(&5);
}
