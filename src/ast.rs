use std::iter::Peekable;
use std::str::Chars;

/*
 * <UnionExpr> ::= <ConcatExpr> ('|' <ConcatExpr>)*
 *
 * <ConcatExpr> ::= <BasicExpr> (<BasicExpr>)*
 *
 * <BasicExpr> ::= <AtomicExpr> ('*')?
 *
 * <AtomicExpr> ::= CHAR | '(' <UnionExpr> ')'
*/

#[derive(Debug)]
pub struct UnionExpr {
    pub concat_exprs: Vec<ConcatExpr>,
}

#[derive(Debug)]
pub struct ConcatExpr {
    pub basic_exprs: Vec<BasicExpr>,
}

#[derive(Debug)]
pub struct BasicExpr {
    pub atomic_expr: AtomicExpr,
    pub is_iter: bool,
}

#[derive(Debug)]
pub enum AtomicExpr {
    LinearizedSymbol(LinearizedSymbol),
    UnionExpr(UnionExpr),
}

#[derive(Debug, Copy, Clone)]
pub struct LinearizedSymbol {
    pub letter: char,
    pub index: usize,
}

static mut LINEARIZED_INDEX: usize = 0;

pub fn parse(regex: &str) -> (usize, UnionExpr) {
    unsafe {
        LINEARIZED_INDEX = 0;
    }

    let mut stream = regex.chars().peekable();

    let ast_root = parse_union_expr(&mut stream);

    unsafe { (LINEARIZED_INDEX, ast_root) }
}

// <UnionExpr> ::= <ConcatExpr> ('|' <ConcatExpr>)*
fn parse_union_expr(stream: &mut Peekable<Chars<'_>>) -> UnionExpr {
    let mut union_expr = UnionExpr {
        concat_exprs: vec![parse_concat_expr(stream)],
    };

    while stream.peek() == Some(&'|') {
        stream.next();
        union_expr.concat_exprs.push(parse_concat_expr(stream));
    }

    union_expr
}

// <ConcatExpr> ::= <BasicExpr> (<BasicExpr>)*
fn parse_concat_expr(stream: &mut Peekable<Chars<'_>>) -> ConcatExpr {
    let mut concat_expr = ConcatExpr {
        basic_exprs: vec![parse_basic_expr(stream)],
    };

    while let Some(ch) = stream.peek() {
        if !ch.is_alphabetic() && ch != &'(' {
            break;
        }

        concat_expr.basic_exprs.push(parse_basic_expr(stream));
    }

    concat_expr
}

// <BasicExpr> ::= <AtomicExpr> ('*')?
fn parse_basic_expr(stream: &mut Peekable<Chars<'_>>) -> BasicExpr {
    let mut basic_expr = BasicExpr {
        atomic_expr: parse_atomic_expr(stream),
        is_iter: false,
    };

    if stream.peek() == Some(&'*') {
        stream.next();
        basic_expr.is_iter = true;
    }

    basic_expr
}

// <AtomicExpr> ::= <CHAR> | '(' <UnionExpr> ')'
fn parse_atomic_expr(stream: &mut Peekable<Chars<'_>>) -> AtomicExpr {
    let letter = stream.next().unwrap();
    if letter == '(' {
        let atomic_expr = AtomicExpr::UnionExpr(parse_union_expr(stream));

        stream.next(); // drops ')'
        return atomic_expr;
    }

    unsafe {
        LINEARIZED_INDEX += 1;
        AtomicExpr::LinearizedSymbol(LinearizedSymbol {
            letter,
            index: LINEARIZED_INDEX,
        })
    }
}
