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

/// Parse a token stream into an AST.
pub fn parser<'a>(mut tokens: Peekable<Lexer<'a>>) -> (Ast, bool) {
    let mut definitions = Vec::new();
    let mut had_errors = false;

    while let Some(_) = tokens.peek() {
        match definition(&mut tokens) {
            Ok(ast) => definitions.push(ast),
            Err(token) => {
                println!("Unexpected {:?} token.", token);
                had_errors = true;
                definitions.push(Ast::Unexpected(token));
                advance_until_semicolon(&mut tokens)
            }
        }
    }

    (Ast::Program(definitions), had_errors)
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

/// Matches an entire type definition. From Token::Type to Token::Semicolon.
/// Returns an Ast::TypeDefinition if everything went ok. Otherwise we get the
/// Token that was misplaced (thus unexpected).
///
/// TODO: Try to write this function a bit nicer.
fn definition<'a>(mut tokens: &mut Peekable<Lexer<'a>>) -> ParseResult {
    match tokens.next() {
        Some(Token::Type) => (),
        Some(t) => return Err(t),
        None => return Err(Token::EOF),
    }

    // Get the type's name from the first identifier.
    let name = match tokens.next() {
        Some(Token::Ident(name)) => name,
        Some(t) => return Err(t),
        None => return Err(Token::EOF),
    };

    match tokens.next() {
        Some(Token::ParL) => (),
        Some(t) => return Err(t),
        None => return Err(Token::EOF),
    }

    // Fill a parameter vector with the different vectors we find.
    let mut pars = Vec::new();
    if let Err(t) = parameters(&mut tokens, &mut pars) {
        return Err(t);
    }

    match tokens.next() {
        Some(Token::ParR) => (),
        Some(t) => return Err(t),
        None => return Err(Token::EOF),
    }

    match tokens.next() {
        Some(Token::Semicolon) => (),
        Some(t) => return Err(t),
        None => return Err(Token::EOF),
    }

    Ok(Ast::TypeDefinition(name, pars))
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

    fn get_parameters(input: &str) -> (ParseResult, Vec<Ast>) {
        let mut pars = Vec::new();
        let r = parameters(&mut Lexer::new(input).peekable(), &mut pars);

        (r, pars)
    }

    fn get_definition(input: &str) -> ParseResult {
        definition(&mut Lexer::new(input).peekable())
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
    fn good_definition() {
        let d = get_definition("tipo Punto(x: Punto);");

        assert_eq!(d.unwrap(), Ast::TypeDefinition(
            String::from("Punto"),
            vec![Ast::Parameter(
                String::from("x"),
                String::from("Punto"),
            )],
        ));
    }

    #[test]
    fn good_definition_many() {
        let d = get_definition("tipo Punto(x: Punto, x: P, x: P);");

        assert_eq!(d.unwrap(), Ast::TypeDefinition(
            String::from("Punto"),
            vec![
                Ast::Parameter(
                    String::from("x"),
                    String::from("Punto"),
                ),
                Ast::Parameter(
                    String::from("x"),
                    String::from("P"),
                ),
                Ast::Parameter(
                    String::from("x"),
                    String::from("P"),
                ),
            ],
        ));
    }

    #[test]
    fn missing_keyword() {
        let d = get_definition("tiipo Punto");
        assert_eq!(d.unwrap_err(), Token::Ident(String::from("tiipo")));
    }

    #[test]
    fn missing_identifier_definition() {
        let d = get_definition("tipo (,,,");
        assert_eq!(d.unwrap_err(), Token::ParL);
    }

    #[test]
    fn missing_parenthesis() {
        let d = get_definition("tipo x he");
        assert_eq!(d.unwrap_err(), Token::Ident(String::from("he")));

        let d = get_definition("tipo P(x: haha;");
        assert_eq!(d.unwrap_err(), Token::Semicolon);
    }

    #[test]
    fn missing_semicolon() {
        let d = get_definition("tipo P(x: E)");
        assert_eq!(d.unwrap_err(), Token::EOF);

        let d = get_definition("tipo P(x: E) \n tipo");
        assert_eq!(d.unwrap_err(), Token::Type);
    }

    #[test]
    fn error_propagation() {
        let d = get_definition("tipo Punto(x Punto);");
        assert_eq!(d.unwrap_err(), Token::Ident(String::from("Punto")));
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
        let (res, ps) = get_parameters("name: Type, other: othert");

        assert_eq!(res.unwrap(), Ast::Empty);
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
        let (res, ps) = get_parameters("name: Type other: othert");

        // Here the function doesn't report the next identifier token because
        // it ends if it doesn't find any Token::Comma's. The unexpected "other"
        // identifier will be reported by the ::definition function.
        assert_eq!(res.unwrap(), Ast::Empty);
        assert_eq!(ps, vec![
            Ast::Parameter(
                String::from("name"),
                String::from("Type"),
            ),
        ]);
    }

    #[test]
    fn missing_colon_parameters() {
        let (res, _) = get_parameters("name Type, other: othert");

        // Error propagates from ::parameter to ::parameters.
        assert_eq!(res.unwrap_err(), Token::Ident(String::from("Type")));
    }
}
