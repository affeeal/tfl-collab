use rand::Rng;

#[derive(Debug, Clone)]
pub struct Config {
    pub max_lookahead_count: usize,
    pub star_height: usize,
    pub alphabet_size: i32,
    pub max_letter_count: usize,
}

/*
<init> ::= ∧<regex>$
<regex> ::= <regex><binary><regex> | (<regex>) | <regex><unary> | <symbol> | (?=<lookahead>$?) | ε
<lookahead> ::= <lookahead><binary><lookahead> | (<lookahead>) | <lookahead><unary> | <symbol> | ε
<binary> ::= '|' | ε
<unary> ::= *
*/

pub fn generate(c: &Config, rcount: i32) -> Vec<String> {
    let mut result = vec![];

    for _ in 0..rcount {
        let regex = generate_rec(
            c.max_letter_count,
            c.star_height,
            c.max_lookahead_count,
            0,
            &mut c.clone(),
        );
        result.push(format!("^{}$", regex));
    }

    result
}

fn get_random_symbol(size: usize) -> String {
    let mut rng = rand::thread_rng();
    let r = rng.gen_range(0..size);
    return ('a'..='z')
        .into_iter()
        .nth(r.try_into().unwrap())
        .unwrap()
        .to_string();
}

fn generate_rec(
    letter_count: usize,
    star_height: usize,
    lookahead_count: usize,
    call_number: usize,
    conf: &mut Config,
) -> String {
    if letter_count == 0 {
        return "".to_string();
    }

    let mut rng = rand::thread_rng();

    let r = rng.gen_range(0..5);

    match r {
        // concat
        0 => {
            let mut tmp = conf.clone();
            let mut lhs = generate_rec(
                letter_count / 2,
                star_height,
                lookahead_count / 2,
                call_number + 1,
                &mut tmp,
            );

            if call_number == 0 && lhs.starts_with("(?=") {
                lhs = generate_rec(
                    letter_count / 2,
                    star_height,
                    lookahead_count - lookahead_count / 2,
                    call_number + 1,
                    &mut tmp,
                );
            }

            let rhs = generate_rec(
                letter_count - letter_count / 2,
                lookahead_count - lookahead_count / 2,
                star_height,
                call_number + 1,
                &mut tmp,
            );
            return format!("{}{}", lhs, rhs);
        }

        // or
        1 => {
            if letter_count < 2 {
                return generate_rec(
                    letter_count,
                    star_height,
                    lookahead_count,
                    call_number,
                    conf,
                );
            }

            let mut lhs = generate_rec(
                letter_count / 2,
                star_height,
                lookahead_count / 2,
                call_number + 1,
                conf,
            );

            if call_number == 0 && lhs.starts_with("(?=") {
                lhs = generate_rec(
                    letter_count / 2,
                    star_height,
                    lookahead_count - lookahead_count / 2,
                    call_number + 1,
                    conf,
                );
            }

            let rhs = generate_rec(
                letter_count - letter_count / 2,
                star_height,
                lookahead_count - lookahead_count / 2,
                call_number + 1,
                conf,
            );

            if lhs.is_empty() || rhs.is_empty() {
                return generate_rec(
                    letter_count,
                    star_height,
                    lookahead_count,
                    call_number + 1,
                    conf,
                );
            }
            return format!("({}|{})", lhs, rhs);
        }

        // star
        2 => {
            if star_height == 0 || call_number == 0 {
                return generate_rec(
                    letter_count,
                    star_height,
                    lookahead_count,
                    call_number,
                    conf,
                );
            }

            let regex = generate_rec(
                letter_count,
                star_height - 1,
                lookahead_count,
                call_number + 1,
                conf,
            );

            if regex.len() > 1 {
                return format!("({})*", regex);
            } else if regex.len() == 1 {
                return format!("{}*", regex);
            } else {
                return generate_rec(
                    letter_count,
                    star_height,
                    lookahead_count,
                    call_number + 1,
                    conf,
                );
            }
        }

        // lookahead
        3 => {
            if lookahead_count == 0 || call_number == 0 {
                return generate_rec(
                    letter_count,
                    star_height,
                    lookahead_count,
                    call_number,
                    conf,
                );
            }

            let regex = generate_lookahed(letter_count, conf.star_height, conf);

            return format!("(?={})", regex);
        }

        // symbol
        _ => {
            return format!(
                "{}{}",
                get_random_symbol(conf.alphabet_size.try_into().unwrap()),
                generate_rec(
                    letter_count - 1,
                    star_height,
                    lookahead_count,
                    call_number + 1,
                    conf
                )
            );
        }
    }
}

// <lookahead> ::= <lookahead><binary><lookahead> | (<lookahead>) | <lookahead><unary> | <symbol> | ε

fn generate_lookahed(letter_count: usize, star_height: usize, conf: &mut Config) -> String {
    if letter_count == 0 {
        return "".to_string();
    }

    let mut rng = rand::thread_rng();

    let r = rng.gen_range(0..5);

    match r {
        // concat
        0 => {
            let lhs = generate_lookahed(letter_count / 2, star_height, conf);
            let rhs = generate_lookahed(letter_count - letter_count / 2, star_height, conf);
            return format!("{}{}", lhs, rhs);
        }

        // or
        1 => {
            if conf.max_letter_count < 2 {
                return generate_lookahed(letter_count, star_height, conf);
            }

            let lhs = generate_lookahed(letter_count / 2, star_height, conf);
            let rhs = generate_lookahed(letter_count - letter_count / 2, star_height, conf);

            if lhs.is_empty() || rhs.is_empty() {
                return generate_lookahed(letter_count, star_height, conf);
            }
            return format!("({}|{})", lhs, rhs);
        }

        // star
        2 => {
            if star_height == 0 {
                return generate_lookahed(letter_count, star_height, conf);
            }

            let mut r = generate_lookahed(letter_count, star_height - 1, conf);

            if r.contains("(?=") {
                r = generate_lookahed(letter_count, star_height - 1, conf);
            }
            if r.len() > 1 {
                return format!("({})*", r);
            } else if r.len() == 1 {
                return format!("{}*", r);
            } else {
                return generate_lookahed(letter_count, star_height, conf);
            }
        }

        _ => {
            return format!(
                "{}{}",
                get_random_symbol(conf.alphabet_size.try_into().unwrap()),
                generate_lookahed(letter_count - 1, star_height, conf)
            );
        }
    }
}
