//! Semantic analyzer module.
//!
//! Usually a semantic analyzer checks if everything is in order (variables
//! are in scope, type inference...) before tying to compile the program.
//!
//! The aim with this one, for simplicity, is to walk dependencies (including
//! cyclic ones) to determine in what order they must be written.

use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;

use super::lexer::Token;
use super::parser::{Ast, Parser};

/// Value returned in [`Semantic::analyze`][0] if any errors are found.
///
/// In this analyzer, we only return error if we find any unexpected tokens.
/// Here we store all of them.
///
/// [0]: struct.Semantic.html#method.analyze
pub type UnexpectedTokens = Vec<Token>;

/// The semantic analyzer in our language.
///
/// Given an AST (from a [`Parser`][0]) determines the order in which all
/// definitions must go.
///
/// [0]: ../parser/struct.Parser.html
pub struct Semantic {
    /// All the type definitions. Since we consumed the AST, we have to store
    /// them somewhere.
    pub definitions: HashMap<String, Vec<(String, String)>>,
    /// The order in which to write the definitions.
    pub order: Vec<String>,
    /// If any cyclic dependency is found, all the types involved are stored
    /// here, so they can be handled accordingly.
    pub cycles: HashSet<String>,
}

impl Semantic {
    /// Builds the semantic analyzer and analyzes the AST.
    ///
    /// # Examples
    ///
    /// ```
    /// use simcom::lexer::Lexer;
    /// use simcom::parser::Parser;
    /// use simcom::semantic::Semantic;
    ///
    /// let content = "tipo A(x: X);";
    /// let ast = Parser::new(Lexer::new(content));
    ///
    /// if let Ok(s) = Semantic::analyze(ast) {
    ///     assert_eq!(s.order[0], String::from("X"));
    ///     assert_eq!(s.order[1], String::from("A"));
    /// } else {
    ///     panic!("Wrong if/else branch!");
    /// }
    /// ```
    ///
    /// ```
    /// use simcom::lexer::{Lexer, Token};
    /// use simcom::parser::Parser;
    /// use simcom::semantic::Semantic;
    ///
    /// // Note the two semicolons:
    /// let content = "tipo A(x: X);;";
    /// let s = Semantic::analyze(Parser::new(Lexer::new(content)));
    ///
    /// if let Err(ve) = s {
    ///     assert_eq!(ve[0], Token::Semicolon);
    /// } else {
    ///     panic!("Wrong if/else branch!");
    /// }
    /// ```
    pub fn analyze(ast: Parser) -> Result<Semantic, UnexpectedTokens> {
        let mut definitions = HashMap::new();
        let mut errors = Vec::new();

        for definition in ast {
            match definition {
                Ast::TypeDefinition(name, parameters) => {
                    definitions.insert(name, build_parameters(parameters));
                },
                Ast::Unexpected(token) => errors.push(token),
                _ => unreachable!(),
            }
        }

        match errors.is_empty() {
            true => SemanticBuilder::build(definitions),
            false => Err(errors),
        }
    }
}

struct SemanticBuilder {
    definitions: HashMap<String, Vec<(String, String)>>,

    order: Vec<String>,
    visited: HashSet<String>,
    cycles: HashSet<String>,
}

impl SemanticBuilder {
    fn build(definitions: HashMap<String, Vec<(String, String)>>) -> Result<Semantic, UnexpectedTokens> {
        let mut sb = SemanticBuilder {
            definitions,
            order: Vec::new(),
            visited: HashSet::new(),
            cycles: HashSet::new(),
        };

        for node in sb.definitions.clone().keys() {
            sb.visit(node);
        }

        Ok(Semantic {
            definitions: sb.definitions,
            order: sb.order,
            cycles: sb.cycles,
        })
    }

    fn visit(&mut self, node: &String) {
        if self.order.contains(node) {
            return;
        } else if self.visited.contains(node) {
            self.cycles = self.visited.clone();
            return;
        }

        self.visited.insert(node.clone());

        if let Some(d) = self.definitions.clone().get(node) {
            for &(_, ref v) in d {
                self.visit(v);
            }
        }

        if !self.cycles.contains(node) {
            self.order.push(node.clone());
            self.visited.remove(node);
        }
    }
}

fn ast_to_parameter(ast: Ast) -> Option<(String, String)> {
    match ast {
        Ast::Parameter(name, typename) => Some((name, typename)),
        _ => None,
    }
}

fn build_parameters(ast: Vec<Ast>) -> Vec<(String, String)> {
    ast.into_iter()
        .filter_map(ast_to_parameter)
        .collect()
}
