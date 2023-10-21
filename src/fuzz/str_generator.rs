use std::collections::HashSet;
use std::collections::VecDeque;

use ndarray::Array2;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::ndfa;

pub struct StringGenerator<'a> {
    automata: &'a ndfa::Automata,
    reachability: Reachability,
    rng: ThreadRng,
}

impl<'a> StringGenerator<'a> {
    const FINITE_STATE_PROBABILITY: f64 = 0.2;
    const COMPLETE_WORD_PROBABILITY: f64 = 0.4;
    const MUTATION_PROBABILITY: f64 = 0.8;

    const EPSILON_CHAIN: [usize; 2] = [ndfa::START_INDEX; 2];
    const EPSILON_WORDS: [String; 1] = [String::new(); 1];

    const MUTATIONS_COUNT: usize = 6;

    pub fn from_automata(automata: &'a ndfa::Automata) -> Self {
        Self {
            automata,
            reachability: Reachability::from_automata(automata),
            rng: rand::thread_rng(),
        }
    }

    pub fn gen_strings(&mut self, count: usize) -> Vec<String> {
        // Empty automata corner case
        if self.automata.is_empty() {
            return Vec::new();
        }

        let mut strings = Vec::<String>::new();

        for _ in 0..count {
            let states = self.gen_states_chain();
            let mut words = self.gen_words_chain(&states);

            self.mutate(&mut words);

            strings.push(self.join_words(&words));
        }

        strings
    }

    fn gen_states_chain(&mut self) -> Vec<usize> {
        let mut states = Vec::<usize>::new();
        states.push(ndfa::START_INDEX);

        let mut current_state = ndfa::START_INDEX;
        loop {
            // Nothing to visit or can exit
            if self.reachability.as_outcoming[current_state].is_empty()
                || self.automata.is_finite_state(current_state)
                    && self.rng.gen_bool(Self::FINITE_STATE_PROBABILITY)
            {
                break;
            }

            let next_state = self.reachability.as_outcoming[current_state]
                .choose(&mut self.rng)
                .unwrap();
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
            return Self::EPSILON_WORDS.to_vec();
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
            let mut outcoming = Vec::<(String, usize)>::new();
            for (i, letter_opt) in self.automata.transition_matrix[state].iter().enumerate() {
                if !self.reachability.as_incoming[*to].contains(&i) && to.ne(&i) {
                    continue;
                }

                if let Some(letter) = letter_opt {
                    outcoming.push((word_prefix.clone() + &letter.to_string(), i));
                }
            }

            if outcoming.is_empty() || state.eq(to)
                && self.rng.gen_bool(Self::COMPLETE_WORD_PROBABILITY)
                && !word_prefix.is_empty()
            {
                return word_prefix;
            }

            states_deq.extend(outcoming);
        }

        unreachable!()
    }

    fn mutate(&mut self, words: &mut Vec<String>) {
        while self.rng.gen_bool(Self::MUTATION_PROBABILITY) {
            match self.rng.gen_range(0..Self::MUTATIONS_COUNT) {
                0 => self.swap_words(words),
                1 => self.swap_letters(words),
                2 => self.duplicate_word(words),
                3 => self.duplicate_letter(words),
                4 => self.remove_word(words),
                5 => self.remove_letter(words),
                _ => unreachable!(),
            };
        }
    }

    fn swap_words(&mut self, words: &mut Vec<String>) {
        if words.len() <= 1 {
            return;
        }

        let i = self.choose_word(words);
        let j = self.choose_word(words);

        words.swap(i, j);
    }

    fn swap_letters(&mut self, words: &mut Vec<String>) {
        if words.is_empty() {
            return;
        }

        let i = self.choose_word(words);
        if words[i].len() <= 1 {
            return;
        }

        let j = self.choose_letter(&words[i]);
        let k = self.choose_letter(&words[i]);

        unsafe {
            words[i].as_bytes_mut().swap(j, k);
        }
    }

    fn duplicate_word(&mut self, words: &mut Vec<String>) {
        if words.is_empty() {
            return;
        }

        let i = self.choose_word(words);

        words.insert(i, words[i].to_string());
    }

    fn duplicate_letter(&mut self, words: &mut Vec<String>) {
        if words.is_empty() {
            return;
        }

        let i = self.choose_word(words);
        if words[i].is_empty() {
            return;
        }

        let j = self.choose_letter(&words[i]);
        let letter = words[i].chars().nth(j).unwrap();
        words[i].insert(j, letter);
    }

    fn remove_word(&mut self, words: &mut Vec<String>) {
        if words.is_empty() {
            return;
        }

        let i = self.choose_word(words);

        words.remove(i);
    }

    fn remove_letter(&mut self, words: &mut Vec<String>) {
        if words.is_empty() {
            return;
        }

        let i = self.choose_word(words);
        if words[i].is_empty() {
            return;
        }

        let j = self.choose_letter(&words[i]);
        words[i].remove(j);
    }

    fn choose_word(&mut self, words: &Vec<String>) -> usize {
        debug_assert!(!words.is_empty());

        self.rng.gen_range(0..words.len())
    }

    fn choose_letter(&mut self, word: &String) -> usize {
        debug_assert!(!word.is_empty());

        self.rng.gen_range(0..word.len())
    }

    fn join_words(&self, words: &Vec<String>) -> String {
        let mut result = String::new();

        for word in words.iter() {
            result.push_str(word);
        }

        result
    }
}

#[derive(Debug)]
struct Reachability {
    matrix: Array2<usize>,
    as_outcoming: Vec<Vec<usize>>,
    as_incoming: Vec<HashSet<usize>>,
}

impl Reachability {
    pub fn from_automata(a: &ndfa::Automata) -> Self {
        let matrix = Self::get_matrix(a);
        let as_outcoming = Self::get_outcoming(&matrix);
        let as_incoming = Self::get_incoming(&as_outcoming);

        Self {
            matrix,
            as_outcoming,
            as_incoming,
        }
    }

    fn get_matrix(a: &ndfa::Automata) -> Array2<usize> {
        let mut adjacency_vec = vec![0; a.size * a.size];
        for (i, row) in a.transition_matrix.iter().enumerate() {
            for (j, letter_opt) in row.iter().enumerate() {
                if letter_opt.is_some() {
                    adjacency_vec[i * a.size + j] = 1;
                }
            }
        }

        let adjacency_matrix = Array2::from_shape_vec((a.size, a.size), adjacency_vec).unwrap();

        let mut reachability_matrix = adjacency_matrix.clone();
        let mut composition_matrix = adjacency_matrix.clone();
        for _ in 0..a.size - 1 {
            composition_matrix = composition_matrix.dot(&adjacency_matrix);
            reachability_matrix += &composition_matrix;
        }

        reachability_matrix
    }

    fn get_outcoming(matrix: &Array2<usize>) -> Vec<Vec<usize>> {
        let mut outcoming = vec![Vec::<usize>::new(); matrix.dim().0];

        for ((i, j), paths_count) in matrix.indexed_iter() {
            if paths_count.gt(&0) {
                outcoming[i].push(j);
            }
        }

        outcoming
    }

    fn get_incoming(outcoming: &Vec<Vec<usize>>) -> Vec<HashSet<usize>> {
        let mut incoming = vec![HashSet::<usize>::new(); outcoming.len()];

        for (i, row) in outcoming.iter().enumerate() {
            for j in row.iter() {
                incoming[*j].insert(i);
            }
        }

        incoming
    }
}
