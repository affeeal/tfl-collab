// non-deterministic finite automata (ndfa)

mod ast;
mod ffl;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

pub struct Automata<T = char> {
    start_states: Vec<bool>,
    transition_matrix: Vec<Vec<Option<T>>>,
    finite_states: Vec<bool>,
    size: usize,
}

const START_INDEX: usize = 0;
const EPSILON: char = 'ε';

impl<T> Automata<T> {
    fn is_start_state(&self, i: usize) -> bool {
        return self.start_states[i];
    }

    fn is_finite_state(&self, i: usize) -> bool {
        return self.finite_states[i];
    }
}

impl Automata<char> {
    const FINITE_INDEX: usize = 1;

    pub fn from_regex(regex: &String) -> Self {
        if regex.is_empty() {
            return Self::epsilon_automata();
        }

        let tree = ast::Tree::from_regex(regex);

        let size = tree.linearized_symbols + 1;

        let mut start_states = vec![false; size];
        start_states[START_INDEX] = true;

        let mut transition_matrix = vec![vec![None; size]; size];
        for s in ffl::get_first_set(&tree) {
            transition_matrix[START_INDEX][s.index] = Some(s.letter);
        }
        for (s1, s2) in ffl::get_follow_set(&tree) {
            transition_matrix[s1.index][s2.index] = Some(s2.letter);
        }

        let mut finite_states = vec![false; size];
        finite_states[START_INDEX] = ffl::does_epsilon_satisfies(&tree);
        for s in ffl::get_last_set(&tree) {
            finite_states[s.index] = true;
        }

        Self {
            start_states,
            transition_matrix,
            finite_states,
            size,
        }
    }

    fn epsilon_automata() -> Self {
        let size = 1;
        let start_states = vec![true; size];
        let transition_matrix = vec![vec![None; size]; size];
        let finite_states = vec![true; size];

        Self {
            start_states,
            transition_matrix,
            finite_states,
            size
        }
    }

    pub fn to_regex(&self) -> Option<String> {
        let mut ndfa = self.prepare_for_state_elimination();

        loop {
            let mut current = START_INDEX;
            while current < ndfa.size
                && (ndfa.is_start_state(current) || ndfa.is_finite_state(current))
            {
                current += 1;
            }

            if current == ndfa.size {
                break ndfa.transition_matrix[START_INDEX][Self::FINITE_INDEX].take();
            }

            for incoming in ndfa.get_incoming_states(current) {
                for outcoming in ndfa.get_outcoming_states(current) {
                    ndfa.eliminate_transition(incoming, current, outcoming);
                }
            }

            ndfa.eliminate_state(current);
        }
    }

    fn prepare_for_state_elimination(&self) -> Automata<String> {
        let size = self.size + 1;

        let mut start_states = vec![false; size];
        start_states[START_INDEX] = true;

        let mut transition_matrix = vec![vec![None; size]; size];
        for (i, row) in self.transition_matrix.iter().enumerate() {
            for (j, letter_opt) in row.iter().enumerate() {
                if let Some(letter) = letter_opt {
                    transition_matrix[i][j] = Some(letter.to_string());
                }
            }

            if self.is_finite_state(i) {
                transition_matrix[i][size - 1] = Some(EPSILON.to_string());
            }
        }

        let mut finite_states = vec![false; size];
        finite_states[size - 1] = true;

        Automata::<String> {
            start_states,
            transition_matrix,
            finite_states,
            size,
        }
    }

    fn transform_transitions(&self) -> Vec<HashMap<char, Vec<usize>>> {
        let mut transformed_transitions = vec![HashMap::<char, Vec<usize>>::new(); self.size];
    
        for (i, transition_row) in self.transition_matrix.iter().enumerate() {
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
}

impl Automata<String> {
    fn eliminate_transition(&mut self, incoming: usize, current: usize, outcoming: usize) {
        let former_regex_opt = &self.transition_matrix[incoming][outcoming];
        let incoming_regex = self.transition_matrix[incoming][current].as_ref().unwrap();
        let cyclic_regex_opt = &self.transition_matrix[current][current];
        let outcoming_regex = self.transition_matrix[current][outcoming].as_ref().unwrap();

        // Euristics
        if Self::is_unfold_applicable(
            former_regex_opt,
            incoming_regex,
            cyclic_regex_opt,
            outcoming_regex,
        ) {
            self.transition_matrix[incoming][outcoming] = Some(format!("{incoming_regex}*"));
            return;
        }

        // Common scenario
        let mut result = String::new();

        if let Some(former_regex) = former_regex_opt {
            if *former_regex != EPSILON.to_string() {
                result.push_str(former_regex);
                result.push('|');
            }
        }

        result.push_str(incoming_regex);

        if let Some(cyclic_regex) = cyclic_regex_opt {
            result.push_str(cyclic_regex);
            result.push('*');
        }

        if *outcoming_regex != EPSILON.to_string() {
            result.push_str(outcoming_regex);
        }

        self.transition_matrix[incoming][outcoming] = Some(Self::wrap_regex(result));
    }

    fn eliminate_state(&mut self, i: usize) {
        self.start_states.swap_remove(i);

        self.transition_matrix.swap_remove(i);
        for transition_row in &mut self.transition_matrix {
            transition_row.swap_remove(i);
        }

        self.finite_states.swap_remove(i);

        self.size -= 1;
    }

    fn is_unfold_applicable(
        former_regex_opt: &Option<String>,
        incoming_regex: &String,
        cyclic_regex_opt: &Option<String>,
        outcoming_regex: &String,
    ) -> bool {
        *former_regex_opt == Some(EPSILON.to_string())
            && Some(incoming_regex.clone()) == *cyclic_regex_opt
            && *outcoming_regex == EPSILON.to_string()
    }

    fn wrap_regex(regex: String) -> String {
        if regex.len() == 1 {
            return regex;
        }

        format!("({regex})")
    }

    fn get_outcoming_states(&self, i: usize) -> Vec<usize> {
        let mut outcoming_states = Vec::<usize>::new();

        for j in 0..self.size {
            if self.transition_matrix[i][j].is_some() && j != i {
                outcoming_states.push(j);
            }
        }

        outcoming_states
    }

    fn get_incoming_states(&self, i: usize) -> Vec<usize> {
        let mut incoming_states = Vec::<usize>::new();

        for j in 0..self.size {
            if self.transition_matrix[j][i].is_some() && j != i {
                incoming_states.push(j);
            }
        }

        incoming_states
    }
}

// Union, concatenation

pub fn union(a1: &Automata, a2: &Automata) -> Automata {
    let size = a1.size + a2.size - 1;

    let mut start_states = vec![false; size];
    start_states[START_INDEX] = true;

    let mut transition_matrix = vec![vec![None; size]; size];
    let mut iter = transition_matrix.iter_mut();

    {
        let start_transitions = iter.next().unwrap();
        start_transitions[1..a1.size].clone_from_slice(&a1.transition_matrix[START_INDEX][1..]);
        start_transitions[a1.size..].clone_from_slice(&a2.transition_matrix[START_INDEX][1..]);
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
    finite_states[START_INDEX] = a1.is_finite_state(START_INDEX) || a2.is_finite_state(START_INDEX);
    finite_states[1..a1.size].clone_from_slice(&a1.finite_states[1..]);
    finite_states[a1.size..].clone_from_slice(&a2.finite_states[1..]);

    Automata {
        start_states,
        transition_matrix,
        finite_states,
        size,
    }
}

pub fn concatenation(a1: &Automata, a2: &Automata) -> Automata {
    let size = a1.size + a2.size - 1;

    let mut start_states = vec![false; size];
    start_states[START_INDEX] = true;

    let mut transition_matrix = vec![vec![None; size]; size];
    let mut iter = transition_matrix.iter_mut();

    for i in 0..a1.size {
        let transitions = iter.next().unwrap();
        transitions[1..a1.size].clone_from_slice(&a1.transition_matrix[i][1..]);
        if a1.is_finite_state(i) {
            transitions[a1.size..].clone_from_slice(&a2.transition_matrix[START_INDEX][1..]);
        }
    }

    for i in 1..a2.size {
        let transitions = iter.next().unwrap();
        transitions[a1.size..].clone_from_slice(&a2.transition_matrix[i][1..]);
    }

    let mut finite_states = vec![false; size];
    if a2.is_finite_state(START_INDEX) {
        finite_states[..a1.size].clone_from_slice(a1.finite_states.as_slice());
    }
    finite_states[a1.size..].clone_from_slice(&a2.finite_states[1..]);

    Automata {
        start_states,
        transition_matrix,
        finite_states,
        size,
    }
}

// Intersection

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

const START_STATE: State = State {
    a1_index: START_INDEX,
    letter: EPSILON,
    a2_index: START_INDEX,
};

const TEMPORARY_INDEX: usize = 0;

pub fn intersection(a1: &Automata, a2: &Automata) -> Automata {
    let mut state_details_map = HashMap::<State, Details>::new();

    dfs(a1, a2, &mut state_details_map);
    remove_traps(&mut state_details_map);

    let mut size = 1;
    for details in state_details_map.values_mut() {
        details.index = size;
        size += 1;
    }

    state_details_map.insert(
        START_STATE,
        Details {
            index: START_INDEX,
            is_finite: a1.finite_states[START_INDEX] && a2.finite_states[START_INDEX],
            incoming_states: Vec::<State>::new(),
        },
    );

    let mut start_states = vec![false; size];
    start_states[START_INDEX] = true;

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

    Automata {
        start_states,
        transition_matrix,
        finite_states,
        size,
    }
}

fn dfs(a1: &Automata, a2: &Automata, state_details_map: &mut HashMap<State, Details>) {
    let a2_transitions = a2.transform_transitions();

    let mut states_deq = VecDeque::<State>::new();
    states_deq.push_back(START_STATE);

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
                                index: TEMPORARY_INDEX,
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
