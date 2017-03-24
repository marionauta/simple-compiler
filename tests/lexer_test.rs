extern crate simcom;

use simcom::lexer::{Lexer, Token};

fn get_tokens(input: &str) -> Vec<Token> {
    Lexer::new(input).collect()
}

#[test]
fn detect_tokens() {
    let tokens = get_tokens("():;");
    assert_eq!(tokens,
               vec![Token::ParL, Token::ParR, Token::Colon, Token::Semicolon]);
}

#[test]
fn detect_tokens_whitespace() {
    let tokens = get_tokens("(     :    )");
    assert_eq!(tokens, vec![Token::ParL, Token::Colon, Token::ParR]);
}

#[test]
fn detect_keyword() {
    let tokens = get_tokens("tipo:: tipo)");
    assert_eq!(tokens,
               vec![Token::Type, Token::Colon, Token::Colon, Token::Type, Token::ParR]);
}

#[test]
fn detect_illegal() {
    let tokens = get_tokens("( ! tipo   :!tipo");
    assert_eq!(tokens,
               vec![Token::ParL,
                    Token::Illegal,
                    Token::Type,
                    Token::Colon,
                    Token::Illegal,
                    Token::Type]);
}

#[test]
fn identifiers() {
    let tokens = get_tokens("tipo pal4abra castaña");
    assert_eq!(tokens,
               vec![Token::Type,
                    Token::Ident(String::from("pal4abra")),
                    Token::Ident(String::from("castaña"))]);
}
