use crate::{
    ndfa::{self, Automata},
    parser::{parse, Token},
};

pub fn gen_rec(r: &str) -> Result<Automata, String> {
    if r.eq("^$") {
        return Ok(Automata::from_regex(&"".to_string()));
    }

    let tokens = parse(r)?;
    let mut s = "".to_string();
    let mut brackets_counter = 0;
    let mut i = 0;
    let mut first_bracket_idx = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::OpenBracket => {
                brackets_counter += 1;
                if brackets_counter == 1 {
                    first_bracket_idx = i;
                }
            }
            Token::CloseBracket => brackets_counter -= 1,
            Token::LookaheadGroup(group) => {
                if brackets_counter == 0 {
                    let mut tmp: String = group
                        .iter()
                        .fold("".to_string(), |acc, t| acc + &t.to_string());

                    if !matches!(group.last(), Some(Token::StringEnd)) {
                        tmp += ".*";
                    }

                    let a1 = Automata::from_regex(&s);

                    let a2 = Automata::from_regex(&tmp);

                    let r3 = tokens[(i + 1)..]
                        .iter()
                        .fold("^".to_string(), |acc, t| acc + &t.to_string())
                        + "$";

                    return Ok(ndfa::concatenation(
                        &a1,
                        &ndfa::intersection(&a2, &gen_rec(&r3)?),
                    ));
                } else {
                    // abc*bc(ab(?=ab$)aa)
                    // abc*bc((ab)*ab(?=ab)(aab|abc))abc
                    // abc*bc(ab(?=ab$)aa|abc)
                    let l = first_bracket_idx;
                    let mut r = l + 1;
                    brackets_counter = 1;
                    let mut alternative_idx = 0;
                    while brackets_counter != 0 {
                        match tokens[r] {
                            Token::OpenBracket => brackets_counter += 1,
                            Token::CloseBracket => brackets_counter -= 1,
                            Token::Binary(_) => {
                                if alternative_idx == 0 && brackets_counter == 1 {
                                    alternative_idx = r;
                                }
                            }
                            _ => {}
                        }
                        r += 1;
                    }

                    r -= 1;

                    let r1 = tokens[0..l]
                        .iter()
                        .fold("".to_string(), |acc, t| acc + &t.to_string());

                    let r2 = tokens[(l + 1)..r]
                        .iter()
                        .fold("^".to_string(), |acc, t| acc + &t.to_string())
                        + "$";

                    let r3 = tokens[(r + 1)..]
                        .iter()
                        .fold("^".to_string(), |acc, t| acc + &t.to_string())
                        + "$";

                    if alternative_idx == 0 {
                        return Ok(ndfa::concatenation(
                            &Automata::from_regex(&r1),
                            &ndfa::concatenation(&gen_rec(&r2)?, &gen_rec(&r3)?),
                        ));
                    } else {
                        let r2 = tokens[(l + 1)..alternative_idx]
                            .iter()
                            .fold("^".to_string(), |acc, t| acc + &t.to_string())
                            + "$";

                        let r3 = tokens[(alternative_idx + 1)..r]
                            .iter()
                            .fold("^".to_string(), |acc, t| acc + &t.to_string())
                            + "$";

                        return Ok(ndfa::concatenation(
                            &Automata::from_regex(&r1),
                            &ndfa::union(&gen_rec(&r2)?, &gen_rec(&r3)?),
                        ));
                    }
                }
            }
            _ => {}
        }

        s += &tokens[i].to_string();
        i += 1;
    }

    Ok(Automata::from_regex(&s))
}
