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
            self.advance(); // съедаем 'if'
            
            // 1. Парсим условие
            let condition = self.parse_expr(0);
            
            // 2. Ждем двоеточие ':'
            if self.current != Token::Colon {
                raiseErr!(SYNTAX_ERR, "Expected ':' after if condition");
            }
            self.advance(); // съедаем ':'
            
            // 3. Пропускаем перенос строки перед блоком
            while self.current == Token::NewLine {
                self.advance();
            }
            
            // 4. Ждем Indent (начало блока)
            if self.current != Token::INDENT {
                raiseErr!(INDENT_ERR, "Expected indent after ':'");
            }
            self.advance(); // съедаем Indent
            
            // 5. Парсим тело блока (parse_block вернет Vec<Stmt>)
            let then_branch = self.parse_block();
            
            // 6. Ищем else (если он есть)
            let mut else_branch = None;
            
            // Пропускаем пустые строки перед else
            while self.current == Token::NewLine {
                self.advance();
            }
            
            if self.current == Token::Else {
                self.advance(); // съедаем 'else'
                
                // Ждем двоеточие после else
                if self.current == Token::Colon {
                    self.advance(); // съедаем ':'
                } else {
                    raiseErr!(INDENT_ERR, "Expected ':' after else");
                }
                
                // Пропускаем перенос строки
                while self.current == Token::NewLine {
                    self.advance();
                }
                
                // Ждем Indent
                if self.current == Token::INDENT {
                    self.advance(); // съедаем Indent
                    else_branch = Some(self.parse_block()); // Сохраняем как Some
                } else {
                    raiseErr!(INDENT_ERR, "Expected indent after else:");
                }
            }
            
            return Some(Stmt::If {
                condition,
                then_branch,
                else_branch, // Option<Vec<Stmt>>
            });
        }

        let expr = self.parse_expr(0);

        if self.current == Token::Assign {
            self.advance(); // съедаем '='
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
            
            // Парсим правую часть с приоритетом ВЫШЕ текущего (чтобы 2+2*2 работало правильно)
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
}