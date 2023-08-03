//! The lexer module.
//!
//! The simpler implementation of a lexer I could think of. It only takes a
//! stream of characters and tansforms it into a tokens one.
//!
//! It doesn't keep track of the current line or column. In a real compiler that
//! information is crucial when you write something wrong and don't know where.

use std::iter::Peekable;
use std::str::Chars;

/// Token types that our language admits.
///
/// All the token types that our little language will need. As it is a very
/// simple language, it doesn't have many. The lexer takes an input and has a
/// token iterator as the output.
///
/// The traits are mostly for tests.
#[derive(Debug, PartialEq)]
pub enum Token {
    /// For anything that we don't recognize.
    Illegal,
    /// Last token, when the input has ended.
    EOF,

    /// Any word: a variable name, a type name...
    Ident(String),

    /// Left parenthesis.
    ParL,
    /// Right parenthesis.
    ParR,
    /// The ':' character.
    Colon,
    /// The ';' character.
    Semicolon,
    /// The ',' character.
    Comma,

    /// The only keyword we have in the language.
    Type,
}

/// The lexer in our language.
///
/// The lexer, also known as tokenizer, transforms the input text into tokens.
pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
}

impl Lexer<'_> {
    /// Create a new lexer.
    ///
    /// To build the lexer, you feed it a `str` with the code. Then you can use
    /// the lexer as a normal `Iterator`, which iterates over [`Token`s][1].
    ///
    /// # Examples
    ///
    ///     use simcom::lexer::{Lexer, Token};
    ///
    ///     let mut tokens = Lexer::new(": tipo");
    ///     assert_eq!(tokens.next().unwrap(), Token::Colon);
    ///     assert_eq!(tokens.next().unwrap(), Token::Type);
    ///     assert_eq!(tokens.next(), None);
    ///
    /// [1]: enum.Token.html
    pub fn new(input: &'_ str) -> Lexer {
        Lexer {
            input: input.chars().peekable(),
        }
    }

    /// The next char in the input. *doesn't* advance the internal iterator.
    #[inline]
    fn peek_char(&mut self) -> Option<&char> {
        self.input.peek()
    }

    /// The next char in the input, advances the internal iterator.
    #[inline]
    fn read_char(&mut self) -> Option<char> {
        self.input.next()
    }

    /// Advances the internal ierator.
    #[inline]
    fn consume_char(&mut self) {
        self.read_char();
    }

    /// If an alphabetic char was found, keep reading chars to build a
    /// identifier. Finally, look if it was a keyword.
    fn read_identifier(&mut self, first_character: char) -> Token {
        let content = {
            // Since in ::next_token we need to call ::read_char the first
            // char was consumed, so we add it here.
            let mut content = String::new();
            content.push(first_character);

            // Keep building the string with allowed characters.
            while let Some(&ch) = self.peek_char() {
                if !(ch.is_alphabetic() || ch.is_digit(10)) {
                    break;
                }

                // We consume the character we previously peeked. Then push it
                // to the string we're building.
                self.consume_char();
                content.push(ch);
            }

            content
        };

        // Match the identifier to all the known keywords to see if it is one
        // of them. Otherwise, return a normal identifier.
        match &content[..] {
            "tipo" => Token::Type,
            _ => Token::Ident(content),
        }
    }

    /// Advance the internal iterator when we find whitespace.
    fn consume_whitespace(&mut self) {
        while let Some(&ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.read_char();
            } else {
                break;
            }
        }
    }

    /// The basis for the iterator, matches the characters to Tokens.
    fn next_token(&mut self) -> Token {
        self.consume_whitespace();
        match self.read_char() {
            Some('(') => Token::ParL,
            Some(')') => Token::ParR,
            Some(':') => Token::Colon,
            Some(';') => Token::Semicolon,
            Some(',') => Token::Comma,
            Some('\0') => Token::EOF,
            // Read the remaining part of the identifier, passing its
            // first character, as we already consumed it.
            Some(ch) if ch.is_alphabetic() => self.read_identifier(ch),
            Some(_) => Token::Illegal,
            // If the internal iterator has given us a None, that means there are no
            // characters left. In other words, EOF was reached.
            None => Token::EOF,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Token::EOF => None,
            x => Some(x),
        }
    }
}
