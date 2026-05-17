use std::rc::Rc;

use crate::runtime::env::{Env, EnvRef};
use crate::runtime::error::{LispError, LispResult};
use crate::runtime::eval;
use crate::runtime::value::{Lambda, Value};
// Special forms aren't normal functions because they evaluate all arguments first
// Important in situations like "if", because
// (if this then else)
// must not evaluate both branches
// special forms recieve raw unevaluated syntax and then decide what to evaluate
// this is a core idea in lisp
// quotes, if, define, lambda, let, begin, set!, and, or

pub fn eval_special_form(name: &str, args: &[Value], env: EnvRef) -> Option<LispResult<Value>> {
    let result = match name {
        "quote" => eval_quote(args, env),
        "if" => eval_if(args, env),
        "define" => eval_define(args, env),
        "lambda" => eval_lambda(args, env),
        "let" => eval_let(args, env),
        "begin" => eval_begin(args, env),
        "set!" => eval_set(args, env),
        "and" => eval_and(args, env),
        "or" => eval_or(args, env),
        _ => return None,
    };

    Some(result)
}

//
// Helpers
//

// Parse a lambda param list
fn parse_params(value: &Value) -> LispResult<Vec<String>> {
    let values = proper_list_to_vec(value)?;
    let mut params = Vec::new();

    for value in values {
        match value{
            Value::Symbol(name) => params.push(name), 
            other => {
                return Err(LispError::Type {
                    message: format!("lambda params must be symbol not -> {other}"),
                });
            }
        }
    }

    Ok(params)
}

// Turn a lambda param list into a vec!
pub fn proper_list_to_vec(value: &Value) -> LispResult<Vec<Value>> {
    let mut items = Vec::new();

    let mut current = value;

    loop {
        match current {
            Value::Nil => return Ok(items),

            Value::Pair(car, cdr) => {
                items.push((**car).clone());
                current = cdr;
            }

            other => {
                return Err(LispError::Type {
                    message: format!("expected proper list, got improper tail.. -> {other}"),
                });
            }
        }
    }
}

//
// Form definitions
//


// Quote prevents escalation
fn eval_quote(args: &[Value], env: EnvRef) -> LispResult<Value> {
    if args.len() != 1 {
        return Err(LispError::Arity {
            message: format!("quote expects 1 arg, got {}", args.len()),
        });
    }
    Ok(args[0].clone())
}

// IF, conditional branching.
// important that one branch evaluates, and only one.
fn eval_if(args: &[Value], env: EnvRef) -> LispResult<Value> {
    if args.len() != 3 {
        return Err(LispError::Arity {
            message: format!("if expects 3 args, got {}", args.len()),
        });
    }

    let condition = eval::eval(&args[0], env.clone())?;

    if condition.is_truthy() {
        eval::eval(&args[1], env)
    } else {
        eval::eval(&args[2], env)
    }
}

// (define x 10)
fn eval_define(args: &[Value], env: EnvRef) -> LispResult<Value> {
    if args.len() < 2 {
        return Err(LispError::Arity {
            message: format!("define expected at least 2 args, got {}", args.len()),
        });
    }

    match &args[0] {
        // Normal variable define:
        //
        //   (define x 10)
        Value::Symbol(name) => {
            if args.len() != 2 {
                return Err(LispError::Arity {
                    message: format!("variable define expected 2 args, got {}", args.len()),
                });
            }
        
            let value = eval::eval(&args[1], env.clone())?;
            env.borrow_mut().define(name.clone(), value);
            Ok(Value::Symbol(name.clone()))
        }
        // Function shorthand:
        //
        //   (define (succ n)
        //     body)
        //
        // desugars to:
        //
        //   (define succ
        //     (lambda (n)
        //       body))
        Value::Pair(_, _) => {
            let signature = proper_list_to_vec(&args[0])?;
        
            if signature.is_empty() {
                return Err(LispError::Type {
                    message: "define function signature cannot be empty".to_string(),
                });
            }
        
            let name = match &signature[0] {
                Value::Symbol(name) => name.clone(),
                other => {
                    return Err(LispError::Type {
                        message: format!("define function name must be symbol, got {other}"),
                    });
                }
            };
        
            let mut params = Vec::new();
        
            for param in &signature[1..] {
                match param {
                    Value::Symbol(name) => params.push(name.clone()),
                    other => {
                        return Err(LispError::Type {
                            message: format!("function parameter must be symbol, got {other}"),
                        });
                    }
                }
            }
        
            let lambda = Value::Lambda(Rc::new(Lambda {
                params,
                body: args[1..].to_vec(),
                env: env.clone(),
            }));
        
            env.borrow_mut().define(name.clone(), lambda);
        
            Ok(Value::Symbol(name))
        }

        other => Err(LispError::Type {
            message: format!("define expected symbol name or function signature, got {other}"),
        }),
    }
}

// Creates an anonymous function, this creates the function, 
// captures the environment it was created in
// does not run the function
fn eval_lambda(args: &[Value], env: EnvRef) -> LispResult<Value> {
    if args.len() < 2 {
        return Err(LispError::Arity {
            message: "lambda expects param and body".to_string(),
        });
    }

    // Parse param names form the first expression
    // turns them into a vec
    let params = parse_params(&args[0])?;

    // Remaining expr are the function body
    let body = args[1..].to_vec();

    // capture the current environment (for closures)
    Ok(Value::Lambda(Rc::new(Lambda {
        params,
        body,
        env,
    })))
}

// Creates temp local bindings
// results in a sub environment
fn eval_let(args: &[Value], env: EnvRef) -> LispResult<Value> {
    if args.len() < 2 {
        return Err(LispError::Arity {
            message: "let expected bindings and body".to_string(),
        });
    }

    // Create nested scope.
    let child = Env::with_parent(env.clone());

    // First arg contains binding list
    let bindings = proper_list_to_vec(&args[0])?;

    for binding in bindings {
        // Each binding should itself be:
        //
        //   (name value)
        let pair = proper_list_to_vec(&binding)?;

        if pair.len() != 2 {
            return Err(LispError::Arity {
                message: "let binding should have exactly 2 elements".to_string(),
            });
        }

        let name = match &pair[0] {
            Value::Symbol(name) => name.clone(),

            other => {
                return Err(LispError::Type {
                    message: format!("let binding name must be symbol, got {other}"),
                });
            }
        };

        // Important:
        // let initializers evaluate in OUTER env,
        // not child env.
        //
        // This matches normal Scheme let semantics.
        let value = eval::eval(&pair[1], env.clone())?;

        child.borrow_mut().define(name, value);
    }

    // Evaluate body inside child scope.
    eval_begin(&args[1..], child)
}

/// begin evaluates expressions sequentially.
// and returns the last result
// ie 'run these in order'
fn eval_begin(args: &[Value], env: EnvRef) -> LispResult<Value> {
    let mut result = Value::Nil;

    for expr in args {
        result = eval::eval(expr, env.clone())?;
    }

    Ok(result)
}

/// set! mutates an EXISTING variable.
///  Unlike define:
/// - variable MUST already exist
/// - searches upward through parent env chain
fn eval_set(args: &[Value], env: EnvRef) -> LispResult<Value> {
    if args.len() != 2 {
        return Err(LispError::Arity {
            message: format!("set! expected 2 args, got {}", args.len()),
        });
    }

    let name = match &args[0] {
        Value::Symbol(name) => name.clone(),

        other => {
            return Err(LispError::Type {
                message: format!("set! expected symbol name, got {other}"),
            });
        }
    };

    // Evaluate new value.
    let value = eval::eval(&args[1], env.clone())?;

    // Mutate existing binding.
    env.borrow_mut().set_existing(&name, value.clone())?;

    Ok(value)
}

fn eval_and(args: &[Value], env: EnvRef) -> LispResult<Value> {
    // Scheme convention:
    //
    // empty and => true
    let mut result = Value::Bool(true);

    for expr in args {
        result = eval::eval(expr, env.clone())?;

        // Stop immediately on false.
        if !result.is_truthy() {
            return Ok(Value::Bool(false));
        }
    }

    // Return last truthy value.
    Ok(result)
}

fn eval_or(args: &[Value], env: EnvRef) -> LispResult<Value> {
    for expr in args {
        let value = eval::eval(expr, env.clone())?;

        // First truthy value wins.
        if value.is_truthy() {
            return Ok(value);
        }
    }

    // Nothing truthy found.
    Ok(Value::Bool(false))
}



