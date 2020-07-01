mod error;
mod lexer;
mod parser;

use std::io;
use std::io::Write;

use crate::error::LoxError;

fn run_prompt() {
    loop {
        print!("> ");
        io::stdout().flush().expect("Could write to stdout");
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(_) => {
                let (tokens, errors) = lexer::lex(&buffer);
                print_errors(&errors);
                for token in &tokens {
                    println!("{:?}", token);
                }

                let (expressions, errors) = parser::parse(&tokens);
                print_errors(&errors);
                for expression in expressions {
                    println!("{:?}", expression);
                }
            }
            Err(error) => eprintln!("error reading line: {}", error),
        }
    }
}

fn print_errors(errors: &Vec<LoxError>) {
    for error in errors {
        println!("{}", error);
    }
}

fn main() {
    run_prompt();
}
