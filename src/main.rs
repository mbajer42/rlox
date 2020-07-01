mod error;
mod lexer;
mod parser;

use std::io;
use std::io::Write;

fn run_prompt() {
    loop {
        print!("> ");
        io::stdout().flush().expect("Could write to stdout");
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(_) => {
                let tokens = lexer::lex(&buffer);
                for token in &tokens {
                    println!("{:?}", token);
                }
                let (expressions, errors) = parser::parse(&tokens);
                for error in errors {
                    println!("{}", error);
                }
                for expression in expressions {
                    println!("{:?}", expression);
                }
            }
            Err(error) => eprintln!("error reading line: {}", error),
        }
    }
}

fn main() {
    run_prompt();
}
