mod error;
mod interpreter;
mod lexer;
mod parser;

use std::io;
use std::io::Write;

use crate::error::LoxError;

fn run_prompt() {
    loop {
        print!("> ");
        io::stdout().flush().expect("Could not write to stdout");
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(_) => {
                let (tokens, lexer_errors) = lexer::lex(&buffer);
                print_errors(&lexer_errors);

                let (expressions, parser_errors) = parser::parse(&tokens);
                print_errors(&parser_errors);

                if !lexer_errors.is_empty() || !parser_errors.is_empty() {
                    std::process::exit(64);
                }
                interpreter::interpret(&expressions);
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
