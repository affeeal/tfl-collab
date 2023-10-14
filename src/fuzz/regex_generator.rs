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

#[derive(Debug)]
pub struct RegexGenerator {
    config: Config,
}

impl RegexGenerator {
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub fn generate(&self, rcount: i32) -> Vec<String> {
        let mut result = vec![];

        for _ in 0..rcount {
            let regex = self.generate_rec(
                self.config.max_letter_count,
                self.config.star_height,
                self.config.max_lookahead_count,
            );
            result.push(format!("^{}$", regex));
        }

        result
    }

    fn get_random_symbol(&self) -> String {
        let mut rng = rand::thread_rng();
        let r = rng.gen_range(0..self.config.alphabet_size);
        return ('a'..='z')
            .into_iter()
            .nth(r.try_into().unwrap())
            .unwrap()
            .to_string();
    }

    fn generate_rec(
        &self,
        letter_count: usize,
        star_height: usize,
        lookahead_count: usize,
    ) -> String {
        if letter_count == 0 {
            return "".to_string();
        }

        let mut rng = rand::thread_rng();

        let r = rng.gen_range(0..5);

        match r {
            // concat
            0 => {
                let lhs = self.generate_rec(letter_count / 2, star_height, lookahead_count / 2);

                let rhs = self.generate_rec(
                    letter_count - letter_count / 2,
                    star_height,
                    lookahead_count - lookahead_count / 2,
                );

                return format!("{}{}", lhs, rhs);
            }

            // or
            1 => {
                if letter_count < 2 {
                    return self.generate_rec(letter_count, star_height, lookahead_count);
                }

                let mut lhs = "".to_string();

                let mut rhs = "".to_string();

                while lhs.is_empty() || rhs.is_empty() || lhs.eq(&rhs) {
                    lhs = self.generate_rec(letter_count / 2, star_height, lookahead_count / 2);
                    rhs = self.generate_rec(
                        letter_count - letter_count / 2,
                        star_height,
                        lookahead_count - lookahead_count / 2,
                    );
                }

                return format!("({}|{})", lhs, rhs);
            }

            // star
            2 => {
                if star_height == 0 {
                    return self.generate_rec(letter_count, star_height, lookahead_count);
                }

                let regex = self.generate_rec(letter_count, star_height - 1, 0);
                if regex.len() >= 1 {
                    return format!("({})*", regex);
                } else {
                    return self.generate_rec(letter_count, star_height, lookahead_count);
                }
            }

            // lookahead
            3 => {
                if lookahead_count == 0 {
                    return self.generate_rec(letter_count, star_height, lookahead_count);
                }

                let regex = self.generate_lookahead(letter_count, self.config.star_height);

                return format!("(?={})", regex);
            }

            // symbol
            _ => {
                return format!(
                    "{}{}",
                    self.get_random_symbol(),
                    self.generate_rec(letter_count - 1, star_height, lookahead_count)
                );
            }
        }
    }

    // <lookahead> ::= <lookahead><binary><lookahead> | (<lookahead>) | <lookahead><unary> | <symbol> | ε

    fn generate_lookahead(&self, letter_count: usize, star_height: usize) -> String {
        if letter_count == 0 {
            return "".to_string();
        }

        let mut rng = rand::thread_rng();

        let r = rng.gen_range(0..5);

        match r {
            // concat
            0 => {
                let lhs = self.generate_lookahead(letter_count / 2, star_height);
                let rhs = self.generate_lookahead(letter_count - letter_count / 2, star_height);
                return format!("{}{}", lhs, rhs);
            }

            // or
            1 => {
                if letter_count < 2 {
                    return self.generate_lookahead(letter_count, star_height);
                }

                let mut lhs = "".to_string();
                let mut rhs = "".to_string();

                while lhs.is_empty() || rhs.is_empty() || lhs.eq(&rhs) {
                    lhs = self.generate_lookahead(letter_count / 2, star_height);
                    rhs = self.generate_lookahead(letter_count - letter_count / 2, star_height);
                }

                return format!("({}|{})", lhs, rhs);
            }

            // star
            2 => {
                if star_height == 0 {
                    return self.generate_lookahead(letter_count, star_height);
                }

                let r = self.generate_lookahead(letter_count, star_height - 1);

                if r.len() >= 1 {
                    return format!("({})*", r);
                } else {
                    return self.generate_lookahead(letter_count, star_height);
                }
            }

            // symbol
            _ => {
                return format!(
                    "{}{}",
                    self.get_random_symbol(),
                    self.generate_lookahead(letter_count - 1, star_height)
                );
            }
        }
    }
}
