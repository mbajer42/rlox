use std::str::Chars;

#[derive(Debug, PartialEq)]
pub enum TokenType<'a> {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier,
    String(&'a str),
    Number(f64),

    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Debug)]
pub struct Token<'a> {
    token_type: TokenType<'a>,
    lexeme: &'a str,
    line: u32,
}

impl<'a> std::cmp::PartialEq for Token<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.token_type == other.token_type
    }
}

struct Lexer<'a> {
    source: &'a str,
    source_iter: std::iter::Peekable<std::iter::Enumerate<Chars<'a>>>,
    start: usize,
    line: u32,
    eof_returned: bool,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            source_iter: source.chars().enumerate().peekable(),
            start: 0,
            line: 1,
            eof_returned: false,
        }
    }

    fn string(&mut self, start_pos: usize) -> Option<TokenType<'a>> {
        while let Some((pos, ch)) = self.source_iter.next() {
            if ch == '"' {
                return Some(TokenType::String(&self.source[start_pos..pos]));
            }
        }
        eprintln!("Unterminatd string");
        None
    }

    fn number(&mut self, start_pos: usize) -> Option<TokenType<'a>> {
        while self.is_digit() {
            self.source_iter.next();
        }

        if self.matches('.') {
            let &(digit_pos, _) = self.source_iter.peek().unwrap();
            match self.source_iter.nth(digit_pos + 1) {
                Some((_, _ch @ '0'..='9')) => {
                    self.source_iter.next();
                    while self.is_digit() {
                        self.source_iter.next();
                    }
                }
                _ => {}
            };
        }

        let number = &self.source[start_pos..self.end_pos()];
        Some(TokenType::Number(number.parse().unwrap()))
    }

    fn identifier(&mut self, start_pos: usize) -> Option<TokenType<'a>> {
        while self.is_alpha() || self.is_digit() {
            self.source_iter.next();
        }
        let text = &self.source[start_pos..self.end_pos()];

        Some(match text {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        })
    }

    fn matches(&mut self, expected: char) -> bool {
        match self.source_iter.peek() {
            Some(&(_, ch)) => expected == ch,
            _ => false,
        }
    }

    fn is_digit(&mut self) -> bool {
        if let Some((_, ch)) = self.source_iter.peek() {
            match ch {
                '0'..='9' => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn is_alpha(&mut self) -> bool {
        if let Some((_, ch)) = self.source_iter.peek() {
            match ch {
                'a'..='z' | 'A'..='Z' | '_' => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn end_pos(&mut self) -> usize {
        if let Some(&(end_pos, _)) = self.source_iter.peek() {
            end_pos
        } else {
            self.source.len()
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((pos, ch)) = self.source_iter.next() {
            self.start = pos;
            let token_type = match ch {
                '(' => Some(TokenType::LeftParen),
                ')' => Some(TokenType::RightParen),
                '{' => Some(TokenType::LeftBrace),
                '}' => Some(TokenType::RightBrace),
                ',' => Some(TokenType::Comma),
                '.' => Some(TokenType::Dot),
                '+' => Some(TokenType::Plus),
                '-' => Some(TokenType::Minus),
                '*' => Some(TokenType::Star),
                ';' => Some(TokenType::Semicolon),
                '!' => {
                    if self.matches('=') {
                        Some(TokenType::BangEqual)
                    } else {
                        Some(TokenType::Bang)
                    }
                }
                '=' => {
                    if self.matches('=') {
                        Some(TokenType::EqualEqual)
                    } else {
                        Some(TokenType::Equal)
                    }
                }
                '<' => {
                    if self.matches('=') {
                        Some(TokenType::LessEqual)
                    } else {
                        Some(TokenType::Less)
                    }
                }
                '>' => {
                    if self.matches('=') {
                        Some(TokenType::GreaterEqual)
                    } else {
                        Some(TokenType::Greater)
                    }
                }
                '/' => {
                    if self.matches('/') {
                        while !self.matches('\n') {
                            self.source_iter.next();
                        }
                        None
                    } else {
                        Some(TokenType::Slash)
                    }
                }
                ' ' => None,
                '\r' => None,
                '\t' => None,
                '\n' => {
                    self.line += 1;
                    None
                }
                '"' => self.string(pos + 1),
                '0'..='9' => self.number(pos),
                'a'..='z' | 'A'..='Z' | '_' => self.identifier(pos),
                _ => {
                    eprintln!("Unexpected character '{}'", ch);
                    None
                }
            };
            if token_type.is_none() {
                return self.next();
            }

            Some(Token {
                token_type: token_type.unwrap(),
                lexeme: &self.source[self.start..self.end_pos()],
                line: self.line,
            })
        } else {
            if self.eof_returned {
                None
            } else {
                self.eof_returned = true;
                Some(Token {
                    token_type: TokenType::Eof,
                    lexeme: "",
                    line: self.line,
                })
            }
        }
    }
}

pub fn lex(source: &str) -> Vec<Token> {
    let lexer = Lexer::new(source);
    let tokens = lexer.collect();

    tokens
}

#[cfg(test)]
mod tests {

    use super::lex;
    use super::Token;
    use super::TokenType;

    #[test]
    fn foo() {
        let source = r#"
            var implemented = "In Rust!";
            fun answer() {
                return 42;
            }
        "#;
        let tokens = lex(source);
        let expected_tokens = vec![
            Token {
                token_type: TokenType::Var,
                lexeme: "var",
                line: 1,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: "implemented",
                line: 1,
            },
            Token {
                token_type: TokenType::Equal,
                lexeme: "=",
                line: 1,
            },
            Token {
                token_type: TokenType::String("In Rust!"),
                lexeme: r#""In Rust!""#,
                line: 1,
            },
            Token {
                token_type: TokenType::Semicolon,
                lexeme: ";",
                line: 1,
            },
            Token {
                token_type: TokenType::Fun,
                lexeme: "fun",
                line: 2,
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: "answer",
                line: 2,
            },
            Token {
                token_type: TokenType::LeftParen,
                lexeme: "(",
                line: 2,
            },
            Token {
                token_type: TokenType::RightParen,
                lexeme: ")",
                line: 2,
            },
            Token {
                token_type: TokenType::LeftBrace,
                lexeme: "{",
                line: 2,
            },
            Token {
                token_type: TokenType::Return,
                lexeme: "return",
                line: 3,
            },
            Token {
                token_type: TokenType::Number(42.0),
                lexeme: "42",
                line: 3,
            },
            Token {
                token_type: TokenType::Semicolon,
                lexeme: ";",
                line: 3,
            },
            Token {
                token_type: TokenType::RightBrace,
                lexeme: "}",
                line: 4,
            },
            Token {
                token_type: TokenType::Eof,
                lexeme: "",
                line: 5,
            },
        ];
        assert_eq!(tokens, expected_tokens);
    }
}
