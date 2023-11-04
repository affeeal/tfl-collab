use std::fmt::Display;
use std::iter::Peekable;
use std::str::Chars;
use std::vec;

#[derive(Debug)]
pub enum Token {
    Binary(String),
    Unary(String),
    SymbolSeq(String),
    LookaheadGroup(Vec<Token>),
    LookbehindGroup(Vec<Token>),
    LookaheadEnd,
    StringEnd,
    OpenBracket,
    CloseBracket,
}

pub enum BracketsTokenType {
    Lookahead,
    Lookbehind,
    SimpleSeq,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::LookaheadGroup(l) => {
                let mut r = "".to_string();
                l.iter().for_each(|t| {
                    r += &t.to_string();
                });

                write!(f, "(?={})", r)
            }
            Token::LookbehindGroup(l) => {
                let mut r = "".to_string();
                l.iter().for_each(|t| {
                    r += &t.to_string();
                });

                write!(f, "(?<={})", r)
            }
            Token::LookaheadEnd => write!(f, ""),
            Token::StringEnd => write!(f, "$"),
            Token::OpenBracket => write!(f, "("),
            Token::CloseBracket => write!(f, ")"),
            Token::Binary(s) => write!(f, "{}", s),
            Token::Unary(s) => write!(f, "{}", s),
            Token::SymbolSeq(s) => write!(f, "{}", s),
        }
    }
}

// Errors
static ERR_INVALID_BEGIN: &str = "regex must be begin with '^'";
static ERR_INVALID_END: &str = "regex must be end with '$'";
static ERR_INVALID_BRACKETS_SEQUENCE: &str = "invalid brackets sequence";
static ERR_INVALID_LOOKAHEAD: &str = "invalid lookahead operation";
static ERR_INVALID_OPERATION: &str = "invalid operation";
static ERR_EMPTY_BRACKETS: &str = "empty brackets";

// <init> ::= ∧<regex>$

pub fn parse(r: &str) -> Result<Vec<Token>, String> {
    let last = r.chars().last();
    let mut stream = r.chars().peekable();

    if stream.next() != Some('^') {
        return Err(ERR_INVALID_BEGIN.to_string());
    }

    if stream.next_back().is_none() || last != Some('$') {
        return Err(ERR_INVALID_END.to_string());
    }

    let tokens = parse_regex(&mut stream)?;

    Ok(tokens)
}

/*
<regex> ::= <regex><binary><regex> |
    (<regex>) |
    <regex><unary> |
    <symbol> |
    (?=<lookahead>$?) | ε
*/

fn parse_regex(stream: &mut Peekable<Chars<'_>>) -> Result<Vec<Token>, &'static str> {
    let mut regex = "".to_string();
    let mut tokens: Vec<Token> = vec![];

    while stream.peek().is_some() {
        let ch = stream.peek().unwrap();
        match ch {
            '(' => {
                if !regex.is_empty() {
                    tokens.push(Token::SymbolSeq(regex));
                    regex = "".to_string();
                }

                let (token_type, extracted) = extract(stream)?;

                match token_type {
                    BracketsTokenType::Lookahead => {
                        let tmp = parse_look_operator(&mut extracted.chars().peekable())?;
                        tokens.push(Token::LookaheadGroup(tmp));
                    }

                    BracketsTokenType::Lookbehind => {
                        let tmp = parse_look_operator(&mut extracted.chars().peekable())?;
                        tokens.push(Token::LookbehindGroup(tmp));
                    }

                    BracketsTokenType::SimpleSeq => {
                        let mut tmp = vec![];
                        tmp.push(Token::OpenBracket);
                        tmp.append(&mut parse_regex(&mut extracted.chars().peekable())?);
                        tmp.push(Token::CloseBracket);
                        tmp = simplify_brackets(tmp);
                        if !matches!(tmp[0], Token::OpenBracket) {
                            tmp.insert(0, Token::OpenBracket);
                            tmp.push(Token::CloseBracket);
                            tokens.append(&mut tmp);
                        } else {
                            tokens.append(&mut tmp);
                        }
                    }
                }
                continue;
            }

            ')' => {
                return Err(ERR_INVALID_BRACKETS_SEQUENCE);
            }

            '|' => {
                if !regex.is_empty() {
                    tokens.push(Token::SymbolSeq(regex));
                }

                // ^|a$ is invalid
                if tokens.is_empty() {
                    return Err(ERR_INVALID_OPERATION);
                }

                tokens.push(Token::Binary("|".to_string()));
                stream.next();

                let mut tmp = parse_regex(stream)?;

                // ^a|$ is invalid
                if tmp.is_empty() {
                    return Err(ERR_INVALID_OPERATION);
                }

                tokens.append(&mut tmp);
                return Ok(tokens);
            }

            '*' => {
                if !regex.is_empty() {
                    tokens.push(Token::SymbolSeq(regex));
                    regex = "".to_string();
                }

                if tokens.is_empty() {
                    return Err(ERR_INVALID_OPERATION);
                }

                // ()* is invalid
                if tokens.len() > 2
                    && matches!(tokens[tokens.len() - 1], Token::CloseBracket)
                    && matches!(tokens[tokens.len() - 2], Token::OpenBracket)
                {
                    return Err(ERR_INVALID_OPERATION);
                }

                // (?=...)* is invalid
                if matches!(tokens.last(), Some(Token::LookaheadGroup(..))) {
                    return Err(ERR_INVALID_OPERATION);
                }

                tokens.push(Token::Unary("*".to_string()));
            }
            _ => {
                regex.push(*ch);
            }
        };
        stream.next();
    }

    if !regex.is_empty() {
        tokens.push(Token::SymbolSeq(regex.clone()));
    }

    Ok(tokens)
}

fn simplify_brackets(tokens: Vec<Token>) -> Vec<Token> {
    let mut result = vec![];
    let mut stack = vec![];
    let mut pairs = vec![];
    let mut tmp = vec![];

    for (i, _) in tokens.iter().enumerate() {
        match &tokens[i] {
            Token::OpenBracket => {
                stack.push(i);
            }
            Token::CloseBracket => {
                pairs.push((stack.pop().unwrap(), i));
            }
            _ => {}
        }
    }

    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    for i in 1..pairs.len() {
        let prev = pairs[i - 1];
        let curr = pairs[i];

        if prev.0 == (curr.0 - 1) && prev.1 == (curr.1 + 1) {
            tmp.push(curr.0);
            tmp.push(curr.1);
        }
    }

    for (i, t) in tokens.into_iter().enumerate() {
        if !tmp.contains(&i) {
            result.push(t);
        }
    }

    result
}

fn extract(stream: &mut Peekable<Chars<'_>>) -> Result<(BracketsTokenType, String), &'static str> {
    let mut token_type = BracketsTokenType::SimpleSeq;
    let mut counter = 1;
    let mut extracted_value = "".to_string();
    let mut is_look_op = false;
    stream.next();

    match stream.peek() {
        Some(&'?') => {
            is_look_op = true;
            stream.next();
        }
        None => return Err(ERR_INVALID_BRACKETS_SEQUENCE),
        Some(_) => {}
    };

    if is_look_op {
        match stream.next() {
            Some('=') => {
                token_type = BracketsTokenType::Lookahead;
            }
            Some('<') => {
                if matches!(stream.next(), Some('=')) {
                    token_type = BracketsTokenType::Lookbehind;
                } else {
                    return Err(ERR_INVALID_LOOKAHEAD);
                }
            }
            _ => return Err(ERR_INVALID_LOOKAHEAD),
        }
    }

    while counter != 0 && stream.peek().is_some() {
        if stream.peek() == Some(&'(') {
            counter += 1;
        } else if stream.peek() == Some(&')') {
            counter -= 1;
        }
        extracted_value.push(*stream.peek().unwrap());
        stream.next();
    }

    if counter != 0 {
        return Err(ERR_INVALID_BRACKETS_SEQUENCE);
    };

    extracted_value.remove(extracted_value.len() - 1);

    if extracted_value.is_empty() {
        return Err(ERR_EMPTY_BRACKETS);
    }

    Ok((token_type, extracted_value.to_string()))
}

/*
<lookahead> ::= <lookahead><binary><lookahead> |
    (<lookahead>) |
    <lookahead><unary> |
    <symbol> | ε
*/

fn parse_look_operator(stream: &mut Peekable<Chars<'_>>) -> Result<Vec<Token>, &'static str> {
    let mut tokens = vec![];
    let mut lookahead = "".to_string();

    while stream.peek().is_some() {
        let ch = stream.peek().unwrap();
        match ch {
            '(' => {
                if !lookahead.is_empty() {
                    tokens.push(Token::SymbolSeq(lookahead));
                    lookahead = "".to_string();
                }

                let (token_type, extracted) = extract(stream)?;

                match token_type {
                    BracketsTokenType::SimpleSeq => {
                        let mut tmp = vec![];
                        tmp.push(Token::OpenBracket);
                        tmp.append(&mut parse_look_operator(&mut extracted.chars().peekable())?);
                        tmp.push(Token::CloseBracket);
                        tmp = simplify_brackets(tmp);
                        if !matches!(tmp[0], Token::OpenBracket) {
                            tmp.insert(0, Token::OpenBracket);
                            tmp.push(Token::CloseBracket);
                            tokens.append(&mut tmp);
                        } else {
                            tokens.append(&mut tmp);
                        }
                    }
                    _ => {
                        return Err(ERR_INVALID_LOOKAHEAD);
                    }
                };

                continue;
            }

            '|' => {
                if !lookahead.is_empty() {
                    tokens.push(Token::SymbolSeq(lookahead));
                }

                if tokens.is_empty() {
                    return Err(ERR_INVALID_OPERATION);
                }

                tokens.push(Token::Binary("|".to_string()));
                stream.next();
                let mut tmp = parse_look_operator(stream)?;

                if tmp.is_empty() {
                    return Err(ERR_INVALID_OPERATION);
                }

                tokens.append(&mut tmp);

                return Ok(tokens);
            }

            ')' => {
                return Err(ERR_INVALID_BRACKETS_SEQUENCE);
            }

            '$' => {
                stream.next();

                if stream.peek().is_some() {
                    return Err(ERR_INVALID_LOOKAHEAD);
                }

                if !lookahead.is_empty() {
                    tokens.push(Token::SymbolSeq(lookahead));
                }
                tokens.push(Token::StringEnd);

                return Ok(tokens);
            }

            '*' => {
                if !lookahead.is_empty() {
                    tokens.push(Token::SymbolSeq(lookahead));
                    lookahead = "".to_string();
                }

                if tokens.is_empty() {
                    return Err(ERR_INVALID_OPERATION);
                }

                tokens.push(Token::Unary("*".to_string()))
            }

            _ => lookahead.push(*ch),
        }
        stream.next();
    }

    if !lookahead.is_empty() {
        tokens.push(Token::SymbolSeq(lookahead));
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
        assert!(matches!(tokens[0], Token::SymbolSeq { .. }));
    }

    #[test]
    fn simple_unary() {
        let regex = "^test*$";
        let res = parse(regex);
        assert!(res.is_ok_and(|tokens| tokens.len() == 2
            && matches!(tokens[0], Token::SymbolSeq { .. })
            && matches!(tokens[1], Token::Unary { .. })));
    }

    #[test]
    fn simple_binary() {
        let regex = "^test|iu9$";
        let res = parse(regex);
        assert!(res.is_ok_and(|tokens| tokens.len() == 3
            && matches!(tokens[0], Token::SymbolSeq { .. })
            && matches!(tokens[1], Token::Binary { .. })
            && matches!(tokens[2], Token::SymbolSeq { .. })));
    }

    #[test]
    fn invalid_binary() {
        let regex = "^5|$";
        assert!(parse(regex).is_err());

        let regex = "^|6$";
        assert!(parse(regex).is_err());

        let regex = "^|$";
        assert!(parse(regex).is_err());
    }

    #[test]
    fn invalid_unary() {
        let regex = "^*abcd$";
        assert!(parse(regex).is_err());

        let regex = "^ab()*cd$";
        assert!(parse(regex).is_err());

        let regex = "^ab(())*cd$";

        assert!(parse(regex).is_err());
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
        assert!(res.is_ok());

        let tokens = res.unwrap();
        assert_eq!(tokens.len(), 4);

        assert!(matches!(tokens[0], Token::SymbolSeq(..)));
        assert!(matches!(tokens[1], Token::OpenBracket));
        assert!(matches!(tokens[2], Token::SymbolSeq(..)));
        assert!(matches!(tokens[3], Token::CloseBracket));
    }

    #[test]
    fn valid_brackets_hard() {
        let regex = "^(te|st)*(abc)$".to_string();

        let res = parse(&regex);

        assert!(res.is_ok_and(|tokens| tokens.len() == 9
            && matches!(tokens[0], Token::OpenBracket)
            && matches!(tokens[1], Token::SymbolSeq(..))
            && matches!(tokens[2], Token::Binary(..))
            && matches!(tokens[3], Token::SymbolSeq(..))
            && matches!(tokens[4], Token::CloseBracket)
            && matches!(tokens[5], Token::Unary(..))
            && matches!(tokens[6], Token::OpenBracket)
            && matches!(tokens[7], Token::SymbolSeq(..))
            && matches!(tokens[8], Token::CloseBracket)));

        let regex = "^(test|(abc)*)*(abc)$".to_string();

        let res = parse(&regex);
        assert!(res.is_ok_and(|tokens| tokens.len() == 12
            && matches!(tokens[0], Token::OpenBracket)
            && matches!(tokens[1], Token::SymbolSeq(..))
            && matches!(tokens[2], Token::Binary(..))
            && matches!(tokens[3], Token::OpenBracket)
            && matches!(tokens[4], Token::SymbolSeq(..))
            && matches!(tokens[5], Token::CloseBracket)
            && matches!(tokens[6], Token::Unary(..))
            && matches!(tokens[7], Token::CloseBracket)
            && matches!(tokens[8], Token::Unary(..))
            && matches!(tokens[9], Token::OpenBracket)
            && matches!(tokens[10], Token::SymbolSeq(..))
            && matches!(tokens[11], Token::CloseBracket)));

        let regex = "^(((test)))$";
        let res = parse(regex);

        assert!(res.is_ok());

        let res = res.unwrap();

        assert_eq!(res.len(), 3);
        assert!(matches!(res[0], Token::OpenBracket));
        assert!(matches!(res[1], Token::SymbolSeq(..)));
        assert!(matches!(res[2], Token::CloseBracket));

        let regex = "^(((test|((abc))*)))*(((abc)))$".to_string();

        let res = parse(&regex);
        assert!(res.is_ok_and(|tokens| tokens.len() == 12
            && matches!(tokens[0], Token::OpenBracket)
            && matches!(tokens[1], Token::SymbolSeq(..))
            && matches!(tokens[2], Token::Binary(..))
            && matches!(tokens[3], Token::OpenBracket)
            && matches!(tokens[4], Token::SymbolSeq(..))
            && matches!(tokens[5], Token::CloseBracket)
            && matches!(tokens[6], Token::Unary(..))
            && matches!(tokens[7], Token::CloseBracket)
            && matches!(tokens[8], Token::Unary(..))
            && matches!(tokens[9], Token::OpenBracket)
            && matches!(tokens[10], Token::SymbolSeq(..))
            && matches!(tokens[11], Token::CloseBracket)));
    }

    #[test]
    fn lookahead_simple() {
        let regex = "^a(?=abc)$";

        let res = parse(regex);

        assert!(res.is_ok_and(|tokens| {
            tokens.len() == 2
                && matches!(tokens[0], Token::SymbolSeq(..))
                && matches!(&tokens[1], Token::LookaheadGroup(group) if group.len() == 1
					&& matches!(&group[0], Token::SymbolSeq(l) if l == "abc"))
        }));

        let regex = "^a(?=abc$)$";
        let res = parse(regex);

        assert!(res.is_ok_and(|tokens| tokens.len() == 2
            && matches!(tokens[0], Token::SymbolSeq(..))
            && matches!(tokens[1], Token::LookaheadGroup(..))));

        let l = parse(regex).unwrap();

        assert!(
            matches!(&l[1], Token::LookaheadGroup(group) if group.len() == 2 
					 && matches!(&group[0], Token::SymbolSeq(..))
					 && matches!(&group[1], Token::StringEnd))
        );
    }

    #[test]
    fn lookahead_invalid() {
        let regex = "^a(?=abc))$";
        assert!(parse(regex).is_err());

        let regex = "^a(?=abc)*abc$";
        assert!(parse(regex).is_err());
    }

    #[test]
    fn lookbehind_simple() {
        let regex = "^(?<=abc)abc$";

        let res = parse(regex);

        assert!(res.is_ok());

        let res = res.unwrap();

        assert_eq!(res.len(), 2);
        assert!(matches!(res[0], Token::LookbehindGroup(..)));
        assert!(matches!(res[1], Token::SymbolSeq(..)));
    }

    #[test]
    fn brackets_simplification() {
        let regex = "^a((ab*c))$";

        let res = parse(regex);

        assert!(res.is_ok());

        let tokens = res.unwrap();

        assert_eq!(tokens.len(), 6);
        assert!(matches!(tokens[0], Token::SymbolSeq(..)));
        assert!(matches!(tokens[1], Token::OpenBracket));
        assert!(matches!(tokens[2], Token::SymbolSeq(..)));
        assert!(matches!(tokens[3], Token::Unary(..)));
        assert!(matches!(tokens[4], Token::SymbolSeq(..)));
        assert!(matches!(tokens[5], Token::CloseBracket));
    }

    #[test]
    fn lookahead_hard() {
        let regex = "^a(?=((ab)*c|kd))abc$";

        assert!(parse(regex).is_ok_and(|tokens| tokens.len() == 3
            && matches!(tokens[0], Token::SymbolSeq(..))
            && matches!(tokens[1], Token::LookaheadGroup(..))
            && matches!(tokens[2], Token::SymbolSeq(..))));

        let regex = "^a(?=(abc|kd))abc$";

        assert!(parse(regex).is_ok_and(|tokens| tokens.len() == 3
            && matches!(tokens[0], Token::SymbolSeq(..))
            && matches!(tokens[1], Token::LookaheadGroup(..))
            && matches!(tokens[2], Token::SymbolSeq(..))));
    }
}
