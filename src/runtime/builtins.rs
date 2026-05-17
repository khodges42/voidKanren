use crate::env::EnvRef;
use crate::error::{LispError, LispResult};
use crate::value::Value;

pub fn install_builtins(env: EnvRef) {
    let mut env = env.borrow_mut();

    env.define("+", Value::Builtin(builtin_add));
    env.define("-", Value::Builtin(builtin_sub));
    env.define("*", Value::Builtin(builtin_mul));
    env.define("/", Value::Builtin(builtin_div));

    env.define("=", Value::Builtin(builtin_num_eq));
    env.define("<", Value::Builtin(builtin_lt));
    env.define(">", Value::Builtin(builtin_gt));
    env.define("<=", Value::Builtin(builtin_lte));
    env.define(">=", Value::Builtin(builtin_gte));

    env.define("cons", Value::Builtin(builtin_cons));
    env.define("car", Value::Builtin(builtin_car));
    env.define("cdr", Value::Builtin(builtin_cdr));
    env.define("list", Value::Builtin(builtin_list));

    env.define("display", Value::Builtin(builtin_display));
    env.define("newline", Value::Builtin(builtin_newline));

    env.define("null?", Value::Builtin(builtin_null));
    env.define("pair?", Value::Builtin(builtin_pair));
    env.define("number?", Value::Builtin(builtin_number));
    env.define("symbol?", Value::Builtin(builtin_symbol));
    env.define("boolean?", Value::Builtin(builtin_boolean));
    env.define("procedure?", Value::Builtin(builtin_procedure));
    env.define("equal?", Value::Builtin(builtin_equal));

    env.define("not", Value::Builtin(builtin_not));
    env.define("inc", Value::Builtin(builtin_inc));
    env.define("dec", Value::Builtin(builtin_dec));
}

//
// Helpers
//

fn expect_int(value: &Value, name: &str) -> LispResult<i64> {
    match value {
        Value::Int(n) => Ok(*n),
        other => Err(LispError::Type { message: format!("{name} expected integer, got {other}"),
     }),
    }
}

fn expect_arity(args: &[Value], expected: usize, name: &str) -> LispResult<()> {
    if args.len() != expected {
        return Err(LispError::Arity {
            message: format!("{name} expects at least {expected} args, got {}", args.len()),
        });
    }
    Ok(())
}

fn compare_numbers(args: &[Value], name: &str, pred: fn(i64, i64) -> bool) -> LispResult<Value> {
    if args.len() < 2 {
        return Err(LispError::Arity {
            message: format!("{name} expects at least 2 args"),
        });
    }

    // Windows lets you iterate in a slidng window.
    for pair in args.windows(2) {
        let a = expect_int(&pair[0], name)?;
        let b = expect_int(&pair[1], name)?;

        if !pred(a, b) {
            return Ok(Value::Bool(false));
        }
    }

    Ok(Value::Bool(true))
}

//
// Builtins
//
fn builtin_add(args: &[Value]) -> LispResult<Value> {
    let mut sum = 0;

    for arg in args {
        match arg {
            Value::Int(n) => sum += n,
            other => {
                return Err(LispError::Type {
                    message: format!("+ expected int, got {other}"),
                });
            }
        }
    }

    Ok(Value::Int(sum))
}

fn builtin_list(args: &[Value]) -> LispResult<Value> {
    Ok(Value::list(args.to_vec()))
}

fn builtin_display(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 1, "display")?;

    match &args[0] {
        Value::String(s) => print!("{s}"),
        other => print!("{other}"),
    }

    Ok(Value::Nil)
}

fn builtin_newline(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 0, "newline")?;
    println!();
    Ok(Value::Nil)
}

fn builtin_sub(args: &[Value]) -> LispResult<Value> {
    if args.is_empty(){
        return Err(LispError::Arity {
            message: "- (subtraction) expects at least 1 arg".to_string(),
        });
    }
    
    let first = expect_int(&args[0], "-")?;

    // We can do 'unary negation' to just negative one number, scheme does this.
    if args.len() == 1 {
        return Ok(Value::Int(-first));
    }

    let mut result = first;

    for arg in &args[1..] {
        result -= expect_int(arg, "-")?;
    }

    Ok(Value::Int(result))
}

fn builtin_mul(args: &[Value]) -> LispResult<Value> {
    let mut product = 1;
    for arg in args {
        product *= expect_int(arg, "*")?;
    }

    Ok(Value::Int(product))
}

// Integer division right now, not doing floats yet
// also check for div by zero
fn builtin_div(args: &[Value]) -> LispResult<Value> {
    if args.is_empty(){
        return Err(LispError::Arity {
            message: "/ (division) expects at least 1 arg".to_string(),
        });
    }

    let mut result = expect_int(&args[0], "/")?;

    for arg in &args[1..] {
        let n = expect_int(arg, "/")?;
        if n == 0 {
            return Err(LispError::Arity {
                message: "div by zero rip".to_string(),
            });
        }

        result /= n;

    }

    Ok(Value::Int(result))

}

fn builtin_num_eq(args: &[Value]) -> LispResult<Value> {
    compare_numbers(args, "=", |a, b| a == b )    
}

fn builtin_lt(args: &[Value]) -> LispResult<Value> {
    compare_numbers(args, "<", |a, b| a < b )    
}

fn builtin_gt(args: &[Value]) -> LispResult<Value> {
    compare_numbers(args, ">", |a, b| a > b )    
}

fn builtin_lte(args: &[Value]) -> LispResult<Value> {
    compare_numbers(args, "<=", |a, b| a <= b )    
}

fn builtin_gte(args: &[Value]) -> LispResult<Value> {
    compare_numbers(args, ">=", |a, b| a >= b )    
}

// CONStruct a list, its a lisp thing.
fn builtin_cons(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 2, "cons")?;
    Ok(Value::cons(args[0].clone(), args[1].clone()))
}

// Get the first part of a pair.
// Lisp lists are like nested (car(car(car(car(cdr)))))
// Car = Contents of Address Register (back when programmers were good and used those directly)
fn builtin_car(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 1, "car")?;

    match &args[0] {
        Value::Pair(car, _) => Ok((**car).clone()),

        other => Err(LispError::Type {
            message: format!("car expected pair, got {other}"),
        }),
    }
}

// Get the rest of the list
// Same as above
// Contents of Decrement Register
fn builtin_cdr(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 1, "cdr")?;

    match &args[0] {
        Value::Pair(_, cdr) => Ok((**cdr).clone()),

        other => Err(LispError::Type {
            message: format!("cdr expected pair, got {other}"),
        }),
    }
}

fn builtin_null(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 1, "null?")?;

    Ok(Value::Bool(matches!(args[0], Value::Nil)))
}

fn builtin_pair(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 1, "pair?")?;

    Ok(Value::Bool(matches!(args[0], Value::Pair(_, _))))
}

fn builtin_number(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 1, "number?")?;

    Ok(Value::Bool(matches!(args[0], Value::Int(_))))
}

fn builtin_symbol(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 1, "symbol?")?;

    Ok(Value::Bool(matches!(args[0], Value::Symbol(_))))
}

fn builtin_boolean(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 1, "boolean?")?;

    Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
}

// Is this callable?
// For now builtin functions and lambdas are procedures.
fn builtin_procedure(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 1, "procedure?")?;

    Ok(Value::Bool(matches!(
        args[0],
        Value::Builtin(_) | Value::Lambda(_)
    )))
}

// Logical negation. Scheme only #f is falsy.
fn builtin_not(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 1, "not")?;

    Ok(Value::Bool(!args[0].is_truthy()))
}

// basically ++ (increment)
fn builtin_inc(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 1, "inc")?;
    Ok(Value::Int(expect_int(&args[0], "inc")? + 1))
}
fn builtin_dec(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 1, "dec")?;
    Ok(Value::Int(expect_int(&args[0], "inc")? - 1))
}

// Doing this one last, theres some annoying stuff there,
fn builtin_equal(args: &[Value]) -> LispResult<Value> {
    expect_arity(args, 1, "equal?")?;

    Ok(Value::Bool(values_equal(&args[0], &args[1])))
}

// We need to recurse builtin_equal to compare nested lists
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => a == b,

        (Value::Bool(a), Value::Bool(b)) => a == b,

        (Value::Symbol(a), Value::Symbol(b)) => a == b,

        (Value::Nil, Value::Nil) => true,

        (Value::Pair(a_car, a_cdr), Value::Pair(b_car, b_cdr)) => {
            values_equal(a_car, b_car)
                && values_equal(a_cdr, b_cdr)
        }

        _ => false,
    }
}



