use fuzz::runner;

pub mod convertor;
pub mod fuzz;
pub mod ndfa;
pub mod parser;

fn main() {
    env_logger::init();
    runner::run_tests(50, 10);
}
