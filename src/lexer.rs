use crate::token::{Token, TokenType};
use std::collections::HashMap;

#[derive(Debug)]
pub enum LexerError {
    UnknownToken(String),
    MultiLine(String),
    ReadString(String),
    ReadIdentifier(String),
    ReadNumber(String),
}

pub struct Lexer {
    pub input: String,
    pub position: usize,
    pub read_position: usize,
    pub ch: char,
    pub keyword: HashMap<String, TokenType>,
    pub end: bool,
}

impl Lexer {
    pub fn init_lex(input: String) -> Self {
        let keys = generate_keywords();
        let c = input.as_bytes()[0].clone();
        Lexer {
            input: input,
            position: 0,
            read_position: 1,
            ch: c as char,
            keyword: keys,
            end: false,
        }
    }

    pub fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.end = true;
            return;
        } else {
            self.ch = self.input.as_bytes()[self.read_position] as char;
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        let mut tok: Token = Token {
            literal: "".to_owned(),
            kind: TokenType::ILLEGAL,
        };
        while self.ch.is_whitespace() {
            self.read_char();
        }
        if self.end == true {
            tok = self.new_token("\0".to_owned(), TokenType::EOF);
            return Ok(tok);
        }
        // INFO: whitespaces are importart in Scheme so skipping whitespaces at the beginning
        // only to avoid the default error
        match self.ch {
            '#' => {
                if self.peek_ch() == 'f' {
                    tok = self.new_token("#f".to_owned(), TokenType::FALSE);
                    self.read_char();
                } else if self.peek_ch() == 't' {
                    tok = self.new_token("#t".to_owned(), TokenType::TRUE);
                    self.read_char();
                } else {
                    tok = self.new_token("#".to_owned(), TokenType::POUND);
                }
            }
            ',' => tok = self.new_token(",".to_owned(), TokenType::COMMA),
            '@' => tok = self.new_token("@".to_owned(), TokenType::AT),
            // Conditionals
            '=' => tok = self.new_token("=".to_owned(), TokenType::EQ),
            '<' => {
                if self.peek_ch() == '=' {
                    self.read_char();
                    tok = self.new_token("<=".to_owned(), TokenType::LTEQ);
                } else {
                    tok = self.new_token("<".to_owned(), TokenType::LT);
                }
            }
            '>' => {
                if self.peek_ch() == '=' {
                    self.read_char();
                    tok = self.new_token(">=".to_owned(), TokenType::GTEQ);
                } else {
                    tok = self.new_token(">".to_owned(), TokenType::GT);
                }
            }

            // Arithmetic Operations
            '/' => tok = self.new_token("/".to_owned(), TokenType::SLASH),
            '+' => tok = self.new_token("+".to_owned(), TokenType::PLUS),
            '*' => tok = self.new_token("*".to_owned(), TokenType::ASTERICK),
            // List
            '(' => tok = self.new_token("(".to_owned(), TokenType::LPAREN),
            ')' => tok = self.new_token(")".to_owned(), TokenType::RPAREN),
            '\'' => tok = self.new_token("\'".to_owned(), TokenType::QUOTE),

            // Comments: following version 2 of write you a scheme (I think this is how comments are written in Haskell)
            // FIXME: semicolons are supposed to be used for comments in scheme
            // FIXME: also minus can lead to a negative number only if the next char is not a whitespace
            '-' => {
                // Check if single line comment
                if self.peek_ch() == '-' {
                    self.read_single_line_comment();
                    tok = self.new_token("single-line-comment".to_owned(), TokenType::COMMENT);
                } else if self.peek_ch().is_numeric() {
                    tok.kind = TokenType::INT;
                    let num = self.read_number();
                    match num {
                        Ok(s) => tok.literal = s,
                        Err(s) => return Err(s),
                    }
                } else {
                    tok = self.new_token("-".to_owned(), TokenType::MINUS);
                }
            }
            '{' => {
                if self.peek_ch() != '-' {
                    return Err(LexerError::MultiLine(
                        "Multiline comment must start with a bracket followed by a minus symbol {-"
                            .to_owned(),
                    ));
                } else {
                    self.read_char();
                    self.read_char(); // two are need to consume - and move to next minus char
                    self.read_multiline_comment();
                }
            }

            // Read String
            '"' => {
                tok.kind = TokenType::STRING;
                let str = self.read_string();
                match str {
                    Ok(s) => tok.literal = s,
                    Err(s) => return Err(s),
                }
            }
            _ => {
                if self.ch.is_alphabetic() {
                    let str = self.read_identifier();
                    match str {
                        Ok(s) => tok.literal = s,
                        Err(s) => return Err(s),
                    }
                    tok.kind = self.lookup_identifier(tok.literal.as_str()); //self.lookup_identifier(tok.literal.as_str());
                } else if self.ch.is_numeric() {
                    tok.kind = TokenType::INT;
                    let num = self.read_number();
                    match num {
                        Ok(s) => tok.literal = s,
                        Err(s) => return Err(s),
                    }
                } else {
                    return Err(LexerError::UnknownToken(self.ch.to_string()));
                }
            }
        }
        self.read_char();
        Ok(tok)
    }

    // Helper function to create tokens
    #[inline(always)]
    fn new_token(&self, lit: String, kind: TokenType) -> Token {
        Token { literal: lit, kind }
    }

    // Helper function to peek at next char (should be the read position)
    #[inline(always)]
    fn peek_ch(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input.as_bytes()[self.read_position] as char
        }
    }

    // Helper function to skip through single line comments
    #[inline(always)]
    fn read_single_line_comment(&mut self) {
        while self.ch != '\n' {
            self.read_char();
        }
    }

    // Helper function to skip through multiline comments
    fn read_multiline_comment(&mut self) {
        while self.ch != '-' {
            self.read_char();
        }
        if self.peek_ch() == '}' {
            self.read_char();
            return;
        } else {
            self.read_multiline_comment();
        }
    }

    fn read_string(&mut self) -> Result<String, LexerError> {
        let pos = self.position + 1; // After the beginning quotation
        self.read_char(); // Consume beginning quotation
                          // TODO: Check if a loop is a good choice for this part
                          // Used in Monkey lang lexer not sure its a good idea here
        loop {
            if self.ch == '"' || self.end {
                break;
            }
            self.read_char();
        }
        let buf = &(self.input.as_bytes()[pos..self.position]);
        let x = std::str::from_utf8(buf);
        match x {
            Ok(s) => Ok(s.to_owned()),
            Err(err) => Err(LexerError::ReadString(err.to_string())),
        }
    }

    fn read_identifier(&mut self) -> Result<String, LexerError> {
        let pos = self.position;
        while self.ch.is_alphanumeric() {
            self.read_char();
        }
        let buf = &(self.input.as_bytes()[pos..self.position]);
        let x = std::str::from_utf8(buf);
        self.position -= 1;
        self.read_position -= 1;
        match x {
            Ok(s) => return Ok(s.to_owned()),
            Err(err) => Err(LexerError::ReadIdentifier(err.to_string())),
        }
    }

    fn lookup_identifier(&self, identifier: &str) -> TokenType {
        let result = self.keyword.get(identifier);
        match result {
            Some(x) => *x,
            None => TokenType::IDENT,
        }
    }

    fn read_number(&mut self) -> Result<String, LexerError> {
        let pos = self.position;
        self.read_char();
        while self.ch.is_numeric() {
            self.read_char();
        }
        let buf = &(self.input.as_bytes()[pos..self.position]);
        let x = std::str::from_utf8(buf);
        // Decrement position and read position by one to compensate for the whole number for
        // being consumed
        self.position -= 1;
        self.read_position -= 1;
        match x {
            Ok(s) => return Ok(s.to_owned()),
            Err(err) => Err(LexerError::ReadNumber(err.to_string())),
        }
    }
}

fn generate_keywords() -> HashMap<String, TokenType> {
    let keys = HashMap::from([
        ("let".to_owned(), TokenType::LET),
        ("lambda".to_owned(), TokenType::LAMBDA),
        ("if".to_owned(), TokenType::IF),
        ("#t".to_owned(), TokenType::TRUE),
        ("#f".to_owned(), TokenType::FALSE),
        ("\'".to_owned(), TokenType::QUOTE),
        ("quote".to_owned(), TokenType::QUOTE),
        ("define".to_owned(), TokenType::DEFINE),
        ("begin".to_owned(), TokenType::BEGIN),
        ("else".to_owned(), TokenType::ELSE),
        ("and".to_owned(), TokenType::AND),
        ("or".to_owned(), TokenType::OR),
        ("not".to_owned(), TokenType::NOT),
    ]);
    keys
}
