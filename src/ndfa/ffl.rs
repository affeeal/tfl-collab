// first-, follow-, last-elements (ffl)

use crate::ndfa::ast::Tree;
use crate::ndfa::ast::Union;
use crate::ndfa::ast::Concat;
use crate::ndfa::ast::Basic;
use crate::ndfa::ast::Atomic;
use crate::ndfa::ast::LinearizedSymbol;

// First-set

pub fn get_first_set(tree: &Tree) -> Vec<LinearizedSymbol> {
    get_first_of_union(&tree.root)
}

fn get_first_of_union(union: &Union) -> Vec<LinearizedSymbol> {
    let mut first_set = Vec::new();

    for concat in &union.concats {
        first_set.extend(get_first_of_concat(concat));
    }

    first_set
}

fn get_first_of_concat(concat: &Concat) -> Vec<LinearizedSymbol> {
    let mut first_set = Vec::new();

    for basic in &concat.basics {
        first_set.extend(get_first_of_basic(basic));

        if !basic.is_iter {
            break;
        }
    }

    first_set
}

fn get_first_of_basic(basic: &Basic) -> Vec<LinearizedSymbol> {
    get_first_of_atomic(&basic.atomic)
}

fn get_first_of_atomic(atomic: &Atomic) -> Vec<LinearizedSymbol> {
    match atomic {
        Atomic::LinearizedSymbol(linearized_symbol) => vec![linearized_symbol.clone()],
        Atomic::Union(union) => get_first_of_union(union),
    }
}

// Last-set

pub fn get_last_set(tree: &Tree) -> Vec<LinearizedSymbol> {
    get_last_of_union(&tree.root)
}

fn get_last_of_union(union: &Union) -> Vec<LinearizedSymbol> {
    let mut last_set = Vec::new();

    for concat in &union.concats {
        last_set.extend(get_last_of_concat(concat));
    }

    last_set
}

fn get_last_of_concat(concat: &Concat) -> Vec<LinearizedSymbol> {
    let mut last_set = Vec::new();

    for basic in concat.basics.iter().rev() {
        last_set.extend(get_last_of_basic(basic));

        if !basic.is_iter {
            break;
        }
    }

    last_set
}

fn get_last_of_basic(basic: &Basic) -> Vec<LinearizedSymbol> {
    get_last_of_atomic(&basic.atomic)
}

fn get_last_of_atomic(atomic: &Atomic) -> Vec<LinearizedSymbol> {
    match atomic {
        Atomic::LinearizedSymbol(linearized_symbol) => vec![linearized_symbol.clone()],
        Atomic::Union(union) => get_last_of_union(union),
    }
}

// Cartesian product of sets

fn get_cartesian_product(
    set1: &Vec<LinearizedSymbol>,
    set2: &Vec<LinearizedSymbol>,
) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
    let mut result = Vec::new();

    for s1 in set1 {
        for s2 in set2 {
            result.push((s1.clone(), s2.clone()));
        }
    }

    result
}

// Follow-set

pub fn get_follow_set(tree: &Tree) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
    get_follow_of_union(&tree.root)
}

fn get_follow_of_union(union: &Union) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
    let mut follow_set = Vec::new();

    for concat in &union.concats {
        follow_set.extend(get_follow_of_concat(concat));
    }

    follow_set
}

fn get_follow_of_concat(concat: &Concat) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
    let mut follow_set = Vec::new();

    let basics = &concat.basics;

    for basic in basics {
        follow_set.extend(get_follow_of_basic(basic));
    }

    for i in 0..basics.len() - 1 {
        for j in (i + 1)..basics.len() {
            follow_set.extend(get_cartesian_product(
                &get_last_of_basic(&basics[i]),
                &get_first_of_basic(&basics[j]),
            ));

            if !basics[j].is_iter {
                break;
            }
        }
    }

    follow_set
}

fn get_follow_of_basic(basic: &Basic) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
    let atomic = &basic.atomic;

    let mut follow_set = get_follow_of_atomic(atomic);

    if basic.is_iter {
        follow_set.extend(get_cartesian_product(
            &get_last_of_atomic(atomic),
            &get_first_of_atomic(atomic),
        ));
    }

    follow_set
}

fn get_follow_of_atomic(atomic_exp: &Atomic) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
    match atomic_exp {
        Atomic::LinearizedSymbol(_linearized_symbol) => Vec::new(),
        Atomic::Union(union) => get_follow_of_union(union),
    }
}

pub fn does_epsilon_satisfy(tree: &Tree) -> bool {
    does_epsilon_satisfy_union(&tree.root)
}

fn does_epsilon_satisfy_union(union: &Union) -> bool {
    for concat in &union.concats {
        if does_epsilon_satisfy_concat(&concat) {
            return true;
        }
    }

    false
}

fn does_epsilon_satisfy_concat(concat: &Concat) -> bool {
    for basic in &concat.basics {
        if !does_epsilon_satisfy_basic(basic) {
            return false;
        }
    }

    true
}

fn does_epsilon_satisfy_basic(basic: &Basic) -> bool {
    basic.is_iter || does_epsilon_satisfy_atomic(&basic.atomic)
}

fn does_epsilon_satisfy_atomic(atomic: &Atomic) -> bool {
    match atomic {
        Atomic::LinearizedSymbol(_linearized_symbol) => false,
        Atomic::Union(union) => does_epsilon_satisfy_union(union),
    }
}

