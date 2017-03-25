extern crate simcom;

use std::fs::File;
use std::io::Read;

use simcom::lexer::Lexer;
use simcom::parser::parser;

fn main() {
    let content = {
        let filename = std::env::args().skip(1).next().unwrap();
        let mut file = File::open(filename).unwrap();

        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        content
    };

    let tokens = Lexer::new(&content).peekable();
    let ast = parser(tokens);
    println!("{:?}", ast);
}
