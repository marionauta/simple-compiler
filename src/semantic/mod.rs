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
    pub fn analyze(ast: Parser) -> Result<Self, UnexpectedTokens> {
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

        match errors.len() {
            0 => SemanticBuilder::build(definitions),
            _ => Err(errors),
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
        let mut sb = Self {
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

        self.order.push(node.clone());
        self.visited.remove(node);
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

#[cfg(test)]
mod test {
    use super::super::lexer::*;
    use super::super::parser::*;
    use super::*;

    fn get_semantic(content: &str) -> Result<Semantic, UnexpectedTokens> {
        Semantic::analyze(Parser::new(Lexer::new(content)))
    }

    #[test]
    fn order_ok() {
        let content = "tipo A(x: long);\
        tipo B(a: A);";

        let a = get_semantic(content).unwrap();

        assert_eq!(a.order[0], String::from("long"));
        assert_eq!(a.order[1], String::from("A"));
        assert_eq!(a.order[2], String::from("B"));
    }

    #[test]
    fn cycle() {
        let content = "tipo A(x: B);\
        tipo B (x: A);";

        let s = get_semantic(content).unwrap();

        assert!(s.cycles.contains("A"));
        assert!(s.cycles.contains("B"));
    }

    #[test]
    fn cycle_order_1() {
        let content = "tipo A(x: B);\
        tipo B(x: A);\
        tipo C(a: A, b: B);";

        let s = get_semantic(content).unwrap();

        assert_eq!(s.order[2], String::from("C"));
        assert!(s.cycles.contains("A"));
        assert!(s.cycles.contains("B"));
    }

    #[test]
    fn cycle_order_2() {
        let content = "tipo A(x: B, c: C);\
        tipo B(x: A);";

        let s = get_semantic(content).unwrap();

        let t_b = String::from("B");
        let t_c = String::from("C");

        let assert = s.order[0] == t_c || (s.order[0] == t_b && s.order[1] == t_c);

        assert!(assert);
        assert!(s.cycles.contains("A"));
        assert!(s.cycles.contains(&t_b));
    }

    #[test]
    fn cycle_order_3() {
        let content = "tipo C(a: A);\
        tipo A(b: B);\
        tipo B(a: A);";

        let s = get_semantic(content).unwrap();

        let t_a = String::from("A");
        let t_b = String::from("B");
        let t_c = String::from("C");

        let assert_1 = s.order == vec![t_a.clone(), t_b.clone(), t_c.clone()];
        let assert_2 = s.order == vec![t_a.clone(), t_c.clone(), t_b.clone()];
        let assert_3 = s.order == vec![t_b.clone(), t_a.clone(), t_c.clone()];

        assert!(assert_1 || assert_2 || assert_3);
        assert!(s.cycles.contains(&t_a));
        assert!(s.cycles.contains(&t_b));
    }
}
