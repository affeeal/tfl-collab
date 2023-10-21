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

    pub fn generate(&self, rcount: usize) -> Vec<String> {
        let mut result = vec![];

        let mut rng = rand::thread_rng();
        for _ in 0..rcount {
            let mut regex;
            let b = rng.gen_bool(0.65);

            if b || self.config.max_lookahead_count < 2 || self.config.max_letter_count < 6 {
                regex = self.generate_rec(
                    self.config.max_letter_count,
                    self.config.star_height,
                    self.config.max_lookahead_count,
                );
                loop {
                    let c = self.get_letters_count(&regex);
                    let k = self.get_lookahead_count(&regex);

                    if c == self.config.max_letter_count {
                        break;
                    }

                    regex = format!(
                        "{}{}",
                        regex,
                        self.generate_rec(
                            self.config.max_letter_count - c,
                            self.config.star_height,
                            self.config.max_lookahead_count - k,
                        )
                    );
                }
            } else {
                let first = self.generate_rec(
                    self.config.max_letter_count / 2,
                    self.config.star_height,
                    self.config.max_lookahead_count / 2,
                );

                // rhs = alternative with lookahead
                let mut second = "".to_string();
                let mut third = "".to_string();
                while !second.contains("(?=") && !third.contains("(?=") {
                    second = self.generate_rec(
                        self.config.max_letter_count / 4,
                        self.config.star_height,
                        self.config.max_lookahead_count / 2 - self.config.max_lookahead_count / 4,
                    );

                    third = self.generate_rec(
                        self.config.max_letter_count / 4,
                        self.config.star_height,
                        self.config.max_lookahead_count / 4,
                    );
                }

                regex = format!("{}({}|{})", first, second, third);
                println!("------------------");
                println!("{}", regex);
                println!("------------------");
            }

            result.push(format!("^{}$", regex));
        }

        result
    }

    fn get_random_symbol(&self) -> String {
        let mut rng = rand::thread_rng();
        let r = rng.gen_range(0..self.config.alphabet_size);
        ('a'..='z')
            .into_iter()
            .nth(r.try_into().unwrap())
            .unwrap()
            .to_string()
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
                    lhs = self.generate_rec(letter_count / 2, star_height, 0);
                    rhs = self.generate_rec(letter_count - letter_count / 2, star_height, 0);
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

            // symbol
            3 => self.get_random_symbol(),

            // lookahead
            _ => {
                if lookahead_count == 0 {
                    return self.generate_rec(letter_count, star_height, lookahead_count);
                }

                let regex = self.generate_lookahead(letter_count);

                return format!("(?={})", regex);
            }
        }
    }

    // <lookahead> ::= <lookahead><binary><lookahead> | (<lookahead>) | <lookahead><unary> | <symbol> | ε

    fn generate_lookahead(&self, letter_count: usize) -> String {
        let mut r = "".to_string();

        loop {
            let k = self.get_letters_count(&r);

            if k == letter_count {
                return r;
            }

            r = format!(
                "{}{}",
                r,
                self.generate_lookahead_rec(letter_count - k, self.config.star_height)
            );
        }
    }

    fn generate_lookahead_rec(&self, letter_count: usize, star_height: usize) -> String {
        if letter_count == 0 {
            return "".to_string();
        }

        let mut rng = rand::thread_rng();

        let r = rng.gen_range(0..4);

        match r {
            // concat
            0 => {
                let lhs = self.generate_lookahead_rec(letter_count / 2, star_height);
                let rhs = self.generate_lookahead_rec(letter_count - letter_count / 2, star_height);
                return format!("{}{}", lhs, rhs);
            }

            // or
            1 => {
                if letter_count < 2 {
                    return self.generate_lookahead_rec(letter_count, star_height);
                }

                let mut lhs = "".to_string();
                let mut rhs = "".to_string();

                while lhs.is_empty() || rhs.is_empty() || lhs.eq(&rhs) {
                    lhs = self.generate_lookahead_rec(letter_count / 2, star_height);
                    rhs = self.generate_lookahead_rec(letter_count - letter_count / 2, star_height);
                }

                return format!("({}|{})", lhs, rhs);
            }

            // star
            2 => {
                if star_height == 0 {
                    return self.generate_lookahead_rec(letter_count, star_height);
                }

                let r = self.generate_lookahead_rec(letter_count, star_height - 1);

                if r.len() >= 1 {
                    return format!("({})*", r);
                } else {
                    return self.generate_lookahead_rec(letter_count, star_height);
                }
            }

            // symbol
            _ => self.get_random_symbol(),
        }
    }

    fn get_letters_count(&self, r: &str) -> usize {
        r.chars()
            .fold(0, |acc, c| if c.is_alphabetic() { acc + 1 } else { acc })
    }

    fn get_lookahead_count(&self, r: &str) -> usize {
        r.matches("(?=").count()
    }
}
