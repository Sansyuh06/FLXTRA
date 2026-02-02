//! JavaScript lexer

use std::iter::Peekable;
use std::str::Chars;

/// Token type
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    Identifier(String),
    
    // Operators
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    Power,          // **
    
    PlusPlus,       // ++
    MinusMinus,     // --
    
    Eq,             // =
    EqEq,           // ==
    EqEqEq,         // ===
    NotEq,          // !=
    NotEqEq,        // !==
    Lt,             // <
    LtEq,           // <=
    Gt,             // >
    GtEq,           // >=
    
    And,            // &&
    Or,             // ||
    Not,            // !
    BitAnd,         // &
    BitOr,          // |
    BitXor,         // ^
    BitNot,         // ~
    Shl,            // <<
    Shr,            // >>
    UShr,           // >>>
    
    Question,       // ?
    Colon,          // :
    NullCoalesce,   // ??
    OptionalChain,  // ?.
    
    PlusEq,         // +=
    MinusEq,        // -=
    StarEq,         // *=
    SlashEq,        // /=
    
    // Delimiters
    LParen,         // (
    RParen,         // )
    LBrace,         // {
    RBrace,         // }
    LBracket,       // [
    RBracket,       // ]
    Comma,          // ,
    Semicolon,      // ;
    Dot,            // .
    Spread,         // ...
    Arrow,          // =>
    
    // Keywords
    Var,
    Let,
    Const,
    Function,
    Return,
    If,
    Else,
    While,
    For,
    Do,
    Break,
    Continue,
    Switch,
    Case,
    Default,
    Try,
    Catch,
    Finally,
    Throw,
    New,
    This,
    Class,
    Extends,
    Super,
    Static,
    Import,
    Export,
    Async,
    Await,
    Typeof,
    Instanceof,
    In,
    Of,
    Delete,
    Void,
    Yield,
    
    // Special
    Eof,
    Invalid(String),
}

/// JavaScript lexer
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    current_line: usize,
    current_col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
            current_line: 1,
            current_col: 1,
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.input.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.input.next();
        if let Some(ch) = c {
            if ch == '\n' {
                self.current_line += 1;
                self.current_col = 1;
            } else {
                self.current_col += 1;
            }
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else if c == '/' {
                // Check for comments
                let mut chars = self.input.clone();
                chars.next();
                if let Some(next) = chars.peek() {
                    if *next == '/' {
                        // Line comment
                        while let Some(c) = self.advance() {
                            if c == '\n' {
                                break;
                            }
                        }
                    } else if *next == '*' {
                        // Block comment
                        self.advance(); // /
                        self.advance(); // *
                        while let Some(c) = self.advance() {
                            if c == '*' && self.peek() == Some('/') {
                                self.advance();
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Token {
        let mut num = String::new();
        let mut has_dot = false;
        let mut has_exp = false;

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num.push(self.advance().unwrap());
            } else if c == '.' && !has_dot && !has_exp {
                has_dot = true;
                num.push(self.advance().unwrap());
            } else if (c == 'e' || c == 'E') && !has_exp {
                has_exp = true;
                num.push(self.advance().unwrap());
                if self.peek() == Some('+') || self.peek() == Some('-') {
                    num.push(self.advance().unwrap());
                }
            } else {
                break;
            }
        }

        Token::Number(num.parse().unwrap_or(f64::NAN))
    }

    fn read_string(&mut self, quote: char) -> Token {
        self.advance(); // Skip opening quote
        let mut s = String::new();

        while let Some(c) = self.advance() {
            if c == quote {
                return Token::String(s);
            } else if c == '\\' {
                if let Some(escaped) = self.advance() {
                    match escaped {
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        '\\' => s.push('\\'),
                        '\'' => s.push('\''),
                        '"' => s.push('"'),
                        '0' => s.push('\0'),
                        _ => s.push(escaped),
                    }
                }
            } else {
                s.push(c);
            }
        }

        Token::Invalid("Unterminated string".to_string())
    }

    fn read_identifier(&mut self) -> Token {
        let mut ident = String::new();

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '$' {
                ident.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        // Check for keywords
        match ident.as_str() {
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            "null" => Token::Null,
            "undefined" => Token::Undefined,
            "var" => Token::Var,
            "let" => Token::Let,
            "const" => Token::Const,
            "function" => Token::Function,
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "do" => Token::Do,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "switch" => Token::Switch,
            "case" => Token::Case,
            "default" => Token::Default,
            "try" => Token::Try,
            "catch" => Token::Catch,
            "finally" => Token::Finally,
            "throw" => Token::Throw,
            "new" => Token::New,
            "this" => Token::This,
            "class" => Token::Class,
            "extends" => Token::Extends,
            "super" => Token::Super,
            "static" => Token::Static,
            "import" => Token::Import,
            "export" => Token::Export,
            "async" => Token::Async,
            "await" => Token::Await,
            "typeof" => Token::Typeof,
            "instanceof" => Token::Instanceof,
            "in" => Token::In,
            "of" => Token::Of,
            "delete" => Token::Delete,
            "void" => Token::Void,
            "yield" => Token::Yield,
            _ => Token::Identifier(ident),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let c = match self.peek() {
            Some(c) => c,
            None => return Token::Eof,
        };

        // Numbers
        if c.is_ascii_digit() {
            return self.read_number();
        }

        // Strings
        if c == '"' || c == '\'' || c == '`' {
            return self.read_string(c);
        }

        // Identifiers and keywords
        if c.is_alphabetic() || c == '_' || c == '$' {
            return self.read_identifier();
        }

        // Operators and punctuation
        self.advance();
        match c {
            '+' => {
                if self.peek() == Some('+') {
                    self.advance();
                    Token::PlusPlus
                } else if self.peek() == Some('=') {
                    self.advance();
                    Token::PlusEq
                } else {
                    Token::Plus
                }
            }
            '-' => {
                if self.peek() == Some('-') {
                    self.advance();
                    Token::MinusMinus
                } else if self.peek() == Some('=') {
                    self.advance();
                    Token::MinusEq
                } else {
                    Token::Minus
                }
            }
            '*' => {
                if self.peek() == Some('*') {
                    self.advance();
                    Token::Power
                } else if self.peek() == Some('=') {
                    self.advance();
                    Token::StarEq
                } else {
                    Token::Star
                }
            }
            '/' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::SlashEq
                } else {
                    Token::Slash
                }
            }
            '%' => Token::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::EqEqEq
                    } else {
                        Token::EqEq
                    }
                } else if self.peek() == Some('>') {
                    self.advance();
                    Token::Arrow
                } else {
                    Token::Eq
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        Token::NotEqEq
                    } else {
                        Token::NotEq
                    }
                } else {
                    Token::Not
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::LtEq
                } else if self.peek() == Some('<') {
                    self.advance();
                    Token::Shl
                } else {
                    Token::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    Token::GtEq
                } else if self.peek() == Some('>') {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        Token::UShr
                    } else {
                        Token::Shr
                    }
                } else {
                    Token::Gt
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    Token::And
                } else {
                    Token::BitAnd
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    Token::Or
                } else {
                    Token::BitOr
                }
            }
            '^' => Token::BitXor,
            '~' => Token::BitNot,
            '?' => {
                if self.peek() == Some('?') {
                    self.advance();
                    Token::NullCoalesce
                } else if self.peek() == Some('.') {
                    self.advance();
                    Token::OptionalChain
                } else {
                    Token::Question
                }
            }
            ':' => Token::Colon,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            '.' => {
                if self.peek() == Some('.') {
                    let mut chars = self.input.clone();
                    chars.next();
                    if chars.peek() == Some(&'.') {
                        self.advance();
                        self.advance();
                        Token::Spread
                    } else {
                        Token::Dot
                    }
                } else {
                    Token::Dot
                }
            }
            _ => Token::Invalid(format!("Unexpected character: {}", c)),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if matches!(token, Token::Eof) {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}
