use std::fmt;

use crate::value::Value;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Bool(true) => write!(f, "#t"), // scheme style
            Value::Bool(false) => write!(f, "#f"), // scheme style
            Value::Symbol(s) => write!(f, "{s}"),
            Value::Nil => write!(f, "()"),
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Builtin(_) => write!(f, "#<builtin>"),
            Value::Lambda(_) => write!(f, "#<procedure>"),
            Value::Pair(car, cdr) => {
                //todo I hate doing this but idk
                write!(f, "(")?;
                // write teh contents
                write_pair_contents(f, car, cdr)?;
                write!(f, ")")
            }
            Value::LogicVar(id) => write!(f, "#<var {}>", id.0),
        }
    }
}

fn write_pair_contents(
    f: &mut fmt::Formatter<'_>,
    car: &Value,
    cdr: &Value,
) -> fmt::Result {
    write!(f, "{car}")?;

    match cdr {
        Value:: Nil => Ok(()),
        Value::Pair(next_car, next_cdr) => {
            write!(f, " ")?;
            // Todo: This probably will get crazy
            write_pair_contents(f, next_car, next_cdr)
        }

        other => write!(f, " . {other}"),
    }
}

// Want to implement display - (), 1, #t, foo, s-expr