use crate::runtime::error::{LispResult, LispError};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    LParen,
    RParen,
    Quote,
    Dot,
    Bool(bool),
    Int(i64),
    Symbol(String),
    String(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

pub fn tokenize(source: &str) -> LispResult<Vec<Token>> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize
}

impl Lexer {
    fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn tokenize(&mut self) ->  LispResult<Vec<Token>> {
        let mut tokens = Vec::new();

        while let Some(character) = self.peek() {
            match character {
                // paranthesis,  quote, dot, semi? whitespace, else
                // todo should I make these consts? like pub const LPAREN or is that insane
                // I guess we do with that enum and we match here, so maybe not? 
                '(' => {
                    tokens.push(self.token(TokenKind::LParen));
                    self.advance();
                },
                ')' => {
                    tokens.push(self.token(TokenKind::RParen));
                    self.advance();
                },
                '\'' => {
                    tokens.push(self.token(TokenKind::Quote));
                    self.advance();
                },
                '.' => {
                    tokens.push(self.token(TokenKind::Dot));
                    self.advance();
                },
                ';' => {
                    // DO we want comments to even be semis?
                    self.skip_comment();
                },
                character if character.is_whitespace() => {
                    self.advance();
                },
                '"' => {
                    tokens.push(self.read_string()?);
                }
                _ => {
                    tokens.push(self.read_atom()?);
                }
               
            }
        }
        Ok(tokens)

    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
        // Gets the current pos character, but it's a ref so we use copied()
    }

    fn read_atom(&mut self) -> LispResult<Token> {
        // We need to remember where the atom starts
        let start_line = self.line;
        let start_column = self.column;
        // The lexer is going to consume multiple characters, so we need to track the line col for errors
        let mut text = String::new();

        while let Some(character) = self.peek() {
            // Read characters until we hit a delimiter 
            if is_delimiter(character) {
                break;
            }
            // if its a stop character, stop, otherwise push onto string
            text.push(character);
            self.advance();
        }
        // and then figure out what kind of token this atom is
        let kind = match text.as_str() {
            "#t" => TokenKind::Bool(true),
            "#f" => TokenKind::Bool(false),

            _ => {
                // Then we can check if it's an int, and hten say whatever if not its a symbol?
                // is this bad to do at any stage?
                if let Ok(n) = text.parse::<i64>() {
                    TokenKind::Int(n)
                } else {
                    TokenKind::Symbol(text)
                }
            }
        };

        Ok(Token {
            kind,
            line: start_line,
            column: start_column,
        })
    }

    fn read_string(&mut self) -> LispResult<Token> {
        let start_line = self.line;
        let start_column = self.column;
    
        // consume opening quote
        self.advance();
    
        let mut text = String::new();
    
        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    // consume closing quote
                    self.advance();
    
                    return Ok(Token {
                        kind: TokenKind::String(text),
                        line: start_line,
                        column: start_column,
                    });
                }
    
                '\\' => {
                    self.advance();
    
                    let escaped = self.peek().ok_or_else(|| LispError::Parse {
                        message: "unterminated string escape".to_string(),
                        line: start_line,
                        column: start_column,
                    })?;
    
                    match escaped {
                        'n' => text.push('\n'),
                        't' => text.push('\t'),
                        '"' => text.push('"'),
                        '\\' => text.push('\\'),
                        other => text.push(other),
                    }
    
                    self.advance();
                }
    
                other => {
                    text.push(other);
                    self.advance();
                }
            }
        }
    
        Err(LispError::Parse {
            message: "unterminated string".to_string(),
            line: start_line,
            column: start_column,
        })
    }

    fn advance(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.pos += 1;

        if character == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        Some(character)
    }
    fn token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            line: self.line,
            column: self.column,
        }
    }
    fn skip_comment(&mut self) {
        while let Some(character) = self.peek(){
            self.advance();

            if character == '\n' {
                break;
            }
        }
    }
}

fn is_delimiter(character: char) -> bool {
    character.is_whitespace()
        || character == '('
        || character == ')'
        || character == '\''
        || character == ';'
}

// I got lazy and vibed these tests

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        tokenize(src).unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn lex_empty_list() {
        assert_eq!(kinds("()"), vec![TokenKind::LParen, TokenKind::RParen]);
    }

    #[test]
    fn lex_numbers_symbols_and_bools() {
        assert_eq!(
            kinds("(define x #t 123 -45)"),
            vec![
                TokenKind::LParen,
                TokenKind::Symbol("define".into()),
                TokenKind::Symbol("x".into()),
                TokenKind::Bool(true),
                TokenKind::Int(123),
                TokenKind::Int(-45),
                TokenKind::RParen,
            ]
        );
    }

    #[test]
    fn lex_quote() {
        assert_eq!(
            kinds("'foo"),
            vec![
                TokenKind::Quote,
                TokenKind::Symbol("foo".into()),
            ]
        );
    }

    #[test]
    fn lex_dotted_pair() {
        assert_eq!(
            kinds("(1 . 2)"),
            vec![
                TokenKind::LParen,
                TokenKind::Int(1),
                TokenKind::Dot,
                TokenKind::Int(2),
                TokenKind::RParen,
            ]
        );
    }

    #[test]
    fn skips_comments() {
        assert_eq!(
            kinds("(1 ; ignore me\n 2)"),
            vec![
                TokenKind::LParen,
                TokenKind::Int(1),
                TokenKind::Int(2),
                TokenKind::RParen,
            ]
        );
    }

    #[test]
    fn tracks_line_and_column() {
        let tokens = tokenize("(\n  42)").unwrap();

        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);

        assert_eq!(tokens[1].kind, TokenKind::Int(42));
        assert_eq!(tokens[1].line, 2);
        assert_eq!(tokens[1].column, 3);
    }
}