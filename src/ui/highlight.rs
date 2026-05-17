use crate::frontend::lexer::{tokenize, TokenKind};
use crate::ui::theme::Theme;

pub fn highlight(source: &str, theme: &Theme) -> String {
    let Ok(tokens) = tokenize(source) else {
        return source.to_string();
    };

    let mut out = String::new();

    for token in tokens {
        match token.kind {
            TokenKind::LParen => {
                out.push_str(&theme.paren);
                out.push('(');
            }

            TokenKind::RParen => {
                out.push_str(&theme.paren);
                out.push(')');
            }

            TokenKind::Quote => {
                out.push_str(&theme.quote);
                out.push('\'');
            }

            TokenKind::Dot => {
                out.push_str(&theme.paren);
                out.push('.');
            }

            TokenKind::Bool(true) => {
                out.push_str(&theme.boolean);
                out.push_str("#t");
            }

            TokenKind::Bool(false) => {
                out.push_str(&theme.boolean);
                out.push_str("#f");
            }

            TokenKind::Int(n) => {
                out.push_str(&theme.number);
                out.push_str(&n.to_string());
            }

            TokenKind::Symbol(s) => {
                if is_keyword(&s) {
                    out.push_str(&theme.keyword);
                } else {
                    out.push_str(&theme.symbol);
                }

                out.push_str(&s);
            }
        }

        out.push_str(&theme.reset);
        out.push(' ');
    }

    out
}

fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "quote"
            | "if"
            | "define"
            | "lambda"
            | "let"
            | "begin"
            | "set!"
            | "and"
            | "or"
    )
}