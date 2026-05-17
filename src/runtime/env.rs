use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::{LispError, LispResult};
use crate::value::Value;

/// Shared, mutable environment reference.
///
/// Rc = multiple things can point at the same env.
/// RefCell = env can be mutated at runtime.
///
/// This is useful for closures and nested scopes.
pub type EnvRef = Rc<RefCell<Env>>;

/// A lexical environment.
///
/// Each Env has:
/// - local bindings
/// - optional parent environment
///
/// Lookup walks:
///
/// child -> parent -> parent -> ...
pub struct Env {
    bindings: HashMap<String, Value>,
    parent: Option<EnvRef>,
}

impl Env {
    /// Create a fresh top-level environment.
    pub fn new() -> EnvRef {
        Rc::new(RefCell::new(Self {
            bindings: HashMap::new(),
            parent: None,
        }))
    }

    /// Create a child environment with a parent.
    ///
    /// Used for lambdas/lets later.
    pub fn with_parent(parent: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Self {
            bindings: HashMap::new(),
            parent: Some(parent),
        }))
    }

    /// Define a name in the CURRENT environment only.
    ///
    /// This does not search parent scopes.
    pub fn define(&mut self, name: impl Into<String>, value: Value) {
        self.bindings.insert(name.into(), value);
    }

    /// Look up a name.
    ///
    /// Search current env first, then parent, then parent's parent, etc.
    pub fn get(&self, name: &str) -> LispResult<Value> {
        if let Some(value) = self.bindings.get(name) {
            return Ok(value.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }

        Err(LispError::UnboundSymbol {
            name: name.to_string(),
        })
    }

    /// Mutate an EXISTING binding somewhere in the scope chain.
    ///
    /// This is for Scheme-style `set!`, not `define`.
    ///
    /// If the symbol exists in the current env, update it here.
    /// Otherwise try the parent.
    /// If it does not exist anywhere, error.
    pub fn set_existing(&mut self, name: &str, value: Value) -> LispResult<()> {
        if self.bindings.contains_key(name) {
            self.bindings.insert(name.to_string(), value);
            return Ok(());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().set_existing(name, value);
        }

        Err(LispError::UnboundSymbol {
            name: name.to_string(),
        })
    }
}