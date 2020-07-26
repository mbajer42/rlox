mod classes;
mod environment;
mod error;
mod functions;
mod interpreter;
mod lexer;
mod object;
mod parser;
mod resolver;
mod statement;
mod token;

use crate::error::LoxError;
use crate::interpreter::Interpreter;

use std::io;
use std::io::Write;

fn run_prompt() {
    let mut interpreter = Interpreter::new();
    loop {
        print!("> ");
        io::stdout().flush().expect("Could not write to stdout");
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(_) => {
                let (tokens, lexer_errors) = lexer::lex(&buffer);
                print_errors(&lexer_errors);

                let (statements, parser_errors) = parser::parse(&tokens);
                print_errors(&parser_errors);

                if !lexer_errors.is_empty() || !parser_errors.is_empty() {
                    std::process::exit(64);
                }

                let scopes = resolver::resolve(&statements);
                if scopes.is_err() {
                    std::process::exit(64);
                }
                interpreter.add_scopes(scopes.unwrap());

                interpreter
                    .interpret(statements)
                    .expect("Interpreter error: ");
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
