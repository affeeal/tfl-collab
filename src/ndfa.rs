pub mod ast;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Automata<T = char> {
    pub size: usize,
    pub transition_matrix: Vec<Vec<Option<T>>>,
    start_states: Vec<bool>,
    finite_states: Vec<bool>,
}

pub const START: usize = 0;

const ARBITARY: char = '.';
const EPSILON: String = String::new();

impl<T: std::clone::Clone> Automata<T> {
    fn new(size: usize) -> Self {
        let mut start_states = vec![false; size];
        start_states[START] = true;

        let transition_matrix = vec![vec![None::<T>; size]; size];

        let finite_states = vec![false; size];

        Self {
            start_states,
            transition_matrix,
            finite_states,
            size,
        }
    }

    pub fn is_start_state(&self, i: usize) -> bool {
        return self.start_states[i];
    }

    pub fn is_finite_state(&self, i: usize) -> bool {
        return self.finite_states[i];
    }
}

impl Automata {
    pub fn new_empty() -> Self {
        Self::new(1)
    }

    pub fn new_epsilon() -> Self {
        let mut automata = Self::new(1);
        automata.finite_states[START] = true;

        automata
    }

    pub fn is_empty(&self) -> bool {
        self.size == 1
            && self.is_start_state(START)
            && self.transition_matrix[START][START].is_none()
            && !self.is_finite_state(START)
    }

    pub fn from_regex(regex: &str) -> Self {
        if regex.is_empty() {
            return Self::new_epsilon();
        }

        let tree = ast::Tree::from_regex(regex);
        let mut automata = Self::new(tree.linearized_symbols + 1);

        for s in tree.get_first_set() {
            automata.transition_matrix[START][s.index] = Some(s.symbol);
        }
        for (s1, s2) in tree.get_follow_set() {
            automata.transition_matrix[s1.index][s2.index] = Some(s2.symbol);
        }

        automata.finite_states[START] = tree.does_epsilon_satisfy();
        for s in tree.get_last_set() {
            automata.finite_states[s.index] = true;
        }

        automata
    }

    pub fn to_regex(&self) -> Option<String> {
        let mut automata = self.prepare_for_state_elimination();

        loop {
            let mut current = START;
            while current < automata.size
                && (automata.is_start_state(current) || automata.is_finite_state(current))
            {
                current += 1;
            }

            if current == automata.size {
                break match &automata.transition_matrix[START].last().unwrap() {
                    Some(regex) => Some(format!("^{regex}$")),
                    None => None,
                };
            }

            for incoming in automata.get_incoming_states(current) {
                for outcoming in automata.get_outcoming_states(current) {
                    automata.eliminate_transition(incoming, current, outcoming);
                }
            }

            automata.eliminate_state(current);
        }
    }

    fn prepare_for_state_elimination(&self) -> Automata<String> {
        let mut automata = Automata::<String>::new(self.size + 1);

        for (i, row) in self.transition_matrix.iter().enumerate() {
            for (j, symbol_opt) in row.iter().enumerate() {
                if let Some(symbol) = symbol_opt {
                    automata.transition_matrix[i][j] = Some(symbol.to_string());
                }
            }

            if self.is_finite_state(i) {
                *automata.transition_matrix[i].last_mut().unwrap() = Some(EPSILON);
            }
        }

        *automata.finite_states.last_mut().unwrap() = true;

        automata
    }

    fn transform_transitions(&self) -> Vec<HashMap<char, Vec<usize>>> {
        let mut transitions = vec![HashMap::<char, Vec<usize>>::new(); self.size];

        for (i, row) in self.transition_matrix.iter().enumerate() {
            for (j, symbol_opt) in row.iter().enumerate() {
                if symbol_opt.is_none() {
                    continue;
                }

                let symbol = symbol_opt.unwrap();
                if let Some(indices) = transitions[i].get_mut(&symbol) {
                    indices.push(j);
                } else {
                    transitions[i].insert(symbol, vec![j]);
                }
            }
        }

        transitions
    }
}

impl Automata<String> {
    fn eliminate_transition(&mut self, incoming: usize, current: usize, outcoming: usize) {
        let former_regex_opt = &self.transition_matrix[incoming][outcoming];
        let incoming_regex = self.transition_matrix[incoming][current].as_ref().unwrap();
        let cyclic_regex_opt = &self.transition_matrix[current][current];
        let outcoming_regex = self.transition_matrix[current][outcoming].as_ref().unwrap();

        // Optimisations
        if Self::is_unfold_axiom_applicable(
            former_regex_opt,
            incoming_regex,
            cyclic_regex_opt,
            outcoming_regex,
        ) {
            self.transition_matrix[incoming][outcoming] =
                Some(format!("{}*", Self::wrap_if_needed(incoming_regex)));
            return;
        }

        // TODO: idempotency, distributivity

        // Common scenario
        let mut result = String::new();

        let mut can_be_epsilon = false;
        let mut must_be_wrapped = false;

        if let Some(former_regex) = former_regex_opt {
            if former_regex.eq(&EPSILON) {
                can_be_epsilon = true;
            } else {
                must_be_wrapped = true;
                result.push_str(&format!("{former_regex}|"));
            }
        }

        result.push_str(incoming_regex);

        if let Some(cyclic_regex) = cyclic_regex_opt {
            result.push_str(&format!("{}*", Self::wrap_if_needed(cyclic_regex)));
        }

        if outcoming_regex.ne(&EPSILON) {
            result.push_str(outcoming_regex);
        }

        if must_be_wrapped {
            result = Self::wrap(&result);
        }

        if can_be_epsilon {
            result = format!("{}?", Self::wrap_if_needed(&result));
        }

        self.transition_matrix[incoming][outcoming] = Some(result);
    }

    fn is_unfold_axiom_applicable(
        former_regex_opt: &Option<String>,
        incoming_regex: &String,
        cyclic_regex_opt: &Option<String>,
        outcoming_regex: &String,
    ) -> bool {
        *former_regex_opt == Some(EPSILON)
            && Some(incoming_regex.clone()) == *cyclic_regex_opt
            && *outcoming_regex == EPSILON
    }

    fn wrap_if_needed(regex: &String) -> String {
        if regex.len() == 1
            || regex.chars().next() == Some('(') && regex.chars().last() == Some(')')
        {
            return regex.to_string();
        }

        Self::wrap(regex)
    }

    fn wrap(regex: &String) -> String {
        format!("({regex})")
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
    let mut automata = Automata::<char>::new(a1.size + a2.size - 1);

    let mut iter = automata.transition_matrix.iter_mut();

    let start_transitions = iter.next().unwrap();
    start_transitions[1..a1.size].clone_from_slice(&a1.transition_matrix[START][1..]);
    start_transitions[a1.size..].clone_from_slice(&a2.transition_matrix[START][1..]);

    for i in 1..a1.size {
        let a1_transitions = iter.next().unwrap();
        a1_transitions[1..a1.size].clone_from_slice(&a1.transition_matrix[i][1..]);
    }

    for i in 1..a2.size {
        let a2_transitions = iter.next().unwrap();
        a2_transitions[a1.size..].clone_from_slice(&a2.transition_matrix[i][1..]);
    }

    automata.finite_states[START] = a1.is_finite_state(START) || a2.is_finite_state(START);
    automata.finite_states[1..a1.size].clone_from_slice(&a1.finite_states[1..]);
    automata.finite_states[a1.size..].clone_from_slice(&a2.finite_states[1..]);

    automata
}

pub fn concatenation(a1: &Automata, a2: &Automata) -> Automata {
    if a1.is_empty() || a2.is_empty() {
        return Automata::new_empty();
    }

    let mut automata = Automata::<char>::new(a1.size + a2.size - 1);

    let mut iter = automata.transition_matrix.iter_mut();

    for i in 0..a1.size {
        let transitions = iter.next().unwrap();
        transitions[1..a1.size].clone_from_slice(&a1.transition_matrix[i][1..]);
        if a1.is_finite_state(i) {
            transitions[a1.size..].clone_from_slice(&a2.transition_matrix[START][1..]);
        }
    }

    for i in 1..a2.size {
        let transitions = iter.next().unwrap();
        transitions[a1.size..].clone_from_slice(&a2.transition_matrix[i][1..]);
    }

    if a2.is_finite_state(START) {
        automata.finite_states[..a1.size].clone_from_slice(a1.finite_states.as_slice());
    }
    automata.finite_states[a1.size..].clone_from_slice(&a2.finite_states[1..]);

    automata
}

// Intersection

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
struct ComplexState {
    a1_index: usize,
    symbol: char,
    a2_index: usize,
}

#[derive(Debug)]
struct Details {
    index: usize,
    is_finite: bool,
    incoming_states: Vec<ComplexState>,
}

const TEMPORARY_INDEX: usize = 0;
const START_STATE: ComplexState = ComplexState {
    a1_index: START,
    symbol: '\0',
    a2_index: START,
};

pub fn intersection(a1: &Automata, a2: &Automata) -> Automata {
    let mut state_details_map = HashMap::<ComplexState, Details>::new();
    state_details_map.insert(
        START_STATE,
        Details {
            index: START,
            is_finite: a1.is_finite_state(START) && a2.is_finite_state(START),
            incoming_states: Vec::<ComplexState>::new(),
        },
    );

    intersection_bfs(a1, a2, &mut state_details_map);
    remove_traps(&mut state_details_map);

    let mut size = 1;
    for (state, details) in state_details_map.iter_mut() {
        if state != &START_STATE {
            details.index = size;
            size += 1;
        }
    }

    let mut automata = Automata::<char>::new(size);

    for (state, details) in &state_details_map {
        let j = details.index;
        for incoming_state in &details.incoming_states {
            let i = state_details_map[&incoming_state].index;
            automata.transition_matrix[i][j] = Some(state.symbol);
        }
    }

    for details in state_details_map.values() {
        if details.is_finite {
            automata.finite_states[details.index] = true;
        }
    }

    automata
}

fn intersection_bfs(
    a1: &Automata,
    a2: &Automata,
    state_details_map: &mut HashMap<ComplexState, Details>,
) {
    let a2_transitions = a2.transform_transitions();

    let mut states_deq = VecDeque::<ComplexState>::new();
    states_deq.push_back(START_STATE);

    while let Some(state) = states_deq.pop_front() {
        let a1_row = &a1.transition_matrix[state.a1_index];
        for (a1_index, symbol_opt) in a1_row.iter().enumerate() {
            if symbol_opt.is_none() {
                continue;
            }

            let mut outcoming_indices = Vec::<(&char, &Vec<usize>)>::new();

            let symbol = &symbol_opt.unwrap();
            if let Some(a2_indices) = a2_transitions[state.a2_index].get(&symbol) {
                outcoming_indices.push((symbol, a2_indices));
            } else if symbol == &ARBITARY {
                for (symbol, a2_indices) in &a2_transitions[state.a2_index] {
                    outcoming_indices.push((symbol, a2_indices));
                }
            }

            for (&symbol, a2_indices) in outcoming_indices {
                for &a2_index in a2_indices {
                    let outcoming_state = ComplexState {
                        a1_index,
                        symbol,
                        a2_index,
                    };

                    if let Some(details) = state_details_map.get_mut(&outcoming_state) {
                        details.incoming_states.push(state.clone());
                    } else {
                        state_details_map.insert(
                            outcoming_state.clone(),
                            Details {
                                index: TEMPORARY_INDEX,
                                is_finite: a1.finite_states[a1_index] && a2.finite_states[a2_index],
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

fn remove_traps(state_details_map: &mut HashMap<ComplexState, Details>) {
    let mut visited_states_set = HashSet::<ComplexState>::new();

    let mut states_deq = VecDeque::<ComplexState>::new();
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
