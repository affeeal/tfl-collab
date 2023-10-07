use crate::ast::AtomicExpr;
use crate::ast::BasicExpr;
use crate::ast::ConcatExpr;
use crate::ast::LinearizedSymbol;
use crate::ast::UnionExpr;

pub fn get_first_set(union_expr: &UnionExpr) -> Vec<LinearizedSymbol> {
    get_first_of_union(union_expr)
}

fn get_first_of_union(union_expr: &UnionExpr) -> Vec<LinearizedSymbol> {
    let mut first_set = Vec::new();

    for concat_expr in &union_expr.concat_exprs {
        first_set.extend(get_first_of_concat(concat_expr));
    }

    first_set
}

fn get_first_of_concat(concat_expr: &ConcatExpr) -> Vec<LinearizedSymbol> {
    let mut first_set = Vec::new();

    for basic_expr in &concat_expr.basic_exprs {
        first_set.extend(get_first_of_basic(basic_expr));

        if !basic_expr.is_iter {
            break;
        }
    }

    first_set
}

fn get_first_of_basic(basic_expr: &BasicExpr) -> Vec<LinearizedSymbol> {
    get_first_of_atomic(&basic_expr.atomic_expr)
}

fn get_first_of_atomic(atomic_expr: &AtomicExpr) -> Vec<LinearizedSymbol> {
    match atomic_expr {
        AtomicExpr::LinearizedSymbol(linearized_symbol) => vec![linearized_symbol.clone()],
        AtomicExpr::UnionExpr(union_expr) => get_first_of_union(union_expr),
    }
}

pub fn get_last_set(union_expr: &UnionExpr) -> Vec<LinearizedSymbol> {
    get_last_of_union(union_expr)
}

fn get_last_of_union(union_expr: &UnionExpr) -> Vec<LinearizedSymbol> {
    let mut last_set = Vec::new();

    for concat_expr in &union_expr.concat_exprs {
        last_set.extend(get_last_of_concat(concat_expr));
    }

    last_set
}

fn get_last_of_concat(concat_expr: &ConcatExpr) -> Vec<LinearizedSymbol> {
    let mut last_set = Vec::new();

    for basic_expr in concat_expr.basic_exprs.iter().rev() {
        last_set.extend(get_last_of_basic(basic_expr));

        if !basic_expr.is_iter {
            break;
        }
    }

    last_set
}

fn get_last_of_basic(basic_expr: &BasicExpr) -> Vec<LinearizedSymbol> {
    get_last_of_atomic(&basic_expr.atomic_expr)
}

fn get_last_of_atomic(atomic_expr: &AtomicExpr) -> Vec<LinearizedSymbol> {
    match atomic_expr {
        AtomicExpr::LinearizedSymbol(linearized_symbol) => vec![linearized_symbol.clone()],
        AtomicExpr::UnionExpr(union_expr) => get_last_of_union(union_expr),
    }
}

fn get_cartesian_product(
    first_set: &Vec<LinearizedSymbol>,
    second_set: &Vec<LinearizedSymbol>,
) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
    let mut result = Vec::new();

    for s1 in first_set {
        for s2 in second_set {
            result.push((s1.clone(), s2.clone()));
        }
    }

    result
}

pub fn get_follow_set(union_expr: &UnionExpr) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
    get_follow_of_union(union_expr)
}

fn get_follow_of_union(union_expr: &UnionExpr) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
    let mut follow_set = Vec::new();

    for concat_expr in &union_expr.concat_exprs {
        follow_set.extend(get_follow_of_concat(concat_expr));
    }

    follow_set
}

fn get_follow_of_concat(concat_expr: &ConcatExpr) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
    let mut follow_set = Vec::new();

    let basic_exprs = &concat_expr.basic_exprs;

    for basic_expr in basic_exprs {
        follow_set.extend(get_follow_of_basic(basic_expr));
    }

    for i in 0..basic_exprs.len() - 1 {
        follow_set.extend(get_cartesian_product(
            &get_last_of_basic(&basic_exprs[i]),
            &get_first_of_basic(&basic_exprs[i + 1]),
        ));
    }

    follow_set
}

fn get_follow_of_basic(basic_expr: &BasicExpr) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
    let atomic_expr = &basic_expr.atomic_expr;

    let mut follow_set = get_follow_of_atomic(atomic_expr);

    if basic_expr.is_iter {
        follow_set.extend(get_cartesian_product(
            &get_last_of_atomic(atomic_expr),
            &get_first_of_atomic(atomic_expr),
        ));
    }

    follow_set
}

fn get_follow_of_atomic(atomic_exp: &AtomicExpr) -> Vec<(LinearizedSymbol, LinearizedSymbol)> {
    match atomic_exp {
        AtomicExpr::LinearizedSymbol(_) => Vec::new(),
        AtomicExpr::UnionExpr(union_expr) => get_follow_of_union(union_expr),
    }
}
