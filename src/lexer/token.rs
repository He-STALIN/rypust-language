#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(i64),
    Str(String),
    Ident(String),
    INDENT,    // space opn block
    DEDENT,    // close block
    NewLine,   // new line tag
    Eof,       // end of file
    Plus,      // +
    Minus,     // -
    Assign,    // =
    Eq,        // ==
    Star,      // *
    Slash,     // /
    Morethan,  // >
    Lessthan,  // <
    LParen,    // (
    RParen,    // )
    Colon,     // :
    Comma,     // ,
    Def,       // Define method
    If,        // If operation
    Else,      // else operation
    For,       // cycle for
    While,     // cycle while
    Raise,     // raise exceptions
    Async,     // async <any>
    Await,     // await async code
    Try,       // try/except struct
    Except,    // except the errors
}

impl Token {
    pub fn precedence(&self) -> i32 {
        match self {
            Token::Lessthan | Token::Morethan => return 10,
            Token::Plus | Token::Minus => return 20,
            Token::Star | Token::Slash => return 40,
            _ => -1,
        }
    }
}