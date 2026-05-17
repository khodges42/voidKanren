use crate::runtime::env::{Env, EnvRef};
use crate::runtime::error::{LispError, LispResult};
use crate::runtime::{forms, builtins, eval};
use crate::frontend::{parser, lexer};
use crate::runtime::value::Value;



pub fn eval(expr: &Value, env: EnvRef) -> LispResult<Value> {
    match expr {
        // Self-evaluating values.
        Value::Int(_) | Value::Bool(_) | Value::Nil | Value::Builtin(_) | Value::Lambda(_)=> Ok(expr.clone()),
        | Value::LogicVar(_) => Ok(expr.clone()),
        // Symbols evaluate by environment lookup.
        Value::Symbol(name) => env.borrow().get(name),
        Value::String(_) => Ok(expr.clone()),
        // A pair means a list/form/application.
        Value::Pair(_, _) => eval_list(expr, env),
    }
}

/// Evaluate multiple expressions in order and return the last result.
///
/// This is useful for files/scripts with many top-level expressions.
pub fn eval_many(exprs: &[Value], env: EnvRef) -> LispResult<Value> {
    let mut result = Value::Nil;

    for expr in exprs {
        result = eval(expr, env.clone())?;
    }

    Ok(result)
}

fn eval_list(expr: &Value, env: EnvRef) -> LispResult<Value> {
    let items = forms::proper_list_to_vec(expr)?;

    if items.is_empty() {
        return Ok(Value::Nil);
    }

    if let Value::Symbol(name) = &items[0] {
        if let Some(result) = forms::eval_special_form(name, &items[1..], env.clone()) {
            return result;
        }
    }

    let func = eval(&items[0], env.clone())?;

    let mut args = Vec::new();

    for arg_expr in &items[1..] {
        args.push(eval(arg_expr, env.clone())?);
    }

    apply(func, &args)
}


/// Apply a function value to already-evaluated arguments.
pub fn apply(func: Value, args: &[Value]) -> LispResult<Value> {
    match func {
        Value::Builtin(f) => f(args),

        Value::Lambda(lambda) => {
            if lambda.params.len() != args.len() {
                return Err(LispError::Arity {
                    message: format!(
                        "lambda expected {} args, got {}",
                        lambda.params.len(),
                        args.len()
                    ),
                });
            }

            let child = Env::with_parent(lambda.env.clone());

            for (param, arg) in lambda.params.iter().zip(args.iter()) {
                child.borrow_mut().define(param.clone(), arg.clone());
            }

            let mut result = Value::Nil;

            for expr in &lambda.body {
                result = eval(expr, child.clone())?;
            }

            Ok(result)
        }

        other => Err(LispError::Type {
            message: format!("attempted to call non-function: {other}"),
        }),
    }
}

#[test]
fn eval_many_top_level_expressions() {
    let env = Env::new();
    builtins::install_builtins(env.clone());

    let exprs = parser::parse_many(
        lexer::tokenize("(define x 1)\n(define y 2)\n(+ x y)").unwrap()
    ).unwrap();

    let result = eval::eval_many(&exprs, env).unwrap();

    assert_eq!(result.to_string(), "3");
}