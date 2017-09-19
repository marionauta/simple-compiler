use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;

use super::lexer::Token;
use super::parser::{Ast, Parser};

pub enum SemanticError {
    Unexpected(Vec<Token>),
}

pub struct Semantic {
    pub definitions: HashMap<String, Vec<(String, String)>>,
    pub order: Vec<String>,
    pub cycles: HashSet<String>,
}

impl Semantic {
    pub fn analyze(ast: Parser) -> Result<Semantic, SemanticError> {
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

        if !errors.is_empty() {
            return Err(SemanticError::Unexpected(errors));
        }

        SemanticBuilder::build(definitions)
    }
}

struct SemanticBuilder {
    definitions: HashMap<String, Vec<(String, String)>>,

    order: Vec<String>,
    visited: HashSet<String>,
    cycles: HashSet<String>,
}

impl SemanticBuilder {
    fn build(definitions: HashMap<String, Vec<(String, String)>>) -> Result<Semantic, SemanticError> {
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
