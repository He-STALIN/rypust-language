#![allow(dead_code, unused_variables)]
// use crate::lexer::token::Token;

use crate::lexer::token::Token;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),
    Str(String),
    Float(f64),
    Ident(String),
    Bool(bool),
    Call { 
        callee: Box<Expr>, 
        args: Vec<Expr> 
    },
    Binary {
        left: Box<Expr>,
        op: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Box<Expr>,
        value: Box<Expr>,
    },
    Function {
        name: String,
        args: Vec<Expr>,
        body: Vec<Stmt>
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Block(Vec<Stmt>),
    If {
        condition: Expr,       // Условие (например, x > 2)
        then_branch: Vec<Stmt>,// Что делать, если true (блок)
        else_branch: Option<Vec<Stmt>>, // Что делать, если false (может быть, а может и нет)
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>> // условие. если цикл закончился
    },
}