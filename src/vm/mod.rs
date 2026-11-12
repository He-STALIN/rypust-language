use std::collections::HashMap;

use rypust::{RUNTIME_ERR, SYNTAX_ERR, raiseErr};

use crate::parser::ast::{Expr, Stmt};

use crate::lexer::token::Token;

#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    Str(String),
    Bool(bool),
    Null
}


pub struct VM {
    variables: HashMap<String, Value>, // память переменных
}

impl VM {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new()
        }
    }

    pub fn exec_block(&mut self, block: &[Stmt]) {
        // println!("--- ENTER BLOCK ---");
        for stmt in block {
            if let Stmt::Expr(expr) = stmt {
                // println!("{:?}", expr);
                self.eval(&expr);
            } else if let Stmt::Block(inner) = stmt {
                self.exec_block(inner);
            }
        }
        // println!("--- EXIT BLOCK ---");
    }

    pub fn eval(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Str(s) => Value::Str(s.clone()),
            Expr::Number(n) => Value::Number(*n),
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Call { callee, args } => {
                // Проверяем, что вызывают именно функцию "print"
                if let Expr::Ident(name) = &**callee {
                    if name == "print" {
                        for arg in args {
                            // Рекурсивно вычисляем каждый аргумент
                            let val = self.eval(arg);
                            match val {
                                Value::Number(n) => print!("{}", n),
                                Value::Str(s) => print!("{}", s),
                                Value::Bool(b) => print!("{}", b),
                                Value::Null => print!("Null"),
                            }
                        }
                        println!();
                        return Value::Null;
                    }
                }
                raiseErr!(RUNTIME_ERR, "Unknown function");
            },
            Expr::Binary { left, op, right} => {
                let left_var = self.eval(left);
                let right_var = self.eval(right);

                if let (Value::Number(l), Value::Number(r)) = (&left_var, &right_var) {
                    match op {
                        Token::Plus => Value::Number(l + r),
                        Token::Minus => Value::Number(l - r),
                        Token::Star => Value::Number(l * r),
                        Token::Slash => Value::Number(l / r),
                        Token::Lessthan => Value::Bool(l < r),
                        Token::Morethan => Value::Bool(l > r),
                        _ => { raiseErr!(SYNTAX_ERR, "Unknown binary operator: {:?}", op); },
                    }
                } else {
                    raiseErr!(RUNTIME_ERR, "Math operations only support numbers");
                }
            },
            Expr::Variable { name, value } => {
                let val = self.eval(value);

                if let Expr::Ident(var_name) = &**name {
                    self.variables.insert(var_name.clone(), val.clone());
                }
                val
            },
            Expr::Ident(name) => {
                // Пока просто выводим имя, если это не вызов
                if self.variables.contains_key(name) {
                    return self.variables.get(name).cloned().unwrap_or(Value::Null);
                } else {
                    raiseErr!(RUNTIME_ERR, "Reference to an unregistered variable");
                }
            },
        }
    }
}