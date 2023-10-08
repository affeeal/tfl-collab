use crate::ast::UnionExpr;
use crate::formalism;

#[derive(Debug)]
pub struct Glushkov {
    start_states: Vec<bool>,
    transition_matrix: Vec<Vec<Option<char>>>,
    finite_states: Vec<bool>,
    size: usize,
}

impl Glushkov {
    const START_STATE: usize = 0;

    pub fn new(linearized_symbols: &usize, ast_root: &UnionExpr) -> Self {
        let size = linearized_symbols + 1;

        let mut start_states = vec![false; size];
        start_states[Self::START_STATE] = true;

        let mut transition_matrix = vec![vec![None; size]; size];
        for s in formalism::get_first_set(ast_root) {
            transition_matrix[Self::START_STATE][s.index] = Some(s.letter);
        }
        for (s1, s2) in formalism::get_follow_set(ast_root) {
            transition_matrix[s1.index][s2.index] = Some(s2.letter);
        }

        let mut finite_states = vec![false; size];
        for s in formalism::get_last_set(ast_root) {
            finite_states[s.index] = true;
        }
        // TODO: finite_states[Self::START_STATE]

        Self {
            start_states,
            transition_matrix,
            finite_states,
            size,
        }
    }

    pub fn union(a1: &Self, a2: &Self) -> Self {
        let size = a1.size + a2.size - 1;

        let mut start_states = vec![false; size];
        start_states[Self::START_STATE] = true;

        let mut transition_matrix = vec![vec![None; size]; size];
        let mut iter = transition_matrix.iter_mut();

        {
            let start_transitions = iter.next().unwrap();
            start_transitions[1..a1.size]
                .clone_from_slice(&a1.transition_matrix[Self::START_STATE][1..]);
            start_transitions[a1.size..]
                .clone_from_slice(&a2.transition_matrix[Self::START_STATE][1..]);
        }

        for i in 1..a1.size {
            let a1_transitions = iter.next().unwrap();
            a1_transitions[1..a1.size].clone_from_slice(&a1.transition_matrix[i][1..]);
        }

        for i in 1..a2.size {
            let a2_transitions = iter.next().unwrap();
            a2_transitions[a1.size..].clone_from_slice(&a2.transition_matrix[i][1..]);
        }

        let mut finite_states = vec![false; size];
        finite_states[Self::START_STATE] =
            a1.finite_states[Self::START_STATE] || a2.finite_states[Self::START_STATE];
        finite_states[1..a1.size].clone_from_slice(&a1.finite_states[1..]);
        finite_states[a1.size..].clone_from_slice(&a2.finite_states[1..]);

        Self {
            start_states,
            transition_matrix,
            finite_states,
            size,
        }
    }

    pub fn concatenation(a1: &Self, a2: &Self) -> Self {
        let size = a1.size + a2.size - 1;

        let mut start_states = vec![false; size];
        start_states[Self::START_STATE] = true;

        let mut transition_matrix = vec![vec![None; size]; size];
        let mut iter = transition_matrix.iter_mut();

        for i in 0..a1.size {
            let transitions = iter.next().unwrap();
            transitions[1..a1.size].clone_from_slice(&a1.transition_matrix[i][1..]);
            if a1.finite_states[i] {
                transitions[a1.size..]
                    .clone_from_slice(&a2.transition_matrix[Self::START_STATE][1..]);
            }
        }

        for i in 1..a2.size {
            let transitions = iter.next().unwrap();
            transitions[a1.size..].clone_from_slice(&a2.transition_matrix[i][1..]);
        }

        let mut finite_states = vec![false; size];
        if a2.finite_states[Self::START_STATE] {
            finite_states[..a1.size].clone_from_slice(a1.finite_states.as_slice());
        }
        finite_states[a1.size..].clone_from_slice(&a2.finite_states[1..]);

        Self {
            start_states,
            transition_matrix,
            finite_states,
            size,
        }

    }

    // TODO: intersection

    // TODO: tests
}
