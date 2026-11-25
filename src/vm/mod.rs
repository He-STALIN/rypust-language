#![allow(non_camel_case_types, unused)]
use std::collections::HashMap;
use std::io::{self, Write};

pub mod types;
use types::intType;
use rypust::raiseErr;
use rypust::error::{RUNTIME_ERR, SYNTAX_ERR, TYPE_ERR, VALUE_ERR};
use crate::parser::ast::{Expr, Stmt};
use crate::lexer::token::Token;

#[derive(Debug, Clone)]
pub enum Value {
    intObject(i64),
    floatObject(f64),
    strObject(String),
    boolObject(bool),
    NoneObject
}

#[derive(Debug, Clone)]
pub struct RypustFunc {
    pub args: Vec<Expr>,
    pub body: Vec<Stmt>,
}

pub struct VM {
    variables: HashMap<String, Value>,          //? variables memory
    functions: HashMap<String, RypustFunc>      //? functions memory
}

impl VM {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new()
        }
    }

    /// Execution block of code into Stmt block
    pub fn exec_block(&mut self, block: &[Stmt]) {
        for stmt in block {
            match stmt {
                Stmt::Expr(expr) => {
                    self.eval(expr);
                },
                Stmt::Block(inner) => {
                    self.exec_block(inner);
                },
                Stmt::If { condition, then_branch, else_branch } => {
                    let cond_val = self.eval(condition);
                    let is_true = match cond_val {
                        Value::boolObject(b) => b,
                        Value::intObject(n) => n != 0,
                        Value::NoneObject => false,
                        _ => true
                    };
                    if is_true {
                        self.exec_block(then_branch);
                    } else if let Some(else_block) = else_branch {
                        self.exec_block(else_block);
                    }
                },
                Stmt::While { condition, body, else_branch } => {
                    loop {
                        //? calculate on each iteration
                        let cond_val = self.eval(condition);
                        
                        //? check to true
                        let is_true = match cond_val {
                            Value::boolObject(b) => b,
                            Value::intObject(n) => n != 0,
                            Value::NoneObject => false,
                            _ => true,
                        };

                        //? if is false - exit from cycle
                        if !is_true {
                            if let Some(else_block) = else_branch {
                                self.exec_block(else_block);
                            }
                            break;
                        }

                        //? exec body of cycle
                        self.exec_block(body); 
                    }
                },
                _ => {
                    raiseErr!(RUNTIME_ERR, "Unknown Stmt format: {:?}", stmt);
                }
            }
        }
    }

    /// evaluate the code block
    pub fn eval(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Str(s) => Value::strObject(s.clone()),
            Expr::Number(n) => Value::intObject(*n),
            Expr::Float(f) => Value::floatObject(*f),
            Expr::Bool(b) => Value::boolObject(*b),
            Expr::Call { callee, args } => {
                if let Expr::Ident(name) = &**callee {
                    match name.as_str() { //? parse function name
                        "print" => {
                            for arg in args {
                                // Рекурсивно вычисляем каждый аргумент
                                let val = self.eval(arg);
                                match val {
                                    Value::intObject(n) => print!("{}", n),
                                    Value::floatObject(f) => print!("{:.1}", f),
                                    Value::strObject(s) => print!("{}", s),
                                    Value::boolObject(b) => print!("{}", b),
                                    Value::NoneObject => print!("Null"),
                                }
                            }
                            println!();
                            return Value::NoneObject;
                        },
                        "input" => {
                            if args.len() > 1 {
                                raiseErr!(TYPE_ERR, "Function `input` expected at most 1 argument, but recieved {}", args.len());
                            }
                            if args.len() == 1 {
                                let prompt = self.eval(&args[0]);
                                match prompt {
                                    Value::NoneObject => print!(""),
                                    Value::boolObject(b) => print!("{}", b),
                                    Value::intObject(n) => print!("{}", n),
                                    Value::floatObject(f) => print!("{:.1}", f),
                                    Value::strObject(s) => print!("{}", s),
                                }
                                io::stdout().flush().unwrap();
                            }
                            let mut input = String::new();
                            let _ = io::stdin().read_line(&mut input);
                            return Value::strObject(input.trim().to_string());
                        },
                        "type" => {
                            let obj = self.eval(&args[0]);
                            match obj {
                                Value::boolObject(_) => return Value::strObject("<class 'bool'>".to_string()),
                                Value::floatObject(_) => return Value::strObject("<class 'float'>".to_string()),
                                Value::intObject(_) => return Value::strObject("<class 'int'>".to_string()),
                                Value::strObject(_) => return Value::strObject("<class 'str'>".to_string()),
                                Value::NoneObject => return Value::strObject("<class 'NoneType'>".to_string()),
                                _ => {raiseErr!(TYPE_ERR, "Unknown object type");}
                            }
                        },
                        "int" => {
                            let obj = self.eval(&args[0]);
                            match obj { //? matching with types
                                Value::intObject(n) => return Value::intObject(n),
                                Value::floatObject(f) => return Value::intObject(f as i64),
                                Value::boolObject(b) => return Value::intObject(b as i64),
                                Value::NoneObject => { raiseErr!(VALUE_ERR, "Cannot convert type `None` to `int`"); },
                                Value::strObject(s) => {
                                    match s.parse::<i64>() {
                                        Ok(num) => return Value::intObject(num),
                                        Err(_) => { raiseErr!(VALUE_ERR, "Cannot convert to 'int': object have unsupported symbols");}
                                    }
                                }
                            }
                        },
                        "str" => {
                            let obj = self.eval(&args[0]);
                            match obj {
                                Value::intObject(n) => return Value::strObject(n.to_string()),
                                Value::boolObject(b) => return Value::strObject(b.to_string()),
                                Value::floatObject(f) => return Value::strObject(f.to_string()),
                                Value::strObject(s) => return Value::strObject(s), //? re-pack value
                                Value::NoneObject => return Value::strObject("None".to_string()),
                            }
                        },
                        "float" => {
                            let obj = self.eval(&args[0]);
                            match obj {
                                Value::floatObject(f) => return Value::floatObject(f),
                                Value::intObject(n) => return Value::floatObject(n as f64),
                                Value::boolObject(b) => return Value::floatObject((b as i64) as f64), //? first bool to int, after to float
                                Value::NoneObject => {raiseErr!(VALUE_ERR, "Cannot convert `None` to `float`"); },
                                Value::strObject(s) => {
                                    match s.parse::<f64>() {
                                        Ok(f) => return Value::floatObject(f),
                                        Err(_) => {raiseErr!(VALUE_ERR, "Cannot convert to 'float': object have unsupported symbols");}
                                    }
                                },
                            }
                        }
                        _ => {
                            //? Call custom function from user if this not a system function
                            if let Some(function) = self.functions.get(name).cloned() {
                                let mut local_vars = HashMap::new();

                                for (i, arg) in function.args.iter().enumerate() {
                                    if let Expr::Ident(kwarg) = arg {
                                        let var = self.eval(&args[i]);
                                        local_vars.insert(kwarg.clone(), var);
                                    }
                                }

                                let old_vars = std::mem::replace(&mut self.variables, local_vars);
                                let mut result = Value::NoneObject; //? result of the execution function
                                for stmt in function.body {
                                    if let Stmt::Expr(expr) = stmt {
                                        result = self.eval(&expr); //? while return the latest result
                                    }
                                }

                                self.variables = old_vars;
                                return result;
                            }
                        }
                    }
                }
                raiseErr!(RUNTIME_ERR, "Unknown function");
            },
            Expr::Binary { left, op, right} => {
                let left_var = self.eval(left);
                let right_var = self.eval(right);

                if let (Value::intObject(l), Value::intObject(r)) = (&left_var, &right_var) {
                    match op {
                        Token::Plus => Value::intObject(l + r),
                        Token::Minus => Value::intObject(l - r),
                        Token::Star => Value::intObject(l * r),
                        Token::Slash => Value::intObject(l / r),
                        Token::Lessthan => Value::boolObject(l < r),
                        Token::Morethan => Value::boolObject(l > r),
                        Token::Eq => Value::boolObject(l == r),
                        Token::NotEq => Value::boolObject(l != r),
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
                match &**name {
                    "int" => return Value::strObject("<class 'int'>".to_string()),
                    "str" => return Value::strObject("<class 'str'>".to_string()),
                    "float" => return Value::strObject("<class 'float'>".to_string()),
                    "bool" => return Value::strObject("<class 'bool'>".to_string()),
                    "None" => return Value::strObject("<class 'NoneType'>".to_string()),
                    "list" => return Value::strObject("<class 'list'>".to_string()),
                    "dict" => return Value::strObject("<class 'dict'>".to_string()),
                    _ => {
                        if self.variables.contains_key(name) {
                            return self.variables.get(name).cloned().unwrap_or(Value::NoneObject);
                        } else {
                            raiseErr!(RUNTIME_ERR, "Reference to an unregistered variable");
                        }
                    }
                }
            },
            Expr::Function { name, args, body } => {
                self.functions.insert(
                    name.clone(),
                    RypustFunc {
                        args: args.clone(),
                        body: body.clone()
                    }
                );
                Value::NoneObject
            }
        }
    }
}