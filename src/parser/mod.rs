use std::iter::Peekable;

use super::lexer::{Lexer, Token};

#[derive(Debug, PartialEq)]
pub enum Ast {
    Program(Vec<Ast>),
    TypeDefinition(String, Vec<Ast>),
    Parameter(String, String),
    Unexpected(Token),
    Empty,
}

type ParseResult = Result<Ast, Token>;

pub fn parser<'a>(mut tokens: Peekable<Lexer<'a>>) -> Ast {
    let mut definitions = Vec::new();

    while let Some(_) = tokens.peek() {
        match definition(&mut tokens) {
            Ok(ast) => definitions.push(ast),
            Err(token) => {
                println!("Unexpected {:?} token.", token);
                definitions.push(Ast::Unexpected(token));
                advance_until_semicolon(&mut tokens)
            }
        }
    }

    Ast::Program(definitions)
}

/// Advances the iterator until a semicolon is found, consuming it.
/// Also, if we find a 'None' value, we stop because otherwise we will get stuck
/// in a never ending loop.
fn advance_until_semicolon<'a>(mut tokens: &mut Peekable<Lexer<'a>>) {   
    match tokens.next() {
        None | Some(Token::Semicolon) => (),
        _ => advance_until_semicolon(&mut tokens),
    }
}

fn definition<'a>(mut tokens: &mut Peekable<Lexer<'a>>) -> ParseResult {
    if let Some(Token::Type) = tokens.next() {
        if let Some(Token::Ident(name)) = tokens.next() {
            tokens.next(); // Skip the '('

            let mut pars = Vec::new();
            if let Err(t) = parameters(&mut tokens, &mut pars) {
                return Err(t);
            }

            tokens.next(); // Skip the ')'
            tokens.next(); // Skip the ';'

            return Ok(Ast::TypeDefinition(name, pars));
        }
    }

    Err(Token::Type)
}

/// Matches a series of parameters, separated by a comma (Token::Comma).
///
/// Fills the passed 'res' vector. Return is Err(_) when an unexpected token was
/// found or when the 'tokens' iterator ends.
fn parameters<'a>(mut tokens: &mut Peekable<Lexer<'a>>,
    mut res: &mut Vec<Ast>) -> ParseResult {

    match parameter(tokens) {
        Ok(x) => res.push(x),
        Err(token) => return Err(token),
    }

    match tokens.peek() {
        Some(&Token::Comma) => {
            tokens.next(); // Consume Token::Comma.
            parameters(&mut tokens, &mut res)
        },
        _ => Ok(Ast::Empty)
    }
}

/// Matches a parameter (the ones inside the type definition's parenthesis).
/// Has the form (Token::Ident, Token::Colon, Token::Ident).
fn parameter<'a>(mut tokens: &mut Peekable<Lexer<'a>>) -> ParseResult {
    match tokens.next() {
        Some(Token::Ident(name)) => match tokens.next() {
            Some(Token::Colon) => match tokens.next() {
                Some(Token::Ident(tname)) => Ok(Ast::Parameter(name, tname)),
                None => Err(Token::EOF),
                Some(x) => Err(x),
            },
            None => Err(Token::EOF),
            Some(x) => Err(x),
        },
        None => Err(Token::EOF),
        Some(x) => Err(x),
    }
}

#[cfg(test)]
mod test {
    use super::super::lexer::Lexer;
    use super::*;

    fn get_parameter(input: &str) -> ParseResult {
        parameter(&mut Lexer::new(input).peekable())
    }

    fn get_parameters(input: &str) -> Vec<Ast> {
        let mut pars = Vec::new();
        let _ = parameters(&mut Lexer::new(input).peekable(), &mut pars);

        pars
    }

    #[test]
    fn semicolon1() {
        let mut tokens = Lexer::new("Hola ,,();  )").peekable();
        advance_until_semicolon(&mut tokens);

        assert_eq!(tokens.next().unwrap(), Token::ParR);
    }

    #[test]
    fn semicolon2() {
        let mut tokens = Lexer::new(",,,,,,;;").peekable();
        advance_until_semicolon(&mut tokens);

        assert_eq!(tokens.next().unwrap(), Token::Semicolon);
    }

    #[test]
    fn semicolon3() {
        let mut tokens = Lexer::new("Hola ,,(); ").peekable();
        advance_until_semicolon(&mut tokens);

        assert_eq!(tokens.next(), None);
    }

    #[test]
    fn semicolon4() {
        let mut tokens = Lexer::new("Hey").peekable();
        advance_until_semicolon(&mut tokens);

        assert_eq!(tokens.next(), None);
    }

    #[test]
    fn good_parameter() {
        let p = get_parameter("name: Type");

        assert_eq!(p.unwrap(), Ast::Parameter(
            String::from("name"),
            String::from("Type"),
        ));
    }

    #[test]
    fn missing_colon() {
        let p = get_parameter("name Type");

        assert_eq!(p.unwrap_err(), Token::Ident(String::from("Type")));
    }

    #[test]
    fn missing_identifier() {
        let p = get_parameter("name: )");

        assert_eq!(p.unwrap_err(), Token::ParR);
    }

    #[test]
    fn good_parameters() {
        let ps = get_parameters("name: Type, other: othert");

        assert_eq!(ps, vec![
            Ast::Parameter(
                String::from("name"),
                String::from("Type"),
            ),
            Ast::Parameter(
                String::from("other"),
                String::from("othert"),
            ),
        ]);
    }

    #[test]
    fn missing_comma() {
        let ps = get_parameters("name: Type other: othert");

        assert_eq!(ps, vec![
            Ast::Parameter(
                String::from("name"),
                String::from("Type"),
            ),
        ]);
    }
}