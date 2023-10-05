/*
Грамматика

<init> ::= ∧<regex>$

<regex> ::= <regex><binary><regex> |
    (<regex>) |
    <regex><unary> |
    <symbol> |
    (?=<lookahead>$?) | ε

<lookahead> ::= <lookahead><binary><lookahead> |
    (<lookahead>) |
    <lookahead><unary> |
    <symbol> | ε

<binary> ::= '|' | ε

<unary> ::= *
*/

use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub(crate) enum Token {
    Regex(String),
    Binary(String),
    Unary(String),
    Lookahead(String),
    CloseBracket,
    OpenBracket,
}

// Errors
const ERR_INVALID_BEGIN: &str = "regex must be begin with '^'";
const ERR_INVALID_END: &str = "regex must be end with '$'";
const ERR_INVALID_BRACKETS_SEQUENCE: &str = "invalid brackets sequence";
const ERR_INVALID_LOOKAHEAD: &str = "invalid lookahead operation";

pub fn parse(r: &str) -> Result<Vec<Token>, String> {
    let mut stream = r.chars().peekable();
    let last = r.chars().last().clone();

    if stream.next() != Some('^') {
        return Err(ERR_INVALID_BEGIN.to_string());
    }

    if last == None || last != Some('$') {
        return Err(ERR_INVALID_END.to_string());
    }

    let tokens = parse_regex(&mut stream)?;

    Ok(tokens)
}

fn parse_regex(stream: &mut Peekable<Chars<'_>>) -> Result<Vec<Token>, String> {
    let mut regex = "".to_string();
    let mut tokens: Vec<Token> = vec![];
    // TODO: добавить проверку на сивмол регуляри
    while stream.peek() != None && stream.peek() != Some(&'$') {
        let ch = stream.peek().unwrap();
        if ch.eq(&'(') {
            tokens.push(Token::Regex(regex.clone()));
            let extracted = extract(stream)?;

            match extracted {
                Token::Lookahead(s) => tokens.push(Token::Lookahead(s.clone())),
                Token::Regex(s) => {
                    let mut tmp = parse_regex(&mut s.chars().peekable())?;

                    tokens.push(Token::OpenBracket);
                    tokens.append(&mut tmp);
                    tokens.push(Token::CloseBracket);
                }
                _ => {}
            }
            regex = "".to_string();
        } else if ch.eq(&')') {
            println!("1234");
            return Err(ERR_INVALID_BRACKETS_SEQUENCE.to_string());
        } else if ch.eq(&'|') {
            tokens.push(Token::Regex(regex.clone()));
            tokens.push(Token::Binary("|".to_string()));
            regex = "".to_string();
        } else if ch.eq(&'*') {
            tokens.push(Token::Regex(regex.clone()));
            tokens.push(Token::Unary("*".to_string()));
            regex = "".to_string();
        } else {
            regex.push(ch.clone());
        }
        stream.next();
    }

    if !regex.is_empty() {
        tokens.push(Token::Regex(regex.clone()));
    }

    return Ok(tokens);
}

fn extract(stream: &mut Peekable<Chars<'_>>) -> Result<Token, String> {
    let token_type: Token;
    let mut counter = 1;
    let mut extracted_value = "".to_string();
    stream.next();

    match stream.peek() {
        Some(t) => {
            if t == &'?' {
                token_type = Token::Lookahead("".to_string());
                stream.next();
            } else {
                token_type = Token::Regex("".to_string());
            }
        }
        None => return Err(ERR_INVALID_BRACKETS_SEQUENCE.to_string()),
    };

    if matches!(token_type, Token::Lookahead(..)) {
        if stream.peek() != Some(&'=') {
            return Err(ERR_INVALID_LOOKAHEAD.to_string());
        }
        stream.next();
    }

    while counter != 0 && stream.peek() != Some(&'$') {
        if stream.peek() == Some(&'(') {
            counter += 1;
        } else if stream.peek() == Some(&')') {
            counter -= 1;
        }
        extracted_value.push(stream.peek().unwrap().clone());
        stream.next();
    }

    if counter != 0 {
        return Err(ERR_INVALID_BRACKETS_SEQUENCE.to_string());
    };

    extracted_value.remove(extracted_value.len() - 1);
    match token_type {
        Token::Lookahead(_) => Ok(Token::Lookahead(extracted_value.to_string())),
        _ => Ok(Token::Regex(extracted_value.to_string())),
    }
}

fn parse_lookahead() {
    unimplemented!();
}

#[cfg(test)]
mod tests {

    use super::{parse, Token};

    #[test]
    fn invalid_begin() {
        let regex = "test$".to_string();
        let res = parse(&regex);
        assert!(res.is_err());
    }

    #[test]
    fn invalid_end() {
        let regex = "^test".to_string();
        let res = parse(&regex);
        assert!(res.is_err());
    }

    #[test]
    fn simple_string() {
        let regex = "^test$".to_string();
        let res = parse(&regex);
        println!("{:?}", res);
        assert!(res.is_ok());
        let tokens = res.unwrap();
        assert!(tokens.len() == 1);
        assert!(matches!(tokens[0], Token::Regex { .. }));
    }

    #[test]
    fn simple_unary() {
        let regex = "^test*$".to_string();
        let res = parse(&regex);
        assert!(res.is_ok_and(|tokens| tokens.len() == 2
            && matches!(tokens[0], Token::Regex { .. })
            && matches!(tokens[1], Token::Unary { .. })));
    }

    #[test]
    fn simple_binary() {
        let regex = "^test|iu9$".to_string();
        let res = parse(&regex);
        assert!(res.is_ok_and(|tokens| tokens.len() == 3
            && matches!(tokens[0], Token::Regex { .. })
            && matches!(tokens[1], Token::Binary { .. })
            && matches!(tokens[2], Token::Regex { .. })));
    }

    #[test]
    fn invalid_brackets() {
        let regex1 = "^test((abc)$".to_string();
        let regex2 = "^testabc)$".to_string();
        let regex3 = "^testabc($".to_string();

        let res = parse(&regex1);
        assert!(res.is_err());

        let res = parse(&regex2);
        assert!(res.is_err());

        let res = parse(&regex3);
        assert!(res.is_err());
    }

    #[test]
    fn valid_brackets() {
        let regex1 = "^test(abc)$".to_string();

        let res = parse(&regex1);
        println!("{:?}", res);
        assert!(res.is_ok_and(|tokens| tokens.len() == 4
            && matches!(tokens[0], Token::Regex(..))
            && matches!(tokens[1], Token::OpenBracket)
            && matches!(tokens[2], Token::Regex(..))
            && matches!(tokens[3], Token::CloseBracket)));
    }
}
