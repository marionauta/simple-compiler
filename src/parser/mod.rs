use std::iter::Peekable;

use super::lexer::{Lexer, Token};

#[derive(Debug, PartialEq)]
pub enum Ast {
    TypeDefinition(String, Vec<Ast>),
    Parameter(String, String),
    Unexpected(Token),
    Empty,
}

type ParseResult = Result<Ast, Token>;

/// The parser in out language.
///
/// The parser transforms the input tokens into an AST.
pub struct Parser<'a> {
    tokens: Peekable<Lexer<'a>>,
}

impl Parser<'_> {
    /// Create a new parser.
    ///
    /// To build the parser, you need a [`Lexer`][0] with tokens. Then you can
    /// use the parser as a normal `Iterator`, wich iterates over [`Ast`s][1].
    ///
    /// # Examples
    ///
    ///     use simcom::lexer::{Lexer, Token};
    ///     use simcom::parser::{Ast, Parser};
    ///
    ///     let mut parser = Parser::new(Lexer::new("? Hello World"));
    ///     assert_eq!(parser.next().unwrap(), Ast::Unexpected(Token::Illegal));
    ///
    /// [0]: ../lexer/struct.Lexer.html
    /// [1]: enum.Ast.html
    pub fn new(tokens: Lexer) -> Parser {
        Parser {
            tokens: tokens.peekable(),
        }
    }

    /// The next Token in the input. *doesn't* advance the internal iterator.
    #[inline]
    fn peek_token(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    /// The next Token in the input, advances the internal iterator.
    #[inline]
    fn read_token(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    /// Advances the internal ierator.
    #[inline]
    fn consume_token(&mut self) {
        self.tokens.next();
    }

    /// Advances the iterator until a semicolon is found, consuming it.
    /// Also, if we find a 'None' value, we stop because otherwise we will
    /// get stuck in a never ending loop.
    fn advance_until_semicolon(&mut self) {
        match self.read_token() {
            None | Some(Token::Semicolon) => (),
            _ => self.advance_until_semicolon(),
        };
    }

    /// Matches an entire type definition. From Token::Type to Token::Semicolon.
    /// Returns an Ast::TypeDefinition if everything went ok. Otherwise we get
    /// the Token that was misplaced (thus unexpected).
    fn parse_definition(&mut self) -> ParseResult {
        match self.read_token() {
            Some(Token::Type) => (),
            Some(t) => return Err(t),
            None => return Err(Token::EOF),
        }

        // Get the type's name from the first identifier.
        let name = match self.read_token() {
            Some(Token::Ident(name)) => name,
            Some(t) => return Err(t),
            None => return Err(Token::EOF),
        };

        match self.read_token() {
            Some(Token::ParL) => (),
            Some(t) => return Err(t),
            None => return Err(Token::EOF),
        }

        let parameters = match self.parse_parameters() {
            Ok(parameters) => parameters,
            Err(token) => return Err(token),
        };

        match self.read_token() {
            Some(Token::ParR) => (),
            Some(t) => return Err(t),
            None => return Err(Token::EOF),
        }

        match self.read_token() {
            Some(Token::Semicolon) => (),
            Some(t) => return Err(t),
            None => return Err(Token::EOF),
        }

        Ok(Ast::TypeDefinition(name, parameters))
    }

    /// Matches a series of parameters, separated by a comma (Token::Comma).
    ///
    /// Return is Err(Token) when an unexpected token was found or when the
    /// internal 'tokens' iterator ends.
    fn parse_parameters(&mut self) -> Result<Vec<Ast>, Token> {
        let parameter = match self.parse_parameter() {
            Ok(parameter) => parameter,
            Err(token) => return Err(token),
        };
        match self.peek_token() {
            Some(Token::Comma) => {
                self.consume_token();
                match self.peek_token() {
                    Some(Token::ParR) => Ok(vec![parameter]),
                    _ => match self.parse_parameters() {
                        Ok(mut parameters) => {
                            parameters.insert(0, parameter);
                            Ok(parameters)
                        }
                        Err(token) => Err(token),
                    },
                }
            }
            _ => Ok(vec![parameter]),
        }
    }

    /// Matches a parameter (the ones inside the type definition's parenthesis).
    /// Has the form (Token::Ident, Token::Colon, Token::Ident).
    fn parse_parameter(&mut self) -> ParseResult {
        match self.read_token() {
            Some(Token::Ident(name)) => match self.read_token() {
                Some(Token::Colon) => match self.read_token() {
                    Some(Token::Ident(tname)) => Ok(Ast::Parameter(name, tname)),
                    Some(x) => Err(x),
                    None => Err(Token::EOF),
                },
                Some(x) => Err(x),
                None => Err(Token::EOF),
            },
            Some(x) => Err(x),
            None => Err(Token::EOF),
        }
    }
}

impl Iterator for Parser<'_> {
    type Item = Ast;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse_definition() {
            Ok(ast) => Some(ast),
            Err(Token::EOF) => None,
            Err(token) => {
                self.advance_until_semicolon();
                Some(Ast::Unexpected(token))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::lexer::Lexer;
    use super::*;

    fn get_parser(input: &str) -> Parser {
        Parser::new(Lexer::new(input))
    }

    fn get_parameter(input: &str) -> ParseResult {
        let mut parser = get_parser(input);
        parser.parse_parameter()
    }

    fn get_parameters(input: &str) -> Result<Vec<Ast>, Token> {
        let mut parser = get_parser(input);
        parser.parse_parameters()
    }

    fn get_definition(input: &str) -> ParseResult {
        let mut parser = get_parser(input);
        parser.parse_definition()
    }

    #[test]
    fn semicolon1() {
        let mut parser = get_parser("Hola ,,();  )");
        parser.advance_until_semicolon();
        assert_eq!(parser.read_token().unwrap(), Token::ParR);
    }

    #[test]
    fn semicolon2() {
        let mut parser = get_parser(",,,,,,;;");
        parser.advance_until_semicolon();
        assert_eq!(parser.read_token().unwrap(), Token::Semicolon);
    }

    #[test]
    fn semicolon3() {
        let mut parser = get_parser("Hola ,,(); ");
        parser.advance_until_semicolon();
        assert_eq!(parser.read_token(), None);
    }

    #[test]
    fn semicolon4() {
        let mut parser = get_parser("Hey");
        parser.advance_until_semicolon();
        assert_eq!(parser.read_token(), None);
    }

    #[test]
    fn good_definition() {
        let d = get_definition("tipo Punto(x: Punto);");

        assert_eq!(
            d.unwrap(),
            Ast::TypeDefinition(
                String::from("Punto"),
                vec![Ast::Parameter(String::from("x"), String::from("Punto"),)],
            )
        );
    }

    #[test]
    fn good_definition_trailing_comma() {
        let d = get_definition("tipo Punto(x: Punto,);");

        assert_eq!(
            d.unwrap(),
            Ast::TypeDefinition(
                String::from("Punto"),
                vec![Ast::Parameter(String::from("x"), String::from("Punto"),)],
            )
        );
    }

    #[test]
    fn good_definition_many() {
        let d = get_definition("tipo Punto(x: Punto, x: P, x: P);");

        assert_eq!(
            d.unwrap(),
            Ast::TypeDefinition(
                String::from("Punto"),
                vec![
                    Ast::Parameter(String::from("x"), String::from("Punto"),),
                    Ast::Parameter(String::from("x"), String::from("P"),),
                    Ast::Parameter(String::from("x"), String::from("P"),),
                ],
            )
        );
    }

    #[test]
    fn good_definition_many_trailing_comma() {
        let d = get_definition("tipo Punto(x: Punto, x: P, x: P,);");

        assert_eq!(
            d.unwrap(),
            Ast::TypeDefinition(
                String::from("Punto"),
                vec![
                    Ast::Parameter(String::from("x"), String::from("Punto"),),
                    Ast::Parameter(String::from("x"), String::from("P"),),
                    Ast::Parameter(String::from("x"), String::from("P"),),
                ],
            )
        );
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

        assert_eq!(
            p.unwrap(),
            Ast::Parameter(String::from("name"), String::from("Type"),)
        );
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
        let result = get_parameters("name: Type, other: othert");
        assert_eq!(
            result.unwrap(),
            vec![
                Ast::Parameter(String::from("name"), String::from("Type"),),
                Ast::Parameter(String::from("other"), String::from("othert"),),
            ]
        );
    }

    #[test]
    fn missing_comma() {
        let result = get_parameters("name: Type other: othert");
        // Here the function doesn't report the next identifier token because
        // it ends if it doesn't find any Token::Comma's. The unexpected "other"
        // identifier will be reported by the ::definition function.
        assert_eq!(
            result.unwrap(),
            vec![Ast::Parameter(String::from("name"), String::from("Type"),),]
        );
    }

    #[test]
    fn missing_colon_parameters() {
        let result = get_parameters("name Type, other: othert");
        // Error propagates from ::parameter to ::parameters.
        assert_eq!(result.unwrap_err(), Token::Ident(String::from("Type")));
    }
}
