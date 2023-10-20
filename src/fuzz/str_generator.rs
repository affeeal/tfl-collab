use std::collections::HashSet;
use std::collections::VecDeque;

use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::ndfa;

pub struct StringGenerator<'a> {
    automata: &'a ndfa::Automata,
    rng: ThreadRng,
}

impl<'a> StringGenerator<'a> {
    const FINITE_STATE_PROBABILITY: f64 = 0.15;
    const COMPLETE_WORD_PROBABILITY: f64 = 0.30;

    const EPSILON_CHAIN: [usize; 2] = [ndfa::START_INDEX; 2];

    pub fn from_automata(automata: &'a ndfa::Automata) -> Self {
        Self {
            automata,
            rng: rand::thread_rng(),
        }
    }

    pub fn gen_strs(&mut self/*, count: &usize*/) /* -> Vec<String> */ {
        let reach_arr = self.automata.get_reach_arr();
        let reach_vec = transform_reach_arr(reach_arr);

        let _states_chain = dbg!(self.gen_states_chain(&reach_vec));
        let _words_chain = dbg!(self.gen_words_chain(&_states_chain));

        // TODO
    }

    fn gen_states_chain(&mut self, reach_vec: &Vec<Vec<usize>>) -> Vec<usize> {
        // Empty automata corner case
        if self.automata.is_empty() {
            return Vec::new();
        }

        let mut states = Vec::<usize>::new();
        states.push(ndfa::START_INDEX);

        let mut current_state = ndfa::START_INDEX;
        loop {
            // Nothing to visit or can exit
            if reach_vec[current_state].is_empty()
                || self.automata.is_finite_state(current_state)
                    && self.rng.gen_bool(Self::FINITE_STATE_PROBABILITY)
            {
                break;
            }

            let next_state = reach_vec[current_state].choose(&mut self.rng).unwrap();
            states.push(next_state.clone());
            current_state = next_state.clone();
        }

        // Epsilon corner case
        if current_state.eq(&ndfa::START_INDEX) {
            return Vec::from(Self::EPSILON_CHAIN);
        }

        states
    }

    fn gen_words_chain(&mut self, states_chain: &Vec<usize>) -> Vec<String> {
        if states_chain.eq(&Self::EPSILON_CHAIN) {
            return vec![String::new()];
        }

        let mut words_chain = Vec::<String>::with_capacity(states_chain.len() - 1);

        let mut first_iter = states_chain.iter();
        let mut second_iter = states_chain.iter().skip(1);

        while let Some(to) = second_iter.next() {
            let from = first_iter.next().unwrap();

            words_chain.push(self.gen_word(&from, &to));
            // NOTE: можно генерировать несколько слов на отрезке.
        }

        words_chain
    }

    fn gen_word(&mut self, from: &usize, to: &usize) -> String {
        let mut states_deq = VecDeque::<(String, usize)>::new();
        states_deq.push_back((String::new(), from.clone()));

        while let Some((word_prefix, state)) = states_deq.pop_front() {
            if state.eq(to)
                && self.rng.gen_bool(Self::COMPLETE_WORD_PROBABILITY)
                && !word_prefix.is_empty()
            {
                return word_prefix;
            }

            for (i, letter_opt) in self.automata.transition_matrix[state].iter().enumerate() {
                if let Some(letter) = letter_opt {
                    states_deq.push_back((word_prefix.clone() + &letter.to_string(), i));
                }
            }
        }

        unreachable!()
    }
}

impl ndfa::Automata {
    fn _get_alphabet(&self) -> HashSet<char> {
        let mut alphabet = HashSet::<char>::new();

        for row in &self.transition_matrix {
            for letter_opt in row {
                if let Some(letter) = letter_opt {
                    alphabet.insert(letter.clone());
                }
            }
        }

        alphabet
    }

    fn get_reach_arr(&self) -> ndarray::Array2<i32> {
        let mut adj_vec = vec![0; self.size * self.size];
        for (i, row) in self.transition_matrix.iter().enumerate() {
            for (j, letter_opt) in row.iter().enumerate() {
                if letter_opt.is_some() {
                    adj_vec[i * self.size + j] = 1;
                }
            }
        }

        let adj_arr = ndarray::Array2::from_shape_vec((self.size, self.size), adj_vec).unwrap();

        let mut reach_arr = adj_arr.clone();
        let mut comp_arr = adj_arr.clone();
        for _ in 0..self.size - 1 {
            comp_arr = comp_arr.dot(&adj_arr);
            reach_arr += &comp_arr;
        }

        reach_arr
    }
}

fn transform_reach_arr(reach_arr: ndarray::Array2<i32>) -> Vec<Vec<usize>> {
    let mut reach_vec = vec![Vec::<usize>::new(); reach_arr.dim().0];

    for ((i, j), paths) in reach_arr.indexed_iter() {
        if paths.gt(&0) {
            reach_vec[i].push(j);
        }
    }

    reach_vec
}

