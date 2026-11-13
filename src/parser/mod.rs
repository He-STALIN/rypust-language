#![allow(unused)]
pub mod ast;

use crate::lexer::Lexer;
use crate::lexer::token::Token;
use ast::{Expr, Stmt};
use rypust::{INDENT_ERR, SYNTAX_ERR, VALUE_ERR, raiseErr};

pub struct Parser {
    lexer: Lexer,
    current: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current = lexer.next_token();
        Self { lexer, current }
    }

    // Сдвигает лексер на следующий токен
    fn advance(&mut self) {
        self.current = self.lexer.next_token();
    }

    fn skip_newlines(&mut self) {
        while self.current == Token::NewLine {
            self.advance();
        }
    }

    // Проверяет, что текущий токен — нужный, иначе паникует
    fn expect(&mut self, expected: Token, error_msg: &str) {
        if self.current != expected {
            raiseErr!(SYNTAX_ERR, "{}", error_msg);
        }
        self.advance();
    }

    // Пропускает NewLine, проверяет Indent, съедает его и парсит блок
    fn parse_indented_block(&mut self) -> Vec<Stmt> {
        self.skip_newlines();
        if self.current != Token::INDENT {
            raiseErr!(INDENT_ERR, "Expected indent after ':'");
        }
        self.advance(); // съедаем Indent
        self.parse_block()
    }

    pub fn parse_block(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        
        // Пропускаем NewLine перед началом блока (если есть)
        while self.current == Token::NewLine {
            self.advance();
        }

        // Пока не дошли до Dedent или конца файла
        while self.current != Token::DEDENT && self.current != Token::Eof {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            }
            
            // Пропускаем все NewLine между операторами (пустые строки)
            while self.current == Token::NewLine {
                self.advance();
            }
        }
        
        // Съедаем Dedent (закрываем блок)
        if self.current == Token::DEDENT {
            self.advance();
        }
        
        stmts
    }

    // Главная функция: разбирает один оператор (строку)
    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        // Пропускаем пустые строки в начале
        while self.current == Token::NewLine {
            self.advance();
        }

        if self.current == Token::Eof {
            return None;
        }

        // Если видим Indent, значит начался блок кода
        if self.current == Token::INDENT {
            self.advance(); // съедаем Indent
            let block = self.parse_block();
            return Some(Stmt::Block(block));
        }

        if let Token::If = self.current.clone() {
            return self.parse_if_condition();
        }

        if let Token::Def = self.current.clone() {
            self.advance(); // съедаем 'def'
            
            // 1. Читаем имя функции (должен быть Ident)
            let name = match self.current.clone() {
                Token::Ident(n) => {
                    self.advance();
                    n
                },
                _ => { raiseErr!(SYNTAX_ERR, "Expected function name after 'def'"); },
            };
            
            // 2. Читаем аргументы в скобках
            if self.current != Token::LParen {
                raiseErr!(SYNTAX_ERR, "Expected '(' after function name");
            }
            self.advance(); // съедаем '('
            
            let mut args = Vec::new();
            while self.current != Token::RParen && self.current != Token::Eof {
                if let Token::Ident(arg_name) = self.current.clone() {
                    args.push(Expr::Ident(arg_name)); // Пока просто имена аргументов
                    self.advance();
                } else {
                    raiseErr!(SYNTAX_ERR, "Expected argument name");
                }
                
                // Если есть запятая, пропускаем её и читаем следующий аргумент
                if self.current == Token::Comma {
                    self.advance();
                }
            }
            
            if self.current != Token::RParen {
                raiseErr!(SYNTAX_ERR, "Expected ')' after arguments");
            }
            self.advance(); // съедаем ')'
            
            // 3. Ждем двоеточие ':'
            if self.current != Token::Colon {
                raiseErr!(SYNTAX_ERR, "Expected ':' after function signature");
            }
            self.advance(); // съедаем ':'
            
            // 4. Пропускаем перенос строки и ждем Indent
            while self.current == Token::NewLine {
                self.advance();
            }
            if self.current != Token::INDENT {
                panic!("SyntaxError: Expected indent after ':'");
            }
            self.advance(); // съедаем Indent
            
            // 5. Парсим тело функции
            let body = self.parse_block();
            
            // Возвращаем узел функции (если он у тебя в Stmt или Expr — подставь нужное)
            return Some(Stmt::Expr(
                Expr::Function {
                    name: name,
                    args, // Vec<Expr>
                    body,
                }
            ));
        }

        if let Token::While = self.current.clone() {
            self.advance(); // съедаем 'while'
            
            // 1. parse condition
            let condition = self.parse_expr(0);
            
            // 2. wait ':'
            self.expect(Token::Colon, "Expected ':' after while condition");
            
            // 3. Parsing cycle body
            let body = self.parse_indented_block();

            let mut else_branch = None;

            if let Token::Else = self.current.clone() {
                else_branch = self.parse_elif_else_chain();
            }
            
            return Some(Stmt::While {
                condition,
                body,
                else_branch
            });
        }

        let expr = self.parse_expr(0);

        if self.current == Token::Assign {
            self.advance(); //? eat '='
            let value = self.parse_expr(0);
            
            // Если слева было имя (Expr::Ident), делаем Variable
            if let Expr::Ident(name) = expr {
                if self.current == Token::NewLine {
                    self.advance();
                }
                return Some(Stmt::Expr(Expr::Variable {
                    name: Box::new(Expr::Ident(name)),
                    value: Box::new(value),
                }));
            } else {
                raiseErr!(VALUE_ERR, "Left side of assignment must be a variable");
            }
        }
        
        // После выражения ОБЯЗАТЕЛЬНО должен идти NewLine или Eof
        if self.current == Token::NewLine {
            self.advance();
        }
        
        Some(Stmt::Expr(expr))
    }

    pub fn parse_expr(&mut self, min_prec: i32) -> Expr {
        let mut left = self.parse_primary();

        // Пока текущий токен — оператор, и его приоритет >= минимальному
        while self.current.precedence() >= min_prec && self.current.precedence() > 0 {
            let op = self.current.clone();
            let prec = op.precedence();
            self.advance(); // съедаем оператор
            
            // Парсим правую часть с приоритетом ВЫШЕ текущего
            let right = self.parse_expr(prec + 1);

            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        left
    }

    fn parse_primary(&mut self) -> Expr {
        match self.current.clone() {
            Token::Number(n) => {
                self.advance();
                Expr::Number(n)
            }
            Token::Str(s) => {
                self.advance();
                Expr::Str(s)
            }
            Token::Ident(name) => {
                self.advance();
                
                // Если после имени идет '(', это вызов функции
                if self.current == Token::LParen {
                    self.advance(); // съедаем '('
                    let mut args = Vec::new();
                    while self.current != Token::RParen && self.current != Token::Eof {
                        args.push(self.parse_expr(0)); // парсим аргументы
                        if self.current == Token::Comma {
                            self.advance();
                        }
                    }
                    self.advance(); // съедаем ')'
                    return Expr::Call { callee: Box::new(Expr::Ident(name)), args };
                }
                
                // Если это просто переменная
                Expr::Ident(name)
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr(0); // парсим то, что внутри скобок
                self.advance(); // съедаем ')'
                expr
            }
            _ => { raiseErr!(SYNTAX_ERR, "Unexpected token in expression: {:?}", self.current); }
        }
    }

    fn parse_if_condition(&mut self) -> Option<Stmt> {
        self.advance(); // съедаем 'if'
        
        // 1. Парсим условие
        let condition = self.parse_expr(0);
        
        // 2. Ждем двоеточие
        println!("Current token: {:?}", self.current);
        self.expect(Token::Colon, "Expected ':' after if condition");
        
        // 3. Парсим тело блока
        let then_branch = self.parse_indented_block();
        
        // 4. Ищем elif или else (передаем управление второй функции)
        let else_branch = self.parse_elif_else_chain();
        
        return Some(Stmt::If {
            condition,
            then_branch,
            else_branch,
        });
    }

    /**
     * Recoursive search of else or elif branches
     */
    fn parse_elif_else_chain(&mut self) -> Option<Vec<Stmt>> {
        // skip empty lines
        self.skip_newlines();
        
        if self.current == Token::Elif {
            self.advance(); // eat 'elif'
            
            // parse condition
            let condition = self.parse_expr(0);
            
            // wait colon
            self.expect(Token::Colon, "Expected ':' after elif condition");
            
            // parse body
            let then_branch = self.parse_indented_block();
            
            // RECURSION: Search elif or else into the else
            let nested_else = self.parse_elif_else_chain();
            
            // Packing to Stmt::If knot
            let if_stmt = Stmt::If {
                condition,
                then_branch,
                else_branch: nested_else,
            };
            
            return Some(vec![if_stmt]);
        } else if self.current == Token::Else {
            self.advance(); // съедаем 'else'
            
            // Ждем двоеточие
            self.expect(Token::Colon, "Expected ':' after else");
            
            // Парсим тело
            let then_branch = self.parse_indented_block();
            
            // Else doen't have nested condition, so we return branch
            return Some(then_branch);
        }
        
        // if condition else/if is empty return None
        None
    }
}