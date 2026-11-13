use std::env;
use std::fs;

mod lexer;
use lexer::Lexer;
use rypust::{raiseErr, RUNTIME_ERR};
mod parser;
use crate::lexer::token::Token;
use crate::parser::Parser;
use crate::vm::Value;
mod vm;
use vm::VM;
use rypust::parse_args;

fn main() {
    println!("\x1b[32m:: Running interpreteter\x1b[0m");
    let args: Vec<String> = env::args().collect();

    let config = parse_args(&args);

    let path = &args[1];
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            raiseErr!("FileOpenError", "Cannot read '{}' - {}", path, e);
        }
    };

    let mode = detect_mode(&source);
    if mode == "Unknown" {
        eprintln!("\x1b[31m\x1b[1mDirectiveError\x1b[0m: File must have directive '$prog <soft|low>'");
        std::process::exit(1); //? exit with general error
    }
    println!("\x1b[33m[mode] {}\x1b[0m", mode);

    let to_compile = &source
        .replace("$prog soft", "")
        .replace("$prog low", "");

    match mode.as_str() {
        "soft" => println!("\x1b[33m[soft] in dev mode\x1b[0m"),
        "low" => println!("\x1b[33m[low] not implemented yet\x1b[0m"),
        _ => { raiseErr!(RUNTIME_ERR, "Unknown mode '{}'", mode); },
    }

    println!("\x1b[32m:: Parsing file\x1b[0m");

    let lex = Lexer::new(&to_compile);
    if config.is_verbose {
        println!("-==[ TOKENS ]==-");
        let mut temp_lexer = Lexer::new(&to_compile); // copying for don't break
        loop {
            let tok = temp_lexer.next_token();
            println!("{:?}", tok);
            if tok == Token::Eof { break; }
        }
    }
    let mut parser = Parser::new(lex);
    let mut vm = VM::new();

    println!("\x1b[32m:: Running execution VM\x1b[0m");
    while let Some(stmt) = parser.parse_stmt() {
        match stmt {
            parser::ast::Stmt::Expr(expr) => { vm.eval(&expr); },
            parser::ast::Stmt::Block(block) => { vm.exec_block(&block); }
            parser::ast::Stmt::If{condition, then_branch, else_branch} => {
                let cond_val = vm.eval(&condition); //? get condition result
                
                let is_true = match cond_val {
                    Value::Bool(b) => b,           // Если это Bool, берем его значение
                    Value::Float(f) => f != 0.0,
                    Value::Number(n) => n != 0,    // Если число, то 0 - это false
                    Value::Null => false,          // Null - это false
                    _ => true,                     // Строки и прочее - true
                };

                if is_true {
                    vm.exec_block(&then_branch);
                } else if let Some(else_block) = else_branch {
                    vm.exec_block(&else_block);
                }
            },
            parser::ast::Stmt::While { condition, body, else_branch } => {
                loop {
                    // calculate on each iteration
                    let cond_val = vm.eval(&condition);
                    
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
                            vm.exec_block(&else_block);
                        }
                        break;
                    }

                    // exec body of cycle
                    vm.exec_block(&body); 
                }
            }
        }
    }
}

fn detect_mode(source: &str) -> String {
    for line in source.lines() {
        if let Some(rest) = line.strip_prefix("$prog ") {
            return rest.split_whitespace().next().unwrap_or("Unknown").to_string();
        }
    }
    "Unknown".to_string()
}