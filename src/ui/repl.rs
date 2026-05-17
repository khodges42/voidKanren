use crate::frontend::parser;
use crate::frontend::lexer;
use crate::runtime::builtins;
use crate::runtime::env;
use crate::runtime::eval;
use crate::ui::banner::print_banner;
use crate::runtime::value::Value;
use crate::ui::theme;
use std::io::{self, Write};

pub fn run() {
    let theme = theme::load_theme();
    print_banner(&theme);
    let mut buffer = String::new();

    // Set up env
    let env = env::Env::new();
    builtins::install_builtins(env.clone());

    loop {
        if buffer.is_empty() {
            print!("{}λ>{} ", theme.prompt, theme.reset);
        } else {
            print!("{}...>{} ", theme.continuation_prompt, theme.reset);
        }
        // stdout is usually line-buffered.

        io::stdout().flush().expect("failed to flush stdout");

        let mut line = String::new();

        match io::stdin().read_line(&mut line) {

            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {
                // Add this line to the accumulated buffer.
                buffer.push_str(&line);

                // If parentheses are not balanced yet, keep reading more lines.

                if !parens_balanced(&buffer) {
                    continue;
                }
                // Once parens are balanced, tokenize and parse the whole buffer.

                match lexer::tokenize(&buffer)
                    .and_then(parser::parse)
                    .and_then(|expr| eval::eval(&expr, env.clone()))
                {
                    Ok(value) => {
                        println!("{value}");
                    }

                    Err(err) => {
                        eprintln!("{err:?}");
                    }
                }

                buffer.clear();
            }
            Err(err) => {
                eprintln!("read error: {err}");
                break;
            }
        }
    }
}

pub fn run_source(source: &str) {
    let env = env::Env::new();
    builtins::install_builtins(env.clone());

    match lexer::tokenize(source)
        .and_then(parser::parse_many)
        .and_then(|exprs| eval::eval_many(&exprs, env.clone()))
    {
        Ok(value) => {
            if !matches!(value, Value::Nil) {
                println!("{value}");
            }
        }
        Err(err) => eprintln!("{err:?}"),
    }
}

fn parens_balanced(source: &str) -> bool {
    let mut depth = 0usize;

    let mut in_comment = false;

    for ch in source.chars() {
        if in_comment {
            if ch == '\n' {
                in_comment = false;
            }

            continue;
        }

        match ch {
            ';' => {
                in_comment = true;
            }

            '(' => {
                depth += 1;
            }

            ')' => {
                // If depth is 0 there are too many closing parens.
                //
                // return true for parser to give syntax error
                if depth == 0 {
                    return true;
                }

                depth -= 1;
            }

            _ => {}
        }
    }

    depth == 0
}

fn paren_depth(source: &str) -> usize {
    let mut depth = 0usize;
    let mut in_comment = false;

    for ch in source.chars() {
        if in_comment {
            if ch == '\n' {
                in_comment = false;
            }
            continue;
        }

        match ch {
            ';' => in_comment = true,
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }

    depth
}