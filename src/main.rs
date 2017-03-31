extern crate simcom;

use std::io::{self, Read};

use simcom::lexer::Lexer;
use simcom::parser::Parser;

fn main() {
    let content = {
        let mut buffer = String::new();
        let stdin = io::stdin();
        stdin.lock().read_to_string(&mut buffer).unwrap();
        
        buffer
    };

    let ast = Parser::new(Lexer::new(&content));

    for node in ast {
        println!("{:?}", node);
    }
}