pub mod calculator {
    pub type Value = u8;
    type Register = usize;
    type Registers = [Value; 4];

    #[derive(Debug)]
    enum Instruction {
        MOVI (Register, Value),
        ADD (Register, Register),
        SUB (Register, Register),
        MUL (Register, Register),
        DIV (Register, Register),
        IN (Register),
        OUT (Register),
    }

    fn get_one_reg(iter: &mut impl Iterator<Item = impl AsRef<str>>) -> Result<Register, String> {
        Ok(match iter.next() {
            Some(s) =>  {
                match s.as_ref() {
                    "A" => 0,
                    "B" => 1,
                    "C" => 2,
                    "D" => 3,
                    err_str => return Err(format!("Unknown reg: {}", err_str)),
                }
            },
            _ => return Err(format!("Cannot parse reg")),
        })
    }

    fn get_reg_with_comma(iter: &mut impl Iterator<Item = impl AsRef<str>>) -> Result<Register, String> {
        match iter.next() {
            Some(s) => {
                let mut split = s.as_ref().split(",");
                let r = get_one_reg(&mut split);
                if split.next().is_none() {
                    return Err(format!("Wrong delimiter"));
                }   
                r
            },
            _ => return Err(format!("Cannot parse 1reg")),
        }
    }

    fn get_two_regs(iter: &mut impl Iterator<Item = impl AsRef<str>>) -> Result<(Register, Register), String> 
    {
        let r1 = match get_reg_with_comma(iter) {
            Ok(r) => r,
            Err(e) => return Err(format!("Cannot parse 1reg. {}", e)),
        };
        let r2 = match get_one_reg(iter) {
            Ok(r) => r,
            Err(e) => return Err(format!("Cannot parse 2reg. {}", e)),
        };
        Ok((r1, r2))
    }

    fn get_reg_value(iter: &mut impl Iterator<Item = impl AsRef<str>>) -> Result<(Register, Value), String> {
        let r1 = match get_reg_with_comma(iter) {
            Ok(r) => r,
            Err(e) => return Err(format!("Cannot parse reg. {}", e)),
        };
        Ok(match iter.next() {
            Some(s) => {
                match s.as_ref().parse::<Value>() {
                    Err(_) => return Err(format!("Cannot parse u8 value: {}", s.as_ref())),
                    Ok(p) => {
                        if p > 127 {
                            return Err(format!("Cannot parse u8 value: {}", s.as_ref()));
                        }
                        (r1, p)
                    }
                }
            },
            _ => return Err(format!("Cannot parse u8 value")),
        })
    }

    fn parse_instruction(line: &str) -> Result<Instruction, String> {
        let mut parts = line.split_whitespace();
        let res = 
        match parts.next() {
            Some("MOVI") => { let (r1, v) = get_reg_value(&mut parts)?; Instruction::MOVI( r1, v ) },
            Some("ADD") => { let (r1, r2) = get_two_regs(&mut parts)?; Instruction::ADD( r1, r2 ) },
            Some("SUB") => { let (r1, r2) = get_two_regs(&mut parts)?; Instruction::SUB( r1, r2 ) },
            Some("MUL") => { let (r1, r2) = get_two_regs(&mut parts)?; Instruction::MUL( r1, r2 ) },
            Some("DIV") => { let (r1, r2) = get_two_regs(&mut parts)?; Instruction::DIV( r1, r2 ) },
            Some("IN") => Instruction::IN(get_one_reg(&mut parts)?),
            Some("OUT") => Instruction::OUT(get_one_reg(&mut parts)?),
            _ => return Err(String::from("Unknown op code")),
        };
        if !parts.next().is_none() {
            return Err(format!("Invalid number of args in op"));
        }
        return Ok(res);
    }

    type ProgFunReturn = Result<Vec<Value>, String>;
    type ProgFun = Box<dyn Fn(&Vec<Value>, bool) -> ProgFunReturn>;

    pub fn create_program(text: &str) -> Result<ProgFun, String> {
        let mut instructions: Vec<Instruction> = Vec::new();
        for line in text.lines() {
            let i = match parse_instruction(line) {
                Ok(i) => i,
                Err(e) => return Err(format!("Error in line: {}\n  {}", line, e)),
            };
            instructions.push(i); 
        }

        let res = Box::new(move |input: &Vec<Value>, on_detailed: bool| -> ProgFunReturn {
            let mut input_iter = input.iter();
            let mut state: Registers = [0; 4];
            let mut output: Vec<Value> = Vec::new();

            for i in &instructions {
                if on_detailed {print!{"{i:?}"};}
                match i {
                    Instruction::MOVI(r, v) => state[*r] = *v,
                    Instruction::ADD(r1, r2) =>  {
                        let t = state[*r1] as u16 + state[*r2] as u16;
                        if on_detailed && t > 255 {print!(" (overflow)");}
                        state[*r1] = t as u8
                    },
                    Instruction::SUB(r1, r2) => {
                        if state[*r1] < state[*r2] {
                            return Err(format!("Attempt to subtract with overflow"));
                        }
                        state[*r1] -= state[*r2]
                    },
                    Instruction::MUL(r1, r2) => state[*r1] *= state[*r2],
                    Instruction::DIV(r1, r2) => {
                        if state[*r2] == 0 {
                            return Err(format!("Div by zero"));
                        }
                        state[*r1] /= state[*r2]
                    },
                    Instruction::IN(r) => {
                        let t = input_iter.next();
                        state[*r] = if t != None {
                            *t.unwrap()
                        } else {
                            return Err(format!("Not enough input values"));
                        };
                    },
                    Instruction::OUT(r) => output.push(state[*r]),
                };
                if on_detailed {println!(" -> regs:{state:?}");}
            }
            return Ok(output); 
        });
        Ok(res)
    }
}

fn main() {
    let source_code = "\
    IN A
    MOVI D, 3
    DIV A, D
    MOVI D, 5
    ADD A, D
    IN B
    MUL A, B
    MOVI D, 2
    SUB A, D
    OUT A
    OUT D";
    //(x / 3 + 5) * y - 2
    let prog = match calculator::create_program(source_code) {
        Err(e) => { println!("{e}"); return;},
        Ok(p) => p
    };
    let values = vec![3, 2];
    println!("{:?}", prog(&values, true));
    println!("{:?}", prog(&values, false));
}


#[cfg(test)]
mod tests {
    mod run_prog {
        use crate::calculator;
        #[test]
        fn not_enought_input() {
            let source_code = "\
            IN A
            MOVI D, 3
            ADD A, D
            IN B
            MUL A, B";
            //(x /3 + 5) * y -2
            let prog = match calculator::create_program(source_code) {
                Err(e) => { println!("{e}"); assert!(false); return; },
                Ok(p) => p
            };
            let values = vec!();
            let _ = match prog(&values, false) {
                Err(e) => { println!("{e}"); assert!(true);},
                Ok(_) => { assert!(false);},
            };
        }

        #[test]
        fn div_by_zero() {
            let source_code = "\
            IN A
            MOVI D, 3
            DIV D, A
            OUT D";
            // 3 / x
            let prog = match calculator::create_program(source_code) {
                Err(e) => { println!("{e}"); assert!(false); return; },
                Ok(p) => p
            };
            let values = vec![0, 0, 0];
            let _ = match prog(&values, false) {
                Err(e) => { println!("{e}"); assert!(true);},
                Ok(_) => { assert!(false);},
            };
        }

        #[test]
        fn neg_ans() {
            let source_code = "\
            IN A
            MOVI D, 3
            SUB D, A
            OUT D";
            // 3 - x
            let prog = match calculator::create_program(source_code) {
                Err(e) => { println!("{e}"); assert!(false); return; },
                Ok(p) => p
            };
            let values = vec![5, 0, 0];
            let _ = match prog(&values, false) {
                Err(e) => { println!("{e}"); assert!(true);},
                Ok(_) => { assert!(false);},
            };
        }

        #[test]
        fn add_oveflow() {
            let source_code = "\
            IN A
            MOVI D, 120
            ADD D, A
            OUT D";
            // 120 + x
            let prog = match calculator::create_program(source_code) {
                Err(e) => { println!("{e}"); assert!(false); return; },
                Ok(p) => p
            };
            let values = vec![240];
            match prog(&values, true) {
                Err(e) => { println!("{e}"); assert!(false);},
                Ok(r) => { assert!(r.len() == 1); assert!(r[0] == ((120 + 240) as u8))},
            };
        }
    }
    //minus res

    mod create_prog {
        use crate::calculator;
        #[test]
        fn wrong_value() {
            let _ = match calculator::create_program("MOVI A, 130\nADD A, B\nMUL D, C") {
                Err(e) => { println!("{e}"); assert!(true);},
                Ok(_) => { assert!(false);},
            };
            let _ = match calculator::create_program("MOVI A, 300\nADD A, B\nMUL D, C") {
                Err(e) => { println!("{e}"); assert!(true);},
                Ok(_) => { assert!(false);},
            };
            let _ = match calculator::create_program("MOVI Y, 100\nADD A, B\nMUL D, C") {
                Err(e) => { println!("{e}"); assert!(true);},
                Ok(_) => { assert!(false);},
            };
            let _ = match calculator::create_program("ADD 1, B\nMUL D, C") {
                Err(e) => { println!("{e}"); assert!(true);},
                Ok(_) => { assert!(false);},
            };
            let _ = match calculator::create_program("ADD A, Y\nMUL D, C") {
                Err(e) => { println!("{e}"); assert!(true);},
                Ok(_) => { assert!(false);},
            };
        }

        #[test]
        fn wrong_inst() {
            let _ = match calculator::create_program("MUL D, C\na 256") {
                Err(e) => { println!("{e}"); assert!(true);},
                Ok(_) => { assert!(false);},
            };
            let _ = match calculator::create_program("MUL D, C olo\nADD A, B") {
                Err(e) => { println!("{e}"); assert!(true);},
                Ok(_) => { assert!(false);},
            };
            let _ = match calculator::create_program("MOVI 256\nADD A, B\nMUL D, C") {
                Err(e) => { println!("{e}"); assert!(true);},
                Ok(_) => { assert!(false);},
            };
        }
        #[test]
        fn forgot_comma() {
            let _ = match calculator::create_program("ADD A B") {
                Err(e) => { println!("{e}");  assert!(true);},
                Ok(_) => { assert!(false);},
            };
        }
        #[test]
        fn empty_prog() {
            let _ = match calculator::create_program("") {
                Err(e) => { println!("{e}"); assert!(false);},
                Ok(_) => { assert!(true);},
            };
        }
    }
}