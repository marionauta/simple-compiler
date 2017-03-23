#[derive(Debug, PartialEq)]
pub enum Token {
    Illegal,
    EOF,
    
    Ident(String),

    ParL,
    ParR,
    Colon,
    Semicolon,
    Comma,
    
    Type,
}
