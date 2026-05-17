use crate::runtime::error::{LispError, LispResult};
use crate::frontend::lexer::{Token, TokenKind, tokenize};
use crate::runtime::value::Value;

pub fn parse(tokens: Vec<Token>) -> LispResult<Value> {
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr()?;

    if let Some(token) = parser.peek() {
        return Err(LispError::Parse {
            message: format!("unexpected token after expression: {:?}", token.kind),
            line: token.line,
            column: token.column,
        });
    }

    Ok(expr)
}

// Parse zero or more top-level lisp expressions
// Unlike parse this does not error if there is more than one expression
// it keeps parsing until tokens run out.
pub fn parse_many(tokens: Vec<Token>) -> LispResult<Vec<Value>> {
    let mut parser = Parser::new(tokens);

    let mut exprs = Vec::new();

    while parser.peek().is_some() {
        exprs.push(parser.parse_expr()?);
    }

    Ok(exprs)
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse_expr(&mut self) -> LispResult<Value> {
        let token = self.next().ok_or_else(|| LispError::Parse {
            message: "unexpected end of input".to_string(),
            line: 0,
            column: 0,
        })?;

        match token.kind {
            TokenKind::Int(n) => Ok(Value::Int(n)),
            TokenKind::Bool(b) => Ok(Value::Bool(b)),
            TokenKind::Symbol(s) => Ok(Value::Symbol(s)),
            TokenKind::String(s) => Ok(Value::String(s)),
            TokenKind::Quote => {
                let quoted = self.parse_expr()?;
                Ok(Value::list(vec![
                    Value::Symbol("quote".to_string()),
                    quoted,
                ]))
            }

            TokenKind::LParen => self.parse_list(token.line, token.column),

            TokenKind::RParen => Err(LispError::Parse {
                message: "unexpected ')'".to_string(),
                line: token.line,
                column: token.column,
            }),

            TokenKind::Dot => Err(LispError::Parse {
                message: "unexpected '.'".to_string(),
                line: token.line,
                column: token.column,
            }),
        }
    }

    fn parse_list(&mut self, start_line: usize, start_column: usize) -> LispResult<Value> {
        let mut items = Vec::new();

        loop {
            let token = self.peek().cloned().ok_or_else(|| LispError::Parse {
                message: "unterminated list".to_string(),
                line: start_line,
                column: start_column,
            })?;

            match token.kind {
                TokenKind::RParen => {
                    self.next();
                    return Ok(Value::list(items));
                }

                TokenKind::Dot => {
                    self.next();

                    if items.is_empty() {
                        return Err(LispError::Parse {
                            message: "dot cannot appear before any list element".to_string(),
                            line: token.line,
                            column: token.column,
                        });
                    }

                    let tail = self.parse_expr()?;

                    let close = self.next().ok_or_else(|| LispError::Parse {
                        message: "expected ')' after dotted pair tail".to_string(),
                        line: start_line,
                        column: start_column,
                    })?;

                    if close.kind != TokenKind::RParen {
                        return Err(LispError::Parse {
                            message: "expected ')' after dotted pair tail".to_string(),
                            line: close.line,
                            column: close.column,
                        });
                    }

                    return Ok(list_with_tail(items, tail));
                }

                _ => {
                    let expr = self.parse_expr()?;
                    items.push(expr);
                }
            }
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }
}

fn list_with_tail(items: Vec<Value>, tail: Value) -> Value {
    items
        .into_iter()
        .rev()
        .fold(tail, |acc, item| Value::cons(item, acc))
}

#[test]
fn parse_many_top_level_expressions() {
    let exprs = parse_many(tokenize("(define x 1)\n(define y 2)\n(+ x y)").unwrap()).unwrap();

    assert_eq!(exprs.len(), 3);
    assert_eq!(exprs[0].to_string(), "(define x 1)");
    assert_eq!(exprs[1].to_string(), "(define y 2)");
    assert_eq!(exprs[2].to_string(), "(+ x y)");
}