pub mod token;
use rypust::raiseErr;
use token::Token;
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    indent_stack: Vec<usize>, // stack of current indents
    at_line_start: bool,      // We at the start of line?
    pending_dedents: usize,   // queue of tokens DEDENT
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            indent_stack: vec![0], // Базовый уровень - 0
            at_line_start: true,
            pending_dedents: 0,
        }
    }

    pub fn next_token(&mut self) -> Token {
        // Если у нас в очереди есть Dedent (после уменьшения отступа), отдаем их по одному
        if self.pending_dedents > 0 {
            self.pending_dedents -= 1;
            return Token::DEDENT;
        }

        // Если мы в начале строки, считаем отступ
        if self.at_line_start {
            self.at_line_start = false;
            let mut spaces = 0;
            
            // counting spaces (1 \t = 4 spaces)
            while self.pos < self.chars.len() {
                match self.chars[self.pos] {
                    ' ' => { spaces += 1; self.pos += 1; }
                    '\t' => { spaces += 4; self.pos += 1; }
                    _ => break,
                }
            }

            let current_indent = *self.indent_stack.last().unwrap();

            if spaces > current_indent {
                self.indent_stack.push(spaces);
                return Token::INDENT;
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
                return Token::DEDENT;
            }
        }

        // Пропускаем пробелы внутри строки (не в начале)
        while self.pos < self.chars.len() && (self.chars[self.pos] == ' ' || self.chars[self.pos] == '\t') {
            self.pos += 1;
        }

        if self.pos >= self.chars.len() {
            return Token::Eof;
        }

        let c = self.chars[self.pos];

        // Обработка переноса строки
        if c == '\n' {
            self.pos += 1;
            self.at_line_start = true;
            return Token::NewLine; // Возвращаем токен новой строки
        }

        if c.is_ascii_digit() { return self.read_number(); }
        if c == '"' { return self.read_string(); }
        if c.is_alphabetic() || c == '_' { return self.read_ident(); }
        
        // Обработка операторов (как у тебя было)
        match c {
            '+' => {
                self.pos += 1;
                return Token::Plus;
            },
            '<' => {
                self.pos += 1;
                return Token::Lessthan;
            },
            '>' => {
                self.pos += 1;
                return Token::Morethan;
            }
            '-' => {
                self.pos += 1;
                return Token::Minus;
            },
            '*' => {
                self.pos += 1;
                return Token::Star;
            },
            '/' => {
                self.pos += 1;
                return Token::Slash;
            },
            '=' => {
                if self.chars[self.pos + 1] == '=' {
                    self.pos += 2; // now and next symbols
                    return Token::Eq;
                }
                else {
                    self.pos += 1;
                    return Token::Assign;
                }
            },
            '(' => {
                self.pos += 1;
                return Token::LParen;
            },
            ')' => {
                self.pos += 1;
                return Token::RParen;
            },
            ':' => {
                self.pos += 1;
                return Token::Colon;
            },
            ',' => {
                self.pos += 1;
                Token::Comma
            },
            _ => {
                // skip unknown symbol
                self.pos += 1;
                self.next_token()
            }
        }
    }

    // fn skip_whitespace(&mut self) {
    //     while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
    //         self.pos += 1;
    //     }
    // }

    fn read_number(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        Token::Number(s.parse().unwrap())
    }

    fn read_string(&mut self) -> Token {
        self.pos += 1; // пропускаем открывающую "
        let start = self.pos;
        while self.pos < self.chars.len() && self.chars[self.pos] != '"' {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        self.pos += 1; // пропускаем закрывающую "
        Token::Str(s)
    }

    fn read_ident(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.chars.len()
            && (self.chars[self.pos].is_alphanumeric() || self.chars[self.pos] == '_')
        {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        match s.as_str() {
            "def" => Token::Def,
            "if" => Token::If,
            "else" => Token::Else,
            "for" => Token::For,
            "while" => Token::While,
            "raise" => Token::Raise,
            "try" => Token::Try,
            "except" => Token::Except,
            "async" => Token::Async,
            "await" => Token::Await,
            _ => Token::Ident(s), // если не ключевое — возвращаем как имя
        }
    }
}