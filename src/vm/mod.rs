use std::collections::HashMap;

use rypust::{RUNTIME_ERR, SYNTAX_ERR, raiseErr};
use crate::parser::ast::{Expr, Stmt};
use crate::lexer::token::Token;

#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Null
}

#[derive(Debug, Clone)]
pub struct RypustFunction {
    pub args: Vec<Expr>,
    pub body: Vec<Stmt>,
}

pub struct VM {
    variables: HashMap<String, Value>, // память переменных
    functions: HashMap<String, RypustFunction>
}

impl VM {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new()
        }
    }

    pub fn exec_block(&mut self, block: &[Stmt]) {
        for stmt in block {
            if let Stmt::Expr(expr) = stmt {
                // println!("{:?}", expr);
                self.eval(&expr);
            } else if let Stmt::Block(inner) = stmt {
                self.exec_block(inner);
            } else if let Stmt::If { condition, then_branch, else_branch } = stmt {
                let cond_val = self.eval(condition);
                let is_true = match cond_val {
                    Value::Bool(b) => b,
                    Value::Number(n) => n != 0,
                    Value::Null => false,
                    _ => true,
                };
                if is_true {
                    self.exec_block(then_branch);
                } else if let Some(else_block) = else_branch {
                    self.exec_block(else_block);
                }
            } else if let Stmt::While { condition, body, else_branch } = stmt {
                loop {
                    // calculate on each iteration
                    let cond_val = self.eval(condition);
                    
                    // check to true
                    let is_true = match cond_val {
                        Value::Bool(b) => b,
                        Value::Number(n) => n != 0,
                        Value::Null => false,
                        _ => true,
                    };

                    // if is false - exit from cycle
                    if !is_true {
                        if let Some(else_block) = else_branch {
                            self.exec_block(else_block);
                        }
                        break;
                    }

                    // exec body of cycle
                    self.exec_block(body); 
                }
            }
        }
    }

    pub fn eval(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Str(s) => Value::Str(s.clone()),
            Expr::Number(n) => Value::Number(*n),
            Expr::Float(f) => Value::Float(*f),
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
                                Value::Float(f) => print!("{}", f),
                                Value::Str(s) => print!("{}", s),
                                Value::Bool(b) => print!("{}", b),
                                Value::Null => print!("Null"),
                            }
                        }
                        println!();
                        return Value::Null;
                    }

                    // parse recieved arguments to function
                    if let Some(function) = self.functions.get(name).cloned() {
                        let mut local_vars = HashMap::new();

                        for (i, arg) in function.args.iter().enumerate() {
                            if let Expr::Ident(kwarg) = arg {
                                let var = self.eval(&args[i]);
                                local_vars.insert(kwarg.clone(), var);
                            }
                        }

                        let old_vars = std::mem::replace(&mut self.variables, local_vars);
                        let mut result = Value::Null; // result of the execution function

                        for stmt in function.body {
                            if let Stmt::Expr(expr) = stmt {
                                result = self.eval(&expr); // while return the latest result
                            }
                        }

                        self.variables = old_vars;

                        return result;
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
                        Token::Eq => Value::Bool(l == r),
                        Token::NotEq => Value::Bool(l != r),
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
            Expr::Function { name, args, body } => {
                self.functions.insert(
                    name.clone(),
                    RypustFunction {
                        args: args.clone(),
                        body: body.clone()
                    }
                );
                Value::Null
            }
        }
    }
}