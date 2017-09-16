use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;

use super::parser::{Ast, Parser};

pub struct SemanticBuilder {
    defs: HashMap<String, Vec<(String, String)>>,

    order: Vec<String>,
    visited: HashSet<String>,
    cycles: HashSet<String>,
}

pub struct Semantic {
    pub defs: HashMap<String, Vec<(String, String)>>,
    pub order: Vec<String>,
    pub cycles: HashSet<String>,
}

impl SemanticBuilder {
    pub fn new(ast: Parser) -> Self {
        let mut defs = HashMap::new();

        for def in ast {
            if let Ast::TypeDefinition(name, parameters) = def {
                defs.insert(name, build_defs(parameters));
            }
        }

        SemanticBuilder {
            defs,
            order: Vec::new(),
            visited: HashSet::new(),
            cycles: HashSet::new(),
        }
    }

    pub fn build(mut self) -> Semantic {
        for node in self.defs.clone().keys() {
            self.visit(node);
        }

        Semantic {
            defs: self.defs,
            order: self.order,
            cycles: self.cycles,
        }
    }

    pub fn visit(&mut self, node: &String) {
        if self.order.contains(node) {
            return;

        } else if self.visited.contains(node) {
            self.cycles = self.visited.clone();
            return;
        }

        self.visited.insert(node.clone());

        if let Some(d) = self.defs.clone().get(node) {
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

fn ast_to_tuple(ast: Ast) -> Option<(String, String)> {
    match ast {
        Ast::Parameter(name, typename) => Some((name, typename)),
        _ => None,
    }
}

fn build_defs(ast: Vec<Ast>) -> Vec<(String, String)> {
    ast.into_iter()
        .filter_map(ast_to_tuple)
        .collect()
}