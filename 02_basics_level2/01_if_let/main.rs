#[derive(Debug)]
enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

// Expression as a tree
#[derive(Debug)]
enum Expression {
    Op {
        op: Operation,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Value(i64),
}

// recursive eval
fn eval(e: Expression) -> Result<i64, String> {
    Ok( match e { // <- OK wrapper for all returns
        Expression::Op { op, left, right } => {
            // with help of if let
            let left_res = eval(*left);
            let left_val = if let Ok(r) = left_res {
                r
            } else {
                return left_res;
            };
            let right_res = eval(*right);
            let right_val = if let Ok(r) = right_res {
                r
            } else {
                return right_res;
            };
            // with help of match
            // let left_val = match eval(*left) {
            //     Ok(r) => r,
            //     Err(e) => return Err(e),
            // };
            // let right_val = match eval(*right) {
            //     Ok(r) => r,
            //     Err(e) => return Err(e),
            // };

            // shortest variant
            // let left_val = eval(*left)?;
            // let right_val = eval(*right)?;

            match op {
                Operation::Add => left_val + right_val,
                Operation::Sub => left_val - right_val,
                Operation::Mul => left_val * right_val,
                Operation::Div => {
                    if right_val == 0 {
                        return Err(String::from("Div zero"));
                    } else {
                        left_val / right_val
                    }
                },
            }
        },
        Expression::Value(val) => val,
    })
}

fn main() {
    let expr = Expression::Op {
        op: Operation::Sub,
        left: Box::new(Expression::Value(20)),
        right: Box::new(Expression::Value(10)),
    };
    println!("Expression: {:?}", expr);
    println!("Result: {:?}", eval(expr));
}

// Test module. Runs with cmd `cargo test`
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value() {
        assert_eq!(eval(Expression::Value(19)), Ok(19));
    }

    #[test]
    fn test_sum() {
        assert_eq!(
            eval(Expression::Op {
                op: Operation::Add,
                left: Box::new(Expression::Value(10)),
                right: Box::new(Expression::Value(20)),
            }),
            Ok(30)
        );
    }

    #[test]
    fn test_recursion() {
        let term1 = Expression::Op {
            op: Operation::Mul,
            left: Box::new(Expression::Value(10)),
            right: Box::new(Expression::Value(9)),
        };
        let term2 = Expression::Op {
            op: Operation::Mul,
            left: Box::new(Expression::Op {
                op: Operation::Sub,
                left: Box::new(Expression::Value(3)),
                right: Box::new(Expression::Value(4)),
            }),
            right: Box::new(Expression::Value(5)),
        };
        assert_eq!(
            eval(Expression::Op {
                op: Operation::Add,
                left: Box::new(term1),
                right: Box::new(term2),
            }),
            Ok(85)
        );
    }

    #[test]
    fn test_error() {
        assert_eq!(
            eval(Expression::Op {
                op: Operation::Div,
                left: Box::new(Expression::Value(99)),
                right: Box::new(Expression::Value(0)),
            }),
            Err(String::from("Div zero"))
        );
    }
}