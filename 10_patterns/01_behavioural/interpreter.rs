use std::collections::VecDeque;

#[derive(PartialEq)]
enum TokenType {
    OP,
    NUM,
    OPEN,
    CLOSE,
    NONE,
}

#[derive(PartialEq)]
enum State {
    NotCalculatedPolska,
    NotCalculatedResult,
    ParseErr,
}

pub struct Interpreter<'a> {
    original: &'a str,
    polska: Option<VecDeque<String>>,
    result: Option<i64>,
    state: State,
}

impl<'a> Interpreter<'a> {
    pub fn new(infix: &'a str) -> Self {
        Self {
            original: infix,
            polska: None,
            result: None,
            state: State::NotCalculatedPolska,
        }
    }

    fn calc_polska(&mut self) {
        let mut prev = TokenType::NONE;
        let mut temp = VecDeque::new();
        let mut res = VecDeque::new();
        let buf = self.original;
        let mut i = 0;
        let mut set_err_state = || self.state = State::ParseErr;

        while i < buf.len() {
            let c = buf.chars().nth(i).unwrap();
            i += 1;
            match c {
                ' ' => continue,
                '+' | '-' => {
                    match prev {
                        TokenType::NONE | TokenType::OPEN => {
                            res.push_back("0".to_string());
                        }
                        TokenType::OP => {
                            return set_err_state();
                        }
                        _ => {}
                    }
                    while matches!(temp.back(), Some(s) if s == "+" || s == "-" || s == "*") {
                        res.push_back(temp.pop_back().unwrap());
                    }
                    temp.push_back(c.to_string());
                    prev = TokenType::OP;
                }
                '*' => {
                    if !(prev == TokenType::NUM || prev == TokenType::CLOSE) {
                        return set_err_state();
                    }
                    while matches!(temp.back(), Some(s) if s == "*") {
                        res.push_back(temp.pop_back().unwrap());
                    }
                    temp.push_back(c.to_string());
                    prev = TokenType::OP;
                }
                '(' => {
                    if prev == TokenType::NUM || prev == TokenType::CLOSE {
                        return set_err_state();
                    }
                    temp.push_back("(".to_string());
                    prev = TokenType::OPEN;
                }
                ')' => {
                    if !(prev == TokenType::NUM || prev == TokenType::CLOSE) {
                        return set_err_state();
                    }
                    let is_open = loop {
                        if let Some(s) = temp.pop_back() {
                            if s == "(" {
                                break true;
                            }
                            res.push_back(s);
                        }
                        break false;
                    };
                    if !is_open {
                        return set_err_state();
                    }
                    prev = TokenType::CLOSE;
                }
                '0'..='9' => {
                    let number = buf
                        .chars()
                        .skip(i - 1)
                        .take_while(|c| c.is_digit(10))
                        .collect::<String>();
                    i = i - 1 + number.len();
                    if matches!(prev, TokenType::NONE | TokenType::OP | TokenType::OPEN) {
                        res.push_back(number);
                    } else {
                        return set_err_state();
                    }
                    prev = TokenType::NUM;
                }
                _ => {
                    return set_err_state();
                }
            };
        }
        while let Some(s) = temp.pop_back() {
            if !(s == "+" || s == "-" || s == "*") {
                return set_err_state();
            }
            res.push_back(s);
        }

        self.state = State::NotCalculatedResult;
        self.polska = Some(res)
    }

    pub fn get_polska(&mut self) -> Option<String> {
        if self.state == State::NotCalculatedPolska {
            self.calc_polska();
        }
        match self.state {
            State::ParseErr => None,
            _ => Some(self.polska.as_ref().unwrap().iter().cloned().collect()),
        }
    }

    pub fn get_result(&mut self) -> Option<i64> {
        if self.state == State::NotCalculatedPolska {
            self.calc_polska();
        }
        match self.state {
            State::NotCalculatedResult => {}
            _ => {
                return self.result;
            }
        };

        let mut temp = VecDeque::new();
        let mut din = self.polska.as_mut().unwrap().clone();
        let mut set_err_state = || {
            self.state = State::ParseErr;
            None
        };
        while let Some(s) = din.pop_front() {
            if let Ok(n) = s.parse::<i64>() {
                temp.push_back(n);
                continue;
            }
            match (temp.pop_back(), temp.pop_back()) {
                (Some(b), Some(a)) => {
                    let res = match s.as_str() {
                        "+" => a + b,
                        "-" => a - b,
                        "*" => a * b,
                        _ => {
                            return set_err_state();
                        }
                    };
                    temp.push_back(res);
                }
                _ => return set_err_state(),
            }
        }
        self.result = temp.pop_back();
        self.result
    }
}

fn main() {
    let s = "1-2+3-4";
    let mut intr = Interpreter::new(s);
    let p = intr.get_polska();
    let r = intr.get_result();
    println!("{s} {p:?} {r:?}");
}
