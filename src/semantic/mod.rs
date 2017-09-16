use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;

use super::parser::{Ast, Parser};

pub struct SemanticBuilder {
    defs: HashMap<String, Vec<Ast>>,
    deps: HashMap<String, Vec<String>>,

    order: Vec<String>,
    visited: HashSet<String>,
    cycles: HashSet<String>,
}

pub struct Semantic {
    pub defs: HashMap<String, Vec<Ast>>,
    pub order: Vec<String>,
    pub cycles: HashSet<String>,
}

impl SemanticBuilder {
    pub fn new(ast: Parser) -> Self {
        let mut defs = HashMap::new();
        let mut deps = HashMap::new();

        for def in ast {
            if let Ast::TypeDefinition(name, parameters) = def {
                deps.insert(name.clone(), build_deps(&parameters));
                defs.insert(name, parameters);
            }
        }

        SemanticBuilder {
            defs,
            deps,
            order: Vec::new(),
            visited: HashSet::new(),
            cycles: HashSet::new(),
        }
    }

    pub fn build(mut self) -> Semantic {
        for node in self.deps.clone().keys() {
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

        self.deps.clone().get(node).map(|d|{
            for v in d {
                self.visit(v);
            }
        });

        if !self.cycles.contains(node) {
            self.order.push(node.clone());
            self.visited.remove(node);
        }
    }
}

fn build_deps(ast: &[Ast]) -> Vec<String> {
    ast.iter()
        .filter_map(|a| match *a {
            Ast::Parameter(_, ref name) => Some(name.clone()),
            _ => None,
        })
        .collect()
}
