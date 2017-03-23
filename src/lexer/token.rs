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
