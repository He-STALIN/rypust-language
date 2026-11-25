use std::env;
use std::fs;
use std::io::Write;
use std::io::stdout;

mod parser;
mod lexer;
mod vm;
use lexer::Lexer;
use rypust::{raiseErr, parse_args};
use rypust::error::RUNTIME_ERR;
use crate::lexer::token::Token;
use crate::parser::Parser;
use crate::vm::{Value, VM};

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = parse_args(&args);
    if config.is_verbose >= 1 {
        println!("\x1b[32m:: Running interpreteter\x1b[0m");
    }

    let path = &args[1];
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            raiseErr!("FileOpenError", "Cannot read '{}' - {}", path, e);
        }
    };

    let mode = detect_mode(&source);
    if mode == "Unknown" {
        raiseErr!("DirectiveError", "File must have directive '$prog <soft|low>'");
    }
    if config.is_verbose >= 1 { println!("\x1b[33m[mode] {}\x1b[0m", mode); }

    let to_compile = &source
        .replace("$prog soft", "")
        .replace("$prog low", "");

    if config.is_verbose >= 1 {
        match mode.as_str() {
            "soft" => println!("\x1b[33m[soft] in dev mode\x1b[0m"),
            "low" => println!("\x1b[33m[low] not implemented yet\x1b[0m"),
            _ => { raiseErr!(RUNTIME_ERR, "Unknown mode '{}'", mode); },
        }
    }

    if config.is_verbose >= 1 { println!("\x1b[32m:: Parsing file\x1b[0m"); }

    let lex = Lexer::new(&to_compile);
    if config.is_verbose >= 2 {
        println!("-==[ TOKENS ]==-");
        let mut temp_lexer = Lexer::new(&to_compile); // copying for don't break
        loop {
            let tok = temp_lexer.next_token();
            print!("{:?}, ", tok);
            stdout().flush().unwrap();
            if tok.token == Token::Eof { break; }
        }
        println!("EOF");
    }
    let mut parser = Parser::new(lex);
    let mut vm = VM::new();

    if mode == "soft" {
        if config.is_verbose >= 1 { println!("\x1b[32m:: Running execution VM\x1b[0m"); }
        while let Some(stmt) = parser.parse_stmt() {
            match stmt {
                parser::ast::Stmt::Expr(expr) => { vm.eval(&expr); },
                parser::ast::Stmt::Block(block) => { vm.exec_block(&block); }
                parser::ast::Stmt::If{condition, then_branch, else_branch} => {
                    let cond_val = vm.eval(&condition); //? get condition result
                    
                    let is_true = match cond_val {
                        Value::boolObject(b) => b,           // if bool just take his value
                        Value::floatObject(f) => f != 0.0,    
                        Value::intObject(n) => n != 0,        // zero is False
                        Value::NoneObject => false,                // None is False
                        _ => true,                                 // Строки и прочее - true
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
                            Value::boolObject(b) => b,
                            Value::intObject(n) => n != 0,
                            Value::NoneObject => false,
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
}

fn detect_mode(source: &str) -> String {
    for line in source.lines() {
        if let Some(rest) = line.strip_prefix("$prog ") {
            return rest.split_whitespace().next().unwrap_or("Unknown").to_string();
        }
    }
    "Unknown".to_string()
}