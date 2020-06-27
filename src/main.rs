mod lexer;

use lexer::lex;
use std::io;
use std::io::Write;

fn run_prompt() {
    loop {
        print!("> ");
        io::stdout().flush().expect("Could write to stdout");
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(_) => {
                let tokens = lex(&buffer);
                for token in tokens {
                    println!("{:?}", token);
                }
            }
            Err(error) => eprintln!("error reading line: {}", error),
        }
    }
}

fn main() {
    run_prompt();
}
