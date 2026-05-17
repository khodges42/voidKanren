use std::fmt;
use std::rc::Rc;

use crate::runtime::error::LispResult;
use crate::runtime::env::EnvRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VarId(pub usize);

pub type BuiltinFn = fn(&[Value]) -> LispResult<Value>;

#[derive(Clone)]
pub struct Lambda {
    pub params: Vec<String>,
    pub body: Vec<Value>,
    pub env: EnvRef,
}

// We use Rc<Value> here over Box<Value> because it allows shared structure,
// i.e. multiple owners can point at the same Value.
#[derive(Clone)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Symbol(String),
    Nil,
    Pair(Rc<Value>, Rc<Value>),
    String(String),
    Builtin(BuiltinFn),
    Lambda(Rc<Lambda>),

    LogicVar(VarId),
}

impl Value {
    pub fn cons(car: Value, cdr: Value) -> Value {
        Value::Pair(Rc::new(car), Rc::new(cdr))
    }

    pub fn list(items: Vec<Value>) -> Value {
        items
            .into_iter()
            .rev()
            .fold(Value::Nil, |acc, item| Value::cons(item, acc))
    }

    pub fn is_truthy(&self) -> bool {
        // Null is truthy for now. I think thats how scheme does it.
        // In scheme pretty much everything but #f is true, #f is the only false.
        !matches!(self, Value::Bool(false))
    }
}