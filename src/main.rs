pub mod ast;
pub mod automata;
pub mod formalism;

fn main() {
    let regex = "(ab|b)*a|a(a|b)b*";
    let (linearized_symbols, ast_root) = ast::parse(&regex);
    dbg!(automata::Glushkov::new(&linearized_symbols, &ast_root));
}
