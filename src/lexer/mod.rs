pub mod token;
pub mod span;
use span::Span;
use rypust::raiseErr;
use token::{Token, SpannedToken};
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    indent_stack: Vec<usize>, // stack of current indents
    at_line_start: bool,      // We at the start of line?
    pending_dedents: usize,   // queue of tokens DEDENT
    line: usize,
    column: usize
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            indent_stack: vec![0], // Базовый уровень - 0
            at_line_start: true,
            pending_dedents: 0,
            line: 1,
            column: 1
        }
    }

    fn advance_char(&mut self) {
        if self.pos < self.chars.len() {
            if self.chars[self.pos] == '\n' {
                self.line += 1;
                self.column = 1; // Сброс на новую строку
            } else {
                self.column += 1;
            }
            self.pos += 1;
        }
    }

    fn make_span(&self, token: Token, line: usize, column: usize) -> SpannedToken {
        return SpannedToken {
            token,
            span: Span { line, column },
        };
    }

    pub fn next_token(&mut self) -> SpannedToken {
        let line = self.line;
        let column = self.column;
        // Если у нас в очереди есть Dedent (после уменьшения отступа), отдаем их по одному
        if self.pending_dedents > 0 {
            self.pending_dedents -= 1;
            return self.make_span(Token::DEDENT, line, column);
        }

        // Если мы в начале строки, считаем отступ
        if self.at_line_start {
            self.at_line_start = false;
            let mut spaces = 0;
            
            // counting spaces (1 \t = 4 spaces)
            while self.pos < self.chars.len() {
                match self.chars[self.pos] {
                    ' ' => { spaces += 1; self.advance_char(); }
                    '\t' => { spaces += 4; self.advance_char(); }
                    _ => break,
                }
            }

            let current_indent = *self.indent_stack.last().unwrap();

            if spaces > current_indent {
                self.indent_stack.push(spaces);
                return self.make_span(Token::INDENT, line, column);
            } else if spaces < current_indent {
                // Убираем уровни из стека, пока не найдем совпадение
                while *self.indent_stack.last().unwrap() > spaces {
                    self.indent_stack.pop();
                    self.pending_dedents += 1;
                }
                
                // Проверка на кривой отступ (например, 2 пробела вместо 4 или 0)
                if *self.indent_stack.last().unwrap() != spaces {
                    raiseErr!("IndentationError", "Unexpected indentation level");
                }

                self.pending_dedents -= 1;
                return self.make_span(Token::DEDENT, line, column);
            }
        }

        // Пропускаем пробелы внутри строки (не в начале)
        while self.pos < self.chars.len() && (self.chars[self.pos] == ' ' || self.chars[self.pos] == '\t') {
            self.advance_char();
        }

        if self.pos >= self.chars.len() {
            return self.make_span(Token::Eof, line, column);
        }

        let c = self.chars[self.pos];

        self.skip_comments();
        // Обработка переноса строки
        if c == '\n' {
            self.advance_char();
            self.at_line_start = true;
            return self.make_span(Token::NewLine, line, column); // Возвращаем токен новой строки
        }

        if c.is_ascii_digit() { return self.read_number(); }
        if c == '"' { return self.read_string(); }
        if c.is_alphabetic() || c == '_' { return self.read_ident(); }
        
        // operators calculating
        match c {
            '+' => {
                self.advance_char();
                return self.make_span(Token::Plus, line, column);
            },
            '#' => {
                while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
                    self.advance_char();
                }
                self.next_token()
            }
            '<' => {
                self.advance_char();
                return self.make_span(Token::Lessthan, line, column);
            },
            '>' => {
                self.advance_char();
                return self.make_span(Token::Morethan, line, column);
            }
            '-' => {
                self.advance_char();
                return self.make_span(Token::Minus, line, column);
            },
            '*' => {
                self.advance_char();
                return self.make_span(Token::Star, line, column);
            },
            '/' => {
                self.advance_char();
                return self.make_span(Token::Slash, line, column);
            },
            '=' => {
                if self.chars[self.pos + 1] == '=' {
                    self.advance_char();
                    self.advance_char();
                    return self.make_span(Token::Eq, line, column);
                }
                else {
                    self.advance_char();
                    return self.make_span(Token::Assign, line, column);
                }
            },
            '!' => {
                if self.chars[self.pos + 1] == '=' {
                    self.advance_char();
                    self.advance_char();
                    return self.make_span(Token::NotEq, line, column);
                } else {
                    self.advance_char();
                    self.next_token()
                }
            }
            '(' => {
                self.advance_char();
                return self.make_span(Token::LParen, line, column);
            },
            ')' => {
                self.advance_char();
                return self.make_span(Token::RParen, line, column);
            },
            ':' => {
                self.advance_char();
                return self.make_span(Token::Colon, line, column);
            },
            ',' => {
                self.advance_char();
                return self.make_span(Token::Comma, line, column);
            },
            _ => {
                // skip unknown symbol
                self.advance_char();
                self.next_token()
            }
        }
    }

    fn read_number(&mut self) -> SpannedToken {
        let start = self.pos;
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            self.advance_char();
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        return self.make_span(Token::Number(s.parse().unwrap()), self.line, self.column);
    }

    fn read_string(&mut self) -> SpannedToken {
        self.advance_char(); // пропускаем открывающую "
        let start = self.pos;
        while self.pos < self.chars.len() && self.chars[self.pos] != '"' {
            self.advance_char();
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        self.advance_char(); // пропускаем закрывающую "
        return self.make_span(Token::Str(s), self.line, self.column);
    }

    fn read_ident(&mut self) -> SpannedToken {
        let start = self.pos;
        while self.pos < self.chars.len()
            && (self.chars[self.pos].is_alphanumeric() || self.chars[self.pos] == '_')
        {
            self.advance_char();
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        match s.as_str() {
            "def" => self.make_span(Token::Def, self.line, self.column),
            "if" => self.make_span(Token::If, self.line, self.column),
            "elif" => self.make_span(Token::Elif, self.line, self.column),
            "else" => self.make_span(Token::Else, self.line, self.column),
            "for" => self.make_span(Token::For, self.line, self.column),
            "while" => self.make_span(Token::While, self.line, self.column),
            "raise" => self.make_span(Token::Raise, self.line, self.column),
            "try" => self.make_span(Token::Try, self.line, self.column),
            "except" => self.make_span(Token::Except, self.line, self.column),
            "async" => self.make_span(Token::Async, self.line, self.column),
            "await" => self.make_span(Token::Await, self.line, self.column),
            _ => self.make_span(Token::Ident(s), self.line, self.column), // если не ключевое — возвращаем как имя
        }
    }

    fn skip_comments(&mut self) {
        if self.chars[self.pos] == '#' {
            while self.chars[self.pos] != '\n' {
                self.advance_char();
            }
        }
    }
}