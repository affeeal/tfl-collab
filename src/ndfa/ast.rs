// abstract syntax tree (ast)

use std::iter::Peekable;
use std::str::Chars;

pub struct Tree {
    pub root: Union,
    pub linearized_symbols: usize
}

/*
 * <Union> ::= <Concat> ('|' <Concat>)*
 *
 * <Concat> ::= <Basic> (<Basic>)*
 *
 * <Basic> ::= <Atomic> ('*')?
 *
 * <Atomic> ::= CHAR | '$' | '(' <Union> ')'
*/

pub struct Union {
    pub concats: Vec<Concat>,
}

pub struct Concat {
    pub basics: Vec<Basic>,
}

pub struct Basic {
    pub atomic: Atomic,
    pub is_iter: bool,
}

pub enum Atomic {
    LinearizedSymbol(LinearizedSymbol),
    Union(Union),
}

#[derive(Copy, Clone)]
pub struct LinearizedSymbol {
    pub letter: char,
    pub index: usize,
}

static mut LINEARIZED_INDEX: usize = 0;

impl Tree {
    pub fn from_regex(regex: &String) -> Self {
        assert!(!regex.is_empty());

        unsafe {
            LINEARIZED_INDEX = 0;
        }

        let mut stream = regex.chars().peekable();

        let root = parse_union(&mut stream);
        let linearized_symbols;
        unsafe {
            linearized_symbols = LINEARIZED_INDEX;
        }

        Self {
            root,
            linearized_symbols
        }
    }
}

// <Union> ::= <Concat> ('|' <Concat>)*
fn parse_union(stream: &mut Peekable<Chars<'_>>) -> Union {
    let mut union = Union {
        concats: vec![parse_concat(stream)],
    };

    while stream.peek() == Some(&'|') {
        stream.next();
        union.concats.push(parse_concat(stream));
    }

    union
}

// <Concat> ::= <Basic> (<Basic>)*
fn parse_concat(stream: &mut Peekable<Chars<'_>>) -> Concat {
    let mut concat = Concat {
        basics: vec![parse_basic(stream)],
    };

    while let Some(ch) = stream.peek() {
        if !is_atomic_start(ch) {
            break;
        }

        concat.basics.push(parse_basic(stream));
    }

    concat
}

fn is_atomic_start(ch: &char) -> bool {
    ch.is_alphabetic() || ch == &'$' || ch == &'('
}

// <Basic> ::= <Atomic> ('*')?
fn parse_basic(stream: &mut Peekable<Chars<'_>>) -> Basic {
    let mut basic = Basic {
        atomic: parse_atomic(stream),
        is_iter: false,
    };

    if stream.peek() == Some(&'*') {
        stream.next();
        basic.is_iter = true;
    }

    basic
}

// <Atomic> ::= <CHAR> | '$' | '(' <Union> ')'
fn parse_atomic(stream: &mut Peekable<Chars<'_>>) -> Atomic {
    let letter = stream.next().unwrap();

    if letter == '(' {
        let atomic = Atomic::Union(parse_union(stream));

        assert_eq!(stream.next(), Some(')'));
        return atomic;
    }

    unsafe {
        LINEARIZED_INDEX += 1;
        Atomic::LinearizedSymbol(LinearizedSymbol {
            letter,
            index: LINEARIZED_INDEX,
        })
    }
}
