use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub struct Tree {
    pub root: Union,
    pub linearized_symbols: usize,
}

/*
 * <Union> ::= <Concat> ('|' <Concat>)*
 *
 * <Concat> ::= <Basic> (<Basic>)*
 *
 * <Basic> ::= <Atomic> ('*')?
 *
 * <Atomic> ::= CHAR | '^' | '$' | '.' | '(' <Union> ')'
*/

#[derive(Debug)]
pub struct Union {
    pub concats: Vec<Concat>,
}

#[derive(Debug)]
pub struct Concat {
    pub basics: Vec<Basic>,
}

#[derive(Debug)]
pub struct Basic {
    pub atomic: Atomic,
    pub is_iter: bool,
}

#[derive(Debug)]
pub enum Atomic {
    LinearizedSymbol(LinearizedSymbol),
    Union(Union),
}

#[derive(Debug, Copy, Clone)]
pub struct LinearizedSymbol {
    pub symbol: char,
    pub index: usize,
}

impl Tree {
    pub fn from_regex(regex: &str) -> Self {
        assert!(!regex.is_empty());

        let mut tree = Self::default();
        tree.initialize(regex);

        tree
    }

    fn default() -> Self {
        Self {
            linearized_symbols: 0,
            root: Union::default(),
        }
    }

    fn initialize(&mut self, regex: &str) {
        let mut stream = regex.chars().peekable();

        self.root = self.parse_union(&mut stream);
    }

    fn parse_union(&mut self, stream: &mut Peekable<Chars<'_>>) -> Union {
        let mut union = Union::new(vec![self.parse_concat(stream)]);

        while stream.peek() == Some(&'|') {
            stream.next();
            union.concats.push(self.parse_concat(stream));
        }

        union
    }

    fn parse_concat(&mut self, stream: &mut Peekable<Chars<'_>>) -> Concat {
        let mut concat = Concat::new(vec![self.parse_basic(stream)]);

        while let Some(symbol) = stream.peek() {
            if !Self::is_atomic_start(*symbol) {
                break;
            }

            concat.basics.push(self.parse_basic(stream));
        }

        concat
    }

    fn is_atomic_start(symbol: char) -> bool {
        symbol.is_alphabetic()
            || symbol == '^'
            || symbol == '$'
            || symbol == '.'
            || symbol == '('
    }

    fn parse_basic(&mut self, stream: &mut Peekable<Chars<'_>>) -> Basic {
        let mut basic = Basic::new(self.parse_atomic(stream), false);

        if stream.peek() == Some(&'*') {
            stream.next();
            basic.is_iter = true;
        }

        basic
    }

    fn parse_atomic(&mut self, stream: &mut Peekable<Chars<'_>>) -> Atomic {
        let symbol = stream.next().unwrap();

        if symbol == '(' {
            let atomic = Atomic::Union(self.parse_union(stream));

            assert_eq!(stream.next(), Some(')'));
            return atomic;
        }

        self.linearized_symbols += 1;
        Atomic::LinearizedSymbol(LinearizedSymbol::new(symbol, self.linearized_symbols))
    }

    // First-set
    
    pub fn get_first_set(&self) -> Vec<LinearizedSymbol> {
        Self::get_first_of_union(&self.root)
    }
    
    fn get_first_of_union(union: &Union) -> Vec<LinearizedSymbol> {
        let mut first_set = Vec::new();
    
        for concat in &union.concats {
            first_set.extend(Self::get_first_of_concat(concat));
        }
    
        first_set
    }
    
    fn get_first_of_concat(concat: &Concat) -> Vec<LinearizedSymbol> {
        let mut first_set = Vec::new();
    
        for basic in &concat.basics {
            first_set.extend(Self::get_first_of_basic(basic));
    
            if !Self::does_epsilon_satisfy_basic(basic) {
                break;
            }
        }
    
        first_set
    }
    
    fn get_first_of_basic(basic: &Basic) -> Vec<LinearizedSymbol> {
        Self::get_first_of_atomic(&basic.atomic)
    }
    
    fn get_first_of_atomic(atomic: &Atomic) -> Vec<LinearizedSymbol> {
        match atomic {
            Atomic::LinearizedSymbol(linearized_symbol) => vec![*linearized_symbol],
            Atomic::Union(union) => Self::get_first_of_union(union),
        }
    }

    // Last-set
    
    pub fn get_last_set(&self) -> Vec<LinearizedSymbol> {
        Self::get_last_of_union(&self.root)
    }
    
    fn get_last_of_union(union: &Union) -> Vec<LinearizedSymbol> {
        let mut last_set = Vec::new();
    
        for concat in &union.concats {
            last_set.extend(Self::get_last_of_concat(concat));
        }
    
        last_set
    }
    
    fn get_last_of_concat(concat: &Concat) -> Vec<LinearizedSymbol> {
        let mut last_set = Vec::new();
    
        for basic in concat.basics.iter().rev() {
            last_set.extend(Self::get_last_of_basic(basic));
    
            if !Self::does_epsilon_satisfy_basic(basic) {
                break;
            }
        }
    
        last_set
    }
    
    fn get_last_of_basic(basic: &Basic) -> Vec<LinearizedSymbol> {
        Self::get_last_of_atomic(&basic.atomic)
    }
    
    fn get_last_of_atomic(atomic: &Atomic) -> Vec<LinearizedSymbol> {
        match atomic {
            Atomic::LinearizedSymbol(linearized_symbol) => vec![*linearized_symbol],
            Atomic::Union(union) => Self::get_last_of_union(union),
        }
    }

    // Follow-set
    
    pub fn get_follow_set(&self) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
        Self::get_follow_of_union(&self.root)
    }
    
    fn get_follow_of_union(union: &Union) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
        let mut follow_set = Vec::new();
    
        for concat in &union.concats {
            follow_set.extend(Self::get_follow_of_concat(concat));
        }
    
        follow_set
    }
    
    fn get_follow_of_concat(concat: &Concat) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
        let mut follow_set = Vec::new();
    
        let basics = &concat.basics;
    
        for basic in basics {
            follow_set.extend(Self::get_follow_of_basic(basic));
        }
    
        for i in 0..basics.len() - 1 {
            for j in (i + 1)..basics.len() {
                follow_set.extend(Self::get_cartesian_product(
                    &Self::get_last_of_basic(&basics[i]),
                    &Self::get_first_of_basic(&basics[j]),
                ));
    
                if !Self::does_epsilon_satisfy_basic(&basics[j]) {
                    break;
                }
            }
        }
    
        follow_set
    }

    fn get_cartesian_product(
        first_set: &Vec<LinearizedSymbol>,
        second_set: &Vec<LinearizedSymbol>,
    ) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
        let mut result = Vec::new();
    
        for first_symbol in first_set {
            for second_symbol in second_set {
                result.push((*first_symbol, *second_symbol));
            }
        }
    
        result
    }
    
    fn get_follow_of_basic(basic: &Basic) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
        let atomic = &basic.atomic;
    
        let mut follow_set = Self::get_follow_of_atomic(atomic);
    
        if basic.is_iter {
            follow_set.extend(Self::get_cartesian_product(
                &Self::get_last_of_atomic(atomic),
                &Self::get_first_of_atomic(atomic),
            ));
        }
    
        follow_set
    }
    
    fn get_follow_of_atomic(atomic_exp: &Atomic) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
        match atomic_exp {
            Atomic::LinearizedSymbol(_linearized_symbol) => Vec::new(),
            Atomic::Union(union) => Self::get_follow_of_union(union),
        }
    }
    
    // Epsilon satisfiability
    
    pub fn does_epsilon_satisfy(&self) -> bool {
        Self::does_epsilon_satisfy_union(&self.root)
    }
    
    fn does_epsilon_satisfy_union(union: &Union) -> bool {
        for concat in &union.concats {
            if Self::does_epsilon_satisfy_concat(&concat) {
                return true;
            }
        }
    
        false
    }
    
    fn does_epsilon_satisfy_concat(concat: &Concat) -> bool {
        for basic in &concat.basics {
            if !Self::does_epsilon_satisfy_basic(basic) {
                return false;
            }
        }
    
        true
    }
    
    fn does_epsilon_satisfy_basic(basic: &Basic) -> bool {
        basic.is_iter || Self::does_epsilon_satisfy_atomic(&basic.atomic)
    }
    
    fn does_epsilon_satisfy_atomic(atomic: &Atomic) -> bool {
        match atomic {
            Atomic::LinearizedSymbol(_linearized_symbol) => false,
            Atomic::Union(union) => Self::does_epsilon_satisfy_union(union),
        }
    }
}

impl Union {
    pub fn new(concats: Vec<Concat>) -> Self {
        Self { concats }
    }

    pub fn default() -> Self {
        Self::new(Vec::<Concat>::new())
    }
}

impl Concat {
    pub fn new(basics: Vec<Basic>) -> Self {
        Self { basics }
    }
}

impl Basic {
    pub fn new(atomic: Atomic, is_iter: bool) -> Self {
        Self { atomic, is_iter }
    }
}

impl LinearizedSymbol {
    pub fn new(symbol: char, index: usize) -> Self {
        Self { symbol, index }
    }
}

