use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

use crate::ast::UnionExpr;
use crate::formalism;

#[derive(Debug)]
pub struct Glushkov {
    start_states: Vec<bool>,
    transition_matrix: Vec<Vec<Option<char>>>,
    finite_states: Vec<bool>,
    size: usize,
}

#[derive(Eq, Hash, PartialEq, Clone)]
struct State {
    a1_index: usize,
    letter: char,
    a2_index: usize,
}

struct Details {
    index: usize,
    is_finite: bool,
    incoming_states: Vec<State>,
}

impl Glushkov {
    const START_LETTER: char = 'ε';
    const START_INDEX: usize = 0;
    const START_STATE: State = State {
        a1_index: Self::START_INDEX,
        letter: Self::START_LETTER,
        a2_index: Self::START_INDEX,
    };

    const TEMPORARY_INDEX: usize = 0;

    pub fn new(linearized_symbols: &usize, ast_root: &UnionExpr) -> Self {
        let size = linearized_symbols + 1;

        let mut start_states = vec![false; size];
        start_states[Self::START_INDEX] = true;

        let mut transition_matrix = vec![vec![None; size]; size];
        for s in formalism::get_first_set(ast_root) {
            transition_matrix[Self::START_INDEX][s.index] = Some(s.letter);
        }
        for (s1, s2) in formalism::get_follow_set(ast_root) {
            transition_matrix[s1.index][s2.index] = Some(s2.letter);
        }

        let mut finite_states = vec![false; size];
        for s in formalism::get_last_set(ast_root) {
            finite_states[s.index] = true;
        }
        // TODO: finite_states[Self::START_INDEX]

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
        start_states[Self::START_INDEX] = true;

        let mut transition_matrix = vec![vec![None; size]; size];
        let mut iter = transition_matrix.iter_mut();

        {
            let start_transitions = iter.next().unwrap();
            start_transitions[1..a1.size]
                .clone_from_slice(&a1.transition_matrix[Self::START_INDEX][1..]);
            start_transitions[a1.size..]
                .clone_from_slice(&a2.transition_matrix[Self::START_INDEX][1..]);
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
        finite_states[Self::START_INDEX] =
            a1.finite_states[Self::START_INDEX] || a2.finite_states[Self::START_INDEX];
        finite_states[1..a1.size].clone_from_slice(&a1.finite_states[1..]);
        finite_states[a1.size..].clone_from_slice(&a2.finite_states[1..]);

        Self {
            start_states,
            transition_matrix,
            finite_states,
            size,
        }
    }

    // TODO: check for emptiness
    pub fn concatenation(a1: &Self, a2: &Self) -> Self {
        let size = a1.size + a2.size - 1;

        let mut start_states = vec![false; size];
        start_states[Self::START_INDEX] = true;

        let mut transition_matrix = vec![vec![None; size]; size];
        let mut iter = transition_matrix.iter_mut();

        for i in 0..a1.size {
            let transitions = iter.next().unwrap();
            transitions[1..a1.size].clone_from_slice(&a1.transition_matrix[i][1..]);
            if a1.finite_states[i] {
                transitions[a1.size..]
                    .clone_from_slice(&a2.transition_matrix[Self::START_INDEX][1..]);
            }
        }

        for i in 1..a2.size {
            let transitions = iter.next().unwrap();
            transitions[a1.size..].clone_from_slice(&a2.transition_matrix[i][1..]);
        }

        let mut finite_states = vec![false; size];
        if a2.finite_states[Self::START_INDEX] {
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

    pub fn intersection(a1: &Self, a2: &Self) -> Self {
        let mut state_details_map = HashMap::<State, Details>::new();

        Self::dfs(a1, a2, &mut state_details_map);
        Self::remove_traps(&mut state_details_map);

        let mut size = 1;
        for details in state_details_map.values_mut() {
            details.index = size;
            size += 1;
        }

        state_details_map.insert(
            Self::START_STATE,
            Details {
                index: Self::START_INDEX,
                is_finite: a1.finite_states[Self::START_INDEX]
                        && a2.finite_states[Self::START_INDEX],
                incoming_states: Vec::<State>::new(),
            },
        );

        let mut start_states = vec![false; size];
        start_states[Self::START_INDEX] = true;

        let mut transition_matrix = vec![vec![None; size]; size];
        for (state, details) in &state_details_map {
            let j = details.index;
            for incoming_state in &details.incoming_states {
                let i = state_details_map[&incoming_state].index;
                transition_matrix[i][j] = Some(state.letter);
            }
        }

        let mut finite_states = vec![false; size];
        for details in state_details_map.values() {
            if details.is_finite {
                finite_states[details.index] = true;
            }
        }

        Self {
            start_states,
            transition_matrix,
            finite_states,
            size,
        }
    }

    fn dfs(a1: &Self, a2: &Self, state_details_map: &mut HashMap<State, Details>) {
        let a2_transitions = Self::transform_transitions(a2);

        let mut states_deq = VecDeque::<State>::new();
        states_deq.push_back(Self::START_STATE);

        while let Some(state) = states_deq.pop_front() {
            let a1_transition_row = &a1.transition_matrix[state.a1_index][1..];
            for (a1_index, letter_opt) in a1_transition_row.iter().enumerate() {
                if letter_opt.is_none() {
                    continue;
                }

                let letter = letter_opt.unwrap();
                if let Some(a2_indices) = a2_transitions[state.a2_index].get(&letter) {
                    for &a2_index in a2_indices {
                        let outcoming_state = State {
                            a1_index,
                            letter,
                            a2_index,
                        };

                        if let Some(details) = state_details_map.get_mut(&outcoming_state) {
                            details.incoming_states.push(state.clone());
                        } else {
                            state_details_map.insert(
                                outcoming_state.clone(),
                                Details {
                                    index: Self::TEMPORARY_INDEX,
                                    is_finite: a1.finite_states[a1_index]
                                            && a2.finite_states[a2_index],
                                    incoming_states: vec![state.clone()],
                                },
                            );

                            states_deq.push_back(outcoming_state);
                        }
                    }
                }
            }
        }
    }

    fn transform_transitions(a: &Self) -> Vec<HashMap<char, Vec<usize>>> {
        let mut transformed_transitions = vec![HashMap::<char, Vec<usize>>::new(); a.size];

        for (i, transition_row) in a.transition_matrix.iter().enumerate() {
            for (j, letter_opt) in transition_row[1..].iter().enumerate() {
                if letter_opt.is_none() {
                    continue;
                }

                let letter = letter_opt.unwrap();
                if let Some(indices) = transformed_transitions[i].get_mut(&letter) {
                    indices.push(j);
                } else {
                    transformed_transitions[i].insert(letter, vec![j]);
                }
            }
        }

        transformed_transitions
    }

    fn remove_traps(state_details_map: &mut HashMap<State, Details>) {
        let mut visited_states_set = HashSet::<State>::new();

        let mut states_deq = VecDeque::<State>::new();
        for (state, details) in state_details_map.iter() {
            if details.is_finite {
                states_deq.push_back(state.clone());
                visited_states_set.insert(state.clone());
            }
        }

        while let Some(state) = states_deq.pop_front() {
            for incoming_state in &state_details_map[&state].incoming_states {
                if !visited_states_set.contains(incoming_state) {
                    states_deq.push_back(incoming_state.clone());
                    visited_states_set.insert(incoming_state.clone());
                }
            }
        }

        state_details_map.retain(|state, _details| visited_states_set.contains(state));
    }
}
