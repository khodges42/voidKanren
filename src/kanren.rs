use std::collections::HashMap;

use crate::runtime::value::{Value, VarId};

/// A miniKanren State.
///
/// This represents one possible logical world.
///
/// subst:
///   current known bindings for logic variables
///
/// next_var:
///   the next fresh logic variable ID to allocate
#[derive(Clone)]
pub struct State {
    pub subst: HashMap<VarId, Value>,
    pub next_var: usize,
}

impl State {
    /// Create an empty logical state.
    ///
    /// No variables are bound yet.
    /// The next fresh variable will be VarId(0).
    pub fn empty() -> Self {
        Self {
            subst: HashMap::new(),
            next_var: 0,
        }
    }

    /// Create a fresh logic variable and return:
    ///
    ///   1. the new logic variable as a Value
    ///   2. the updated State
    ///
    /// Important:
    /// We return a NEW state instead of mutating the old one.
    /// This keeps search branches independent later.
    pub fn fresh_var(&self) -> (Value, State) {
        let id = VarId(self.next_var);

        let mut next_state = self.clone();
        next_state.next_var += 1;

        (Value::LogicVar(id), next_state)
    }
}

/// Follow substitution chains.
///
/// Example:
///
///   x -> y
///   y -> 5
///
/// walk(x) returns 5.
///
/// If the value is not a bound logic variable, it returns the value unchanged.
pub fn walk(value: &Value, subst: &HashMap<VarId, Value>) -> Value {
    match value {
        Value::LogicVar(var) => {
            if let Some(bound_value) = subst.get(var) {
                walk(bound_value, subst)
            } else {
                value.clone()
            }
        }

        _ => value.clone(),
    }
}

/// Try to make two values equal under the current State.
///
/// If successful:
///   returns a NEW State with additional substitutions.
///
/// If impossible:
///   returns None.
///
/// Examples:
///
///   unify(x, 5)
///     => binds x to 5
///
///   unify((x . y), (1 . 2))
///     => binds x to 1 and y to 2
///
///   unify(1, 2)
///     => fails
pub fn unify(a: &Value, b: &Value, state: &State) -> Option<State> {
    let a = walk(a, &state.subst);
    let b = walk(b, &state.subst);

    match (&a, &b) {
        // Same integers unify.
        (Value::Int(x), Value::Int(y)) if x == y => Some(state.clone()),

        // Same booleans unify.
        (Value::Bool(x), Value::Bool(y)) if x == y => Some(state.clone()),

        // Same symbols unify.
        (Value::Symbol(x), Value::Symbol(y)) if x == y => Some(state.clone()),

        // Nil unifies with Nil.
        (Value::Nil, Value::Nil) => Some(state.clone()),

        // If left side is an unbound logic var, bind it.
        (Value::LogicVar(var), _) => bind_var(*var, &b, state),

        // If right side is an unbound logic var, bind it.
        (_, Value::LogicVar(var)) => bind_var(*var, &a, state),

        // Pairs unify recursively:
        //
        //   car with car
        //   cdr with cdr
        (Value::Pair(a_car, a_cdr), Value::Pair(b_car, b_cdr)) => {
            let state_after_car = unify(a_car, b_car, state)?;
            unify(a_cdr, b_cdr, &state_after_car)
        }

        // Everything else fails.
        _ => None,
    }
}

/// Bind a logic variable to a value by extending the substitution map.
///
/// This clones the state and returns an updated copy.
///
/// Later you may add an occurs-check here to prevent:
///
///   x = (1 . x)
///
/// For classic miniKanren, occurs-check is often omitted.
fn bind_var(var: VarId, value: &Value, state: &State) -> Option<State> {
    let mut next_state = state.clone();

    next_state.subst.insert(var, value.clone());

    Some(next_state)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(n: i64) -> Value {
        Value::Int(n)
    }

    fn var(n: usize) -> Value {
        Value::LogicVar(VarId(n))
    }

    #[test]
    fn fresh_var_allocates_unique_variables() {
        let state = State::empty();

        let (v0, state) = state.fresh_var();
        let (v1, state) = state.fresh_var();

        assert_eq!(v0.to_string(), "#<var 0>");
        assert_eq!(v1.to_string(), "#<var 1>");
        assert_eq!(state.next_var, 2);
    }

    #[test]
    fn walk_unbound_var_returns_var() {
        let state = State::empty();

        let result = walk(&var(0), &state.subst);

        assert_eq!(result.to_string(), "#<var 0>");
    }

    #[test]
    fn walk_bound_var_returns_value() {
        let mut state = State::empty();
        state.subst.insert(VarId(0), int(5));

        let result = walk(&var(0), &state.subst);

        assert_eq!(result.to_string(), "5");
    }

    #[test]
    fn walk_follows_chain() {
        let mut state = State::empty();

        state.subst.insert(VarId(0), var(1));
        state.subst.insert(VarId(1), int(5));

        let result = walk(&var(0), &state.subst);

        assert_eq!(result.to_string(), "5");
    }

    #[test]
    fn unify_var_with_int() {
        let state = State::empty();

        let result = unify(&var(0), &int(5), &state).unwrap();

        assert_eq!(walk(&var(0), &result.subst).to_string(), "5");
    }

    #[test]
    fn unify_same_int_succeeds() {
        let state = State::empty();

        assert!(unify(&int(5), &int(5), &state).is_some());
    }

    #[test]
    fn unify_different_int_fails() {
        let state = State::empty();

        assert!(unify(&int(5), &int(6), &state).is_none());
    }

    #[test]
    fn unify_pair_structure() {
        let state = State::empty();

        let left = Value::cons(var(0), var(1));
        let right = Value::cons(int(1), int(2));

        let result = unify(&left, &right, &state).unwrap();

        assert_eq!(walk(&var(0), &result.subst).to_string(), "1");
        assert_eq!(walk(&var(1), &result.subst).to_string(), "2");
    }

    #[test]
    fn unify_nested_pair_structure() {
        let state = State::empty();

        let left = Value::list(vec![var(0), var(1), int(3)]);
        let right = Value::list(vec![int(1), int(2), var(2)]);

        let result = unify(&left, &right, &state).unwrap();

        assert_eq!(walk(&var(0), &result.subst).to_string(), "1");
        assert_eq!(walk(&var(1), &result.subst).to_string(), "2");
        assert_eq!(walk(&var(2), &result.subst).to_string(), "3");
    }

    #[test]
    fn unify_conflicting_structure_fails() {
        let state = State::empty();

        let left = Value::list(vec![int(1), int(2)]);
        let right = Value::list(vec![int(1), int(999)]);

        assert!(unify(&left, &right, &state).is_none());
    }
}