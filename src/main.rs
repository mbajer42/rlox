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

use std::fs::File;
use std::io;
use std::io::prelude::*;
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

fn run_file(filename: &str) {
    let mut file = File::open(filename).expect("Could not read file: ");
    let mut code = String::new();
    file.read_to_string(&mut code)
        .expect("Could not read file: ");

    let mut interpreter = Interpreter::new();
    let (tokens, lexer_errors) = lexer::lex(&code);
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

fn print_errors(errors: &Vec<LoxError>) {
    for error in errors {
        eprintln!("{}", error);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        run_prompt();
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        eprintln!("Unexpected number of arguments. Expected none (interactive) or one(file).");
    }
}
