pub mod token;

use std::iter::Peekable;
use std::str::Chars;

use self::token::Token;

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer {
        Lexer { input: input.chars().peekable() }
    }

    fn read_char(&mut self) -> Option<char> {
        self.input.next()
    }

    fn peek_char(&mut self) -> Option<&char> {
        self.input.peek()
    }

    fn read_identifier(&mut self, ch: char) -> Token {
        let content = {
            let mut content = String::new();
            content.push(ch);

            while let Some(&ch) = self.peek_char() {
                if !(ch.is_alphabetic() || ch.is_digit(10)) {
                    break;
                }

                // We call ::unwrap because we already made sure there is an
                // element with the ::peek above.
                content.push(self.read_char().unwrap());
            }

            content
        };

        match &content[..] {
            "tipo" => Token::Type,
            _ => Token::Ident(content),
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(&ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.read_char();
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Token {
        self.consume_whitespace();

        if let Some(ch) = self.read_char() {
            match ch {
                '(' => Token::ParL,
                ')' => Token::ParR,
                ':' => Token::Colon,
                ';' => Token::Semicolon,
                ',' => Token::Comma,
                '\0' => Token::EOF,
                _ => if ch.is_alphabetic() {
                    // As `read_identifier` already increments the read position, we no
                    // longer need `read_char` below, so we perform an early return.
                    return self.read_identifier(ch);
                } else {
                    Token::Illegal
                }
            }
        } else {
            Token::EOF
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Token::EOF => None,
            x => Some(x),
        }
    }
}
