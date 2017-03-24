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

impl<'a> Lexer<'a> {
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
    pub fn new(input: &'a str) -> Lexer {
        Lexer { input: input.chars().peekable() }
    }

    /// The next char in the input, advances the internal iterator.
    fn read_char(&mut self) -> Option<char> {
        self.input.next()
    }

    /// The next char in the input. *doesn't* advance the internal iterator.
    fn peek_char(&mut self) -> Option<&char> {
        self.input.peek()
    }

    /// If an alphabetic char was found, keep reading chars to build a
    /// identifier. Finally, look if it was a keyword.
    fn read_identifier(&mut self, ch: char) -> Token {
        let content = {
            // Since in ::next_token we need to call ::read_char the first
            // char was consumed, so we add it here.
            let mut content = String::new();
            content.push(ch);

            // Keep building the string with allowed characters.
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

        if let Some(ch) = self.read_char() {
            match ch {
                '(' => Token::ParL,
                ')' => Token::ParR,
                ':' => Token::Colon,
                ';' => Token::Semicolon,
                ',' => Token::Comma,
                '\0' => Token::EOF,
                _ => if ch.is_alphabetic() {
                    // As ::read_identifier already advances the iterator, we
                    // can safely perform an early return.
                    return self.read_identifier(ch);
                } else {
                    Token::Illegal
                }
            }
        // If the internal iterator has given us a None, that means there are no
        // characters left. In other words, EOF was reached.
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
