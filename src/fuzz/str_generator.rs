use std::collections::VecDeque;

use crate::ndfa::Automata;

pub fn generate_str(a: &Automata, count: usize) -> Vec<String> {
    let mut strs = Vec::<String>::new();

    let mut strs_deq = VecDeque::<(String, usize)>::new();
    strs_deq.push_back(("".to_string(), 0));

    while let Some((s, i)) = strs_deq.pop_front() {
        if a.is_finite_state(i) {
            strs.push(s.clone());
        }

        if strs.len() >= count {
            break;
        }

        for (j, letter_opt) in a.transition_matrix[i].iter().enumerate() {
            if letter_opt.is_none() {
                continue;
            }

            let letter = letter_opt.unwrap();
            strs_deq.push_back((mutate(format!("{s}{letter}")), j));
        }
    }

    strs
}

fn mutate(s: String) -> String {
    unimplemented!()
}
