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
    let last = r.chars().last().clone();
    let mut stream = r.chars().peekable();

    if stream.next() != Some('^') {
        return Err(ERR_INVALID_BEGIN.to_string());
    }

    if stream.next_back() == None || last != Some('$') {
        return Err(ERR_INVALID_END.to_string());
    }

    let tokens = parse_regex(&mut stream)?;

    Ok(tokens)
}

fn parse_regex(stream: &mut Peekable<Chars<'_>>) -> Result<Vec<Token>, String> {
    let mut regex = "".to_string();
    let mut tokens: Vec<Token> = vec![];

    while stream.peek() != None {
        let ch = stream.peek().unwrap();
        match ch.clone() {
            '(' => {
                if !regex.is_empty() {
                    tokens.push(Token::Regex(regex));
                    regex = "".to_string();
                }

                let extracted = extract(stream)?;

                match extracted {
                    Token::Lookahead(s) => {
                        let mut tmp = parse_lookahead(&mut s.chars().peekable())?;
                        tokens.push(Token::OpenBracket);
                        tokens.append(&mut tmp);
                        tokens.push(Token::CloseBracket);
                    }
                    Token::Regex(s) => {
                        let mut tmp = parse_regex(&mut s.chars().peekable())?;

                        tokens.push(Token::OpenBracket);
                        tokens.append(&mut tmp);
                        tokens.push(Token::CloseBracket);
                    }
                    _ => {}
                }
                continue;
            }
            ')' => {
                return Err(ERR_INVALID_BRACKETS_SEQUENCE.to_string());
            }
            '|' => {
                if !regex.is_empty() {
                    tokens.push(Token::Regex(regex));
                    regex = "".to_string();
                }
                tokens.push(Token::Binary("|".to_string()));
            }
            '*' => {
                if !regex.is_empty() {
                    tokens.push(Token::Regex(regex));
                    regex = "".to_string();
                }
                tokens.push(Token::Unary("*".to_string()));
            }
            _ => {
                regex.push(ch.clone());
            }
        };
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

    while counter != 0 && stream.peek() != None {
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

// <lookahead> ::= <lookahead><binary><lookahead> |
//     (<lookahead>) |
//     <lookahead><unary> |
//     <symbol> | ε

// TODO: протестировать с '$' в конце lookahead
fn parse_lookahead(stream: &mut Peekable<Chars<'_>>) -> Result<Vec<Token>, String> {
    let mut tokens = vec![];
    let mut lookahead = "".to_string();

    while stream.peek() != None {
        let ch = stream.peek().unwrap();
        match ch.clone() {
            '(' => {
                if !lookahead.is_empty() {
                    tokens.push(Token::Regex(lookahead));
                    lookahead = "".to_string();
                }

                let extracted = extract(stream)?;

                match extracted {
                    Token::Regex(s) => {
                        let mut tmp = parse_lookahead(&mut s.chars().peekable())?;
                        tokens.push(Token::OpenBracket);
                        tokens.append(&mut tmp);
                        tokens.push(Token::CloseBracket);
                    }
                    _ => {
                        return Err(ERR_INVALID_LOOKAHEAD.to_string());
                    }
                };

                continue;
            }
            '|' => {
                if !lookahead.is_empty() {
                    tokens.push(Token::Lookahead(lookahead));
                    lookahead = "".to_string();
                }
                tokens.push(Token::Binary("|".to_string()))
            }
            ')' => {
                return Err(ERR_INVALID_BRACKETS_SEQUENCE.to_string());
            }
            '*' => {
                if !lookahead.is_empty() {
                    tokens.push(Token::Lookahead(lookahead));
                    lookahead = "".to_string();
                }

                tokens.push(Token::Unary("*".to_string()))
            }

            _ => lookahead.push(ch.clone()),
        }
        stream.next();
    }

    if !lookahead.is_empty() {
        tokens.push(Token::Lookahead(lookahead));
    }

    Ok(tokens)
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
        assert!(res.is_ok_and(|tokens| tokens.len() == 4
            && matches!(tokens[0], Token::Regex(..))
            && matches!(tokens[1], Token::OpenBracket)
            && matches!(tokens[2], Token::Regex(..))
            && matches!(tokens[3], Token::CloseBracket)));
    }

    #[test]
    fn valid_brackets_hard() {
        let regex1 = "^(te|st)*(abc)$".to_string();

        let res = parse(&regex1);
        assert!(res.is_ok_and(|tokens| tokens.len() == 9
            && matches!(tokens[0], Token::OpenBracket)
            && matches!(tokens[1], Token::Regex(..))
            && matches!(tokens[2], Token::Binary(..))
            && matches!(tokens[3], Token::Regex(..))
            && matches!(tokens[4], Token::CloseBracket)
            && matches!(tokens[5], Token::Unary(..))
            && matches!(tokens[6], Token::OpenBracket)
            && matches!(tokens[7], Token::Regex(..))
            && matches!(tokens[8], Token::CloseBracket)));

        let regex2 = "^(test|(abc)*)*(abc)$".to_string();
        let res = parse(&regex2);
        assert!(res.is_ok_and(|tokens| tokens.len() == 12
            && matches!(tokens[0], Token::OpenBracket)
            && matches!(tokens[1], Token::Regex(..))
            && matches!(tokens[2], Token::Binary(..))
            && matches!(tokens[3], Token::OpenBracket)
            && matches!(tokens[4], Token::Regex(..))
            && matches!(tokens[5], Token::CloseBracket)
            && matches!(tokens[6], Token::Unary(..))
            && matches!(tokens[7], Token::CloseBracket)
            && matches!(tokens[8], Token::Unary(..))
            && matches!(tokens[9], Token::OpenBracket)
            && matches!(tokens[10], Token::Regex(..))
            && matches!(tokens[11], Token::CloseBracket)));
    }

    #[test]
    fn lookahead_simple() {
        let regex = "^a(?=abc)$";

        assert!(parse(regex).is_ok_and(|tokens| tokens.len() == 4
            && matches!(tokens[0], Token::Regex(..))
            && matches!(tokens[1], Token::OpenBracket)
            && matches!(tokens[2], Token::Lookahead(..))
            && matches!(tokens[3], Token::CloseBracket)));

        let regex = "^a(?=(abc|kd))abc$";

        assert!(parse(regex).is_ok_and(|tokens| tokens.len() == 9
            && matches!(tokens[0], Token::Regex(..))
            && matches!(tokens[1], Token::OpenBracket)
            && matches!(tokens[2], Token::OpenBracket)
            && matches!(tokens[3], Token::Lookahead(..))
            && matches!(tokens[4], Token::Binary(..))
            && matches!(tokens[5], Token::Lookahead(..))
            && matches!(tokens[6], Token::CloseBracket)
            && matches!(tokens[7], Token::CloseBracket)
            && matches!(tokens[8], Token::Regex(..))));
    }

	#[test]
    fn lookahead_hard() {
        let regex = "^a(?=((ab)*c|kd))abc$";

        assert!(parse(regex).is_ok_and(|tokens| tokens.len() == 13
            && matches!(tokens[0], Token::Regex(..))
            && matches!(tokens[1], Token::OpenBracket)
            && matches!(tokens[2], Token::OpenBracket)
            && matches!(tokens[3], Token::OpenBracket)
            && matches!(tokens[4], Token::Lookahead(..))
            && matches!(tokens[5], Token::CloseBracket)
            && matches!(tokens[6], Token::Unary(..))
            && matches!(tokens[7], Token::Lookahead(..))
            && matches!(tokens[8], Token::Binary(..))
            && matches!(tokens[9], Token::Lookahead(..))
            && matches!(tokens[10], Token::CloseBracket)
            && matches!(tokens[11], Token::CloseBracket)
            && matches!(tokens[12], Token::Regex(..))));
    }
}
