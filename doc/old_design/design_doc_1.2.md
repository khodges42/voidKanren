# Lisp REPL with miniKanren (v1.2)

## Core Philosophy

Build a small Lisp first, then layer relational programming on top without
turning the evaluator into an eldritch junk drawer.

The architecture enforces three hard boundaries:

```
Lisp layer:    "What did the user type?"
Evaluator:     "What does this expression mean?"
Kanren engine: "Given a goal and a state, what possible states follow?"
```

`eval_run` should build goals, invoke the relational engine, and reify answers.
The solver itself belongs entirely in the Kanren layer. These concerns do not bleed
into each other.

---

## File Layout

```
main.rs           // tiny entrypoint
repl.rs           // REPL loop, multiline input, intro text

lexer.rs          // source -> tokens
parser.rs         // tokens -> Value / AST

value.rs          // runtime values
printer.rs        // pretty printing, dotted pair handling

env.rs            // lexical environments and closures

eval.rs           // core evaluator / apply
forms.rs          // special forms
builtins.rs       // builtin eager functions

kanren.rs         // State, Goal, walk, unify, reify, run_goal
kanren_forms.rs   // run, fresh, ==, conde — the Lisp/Kanren bridge

error.rs          // LispError, spans, diagnostics
```

---

## Runtime Value Model

`Value` is recursive because Lisp lists are recursive. The representation needs
indirection to satisfy Rust's finite-size requirement for enums.

```rust
type VarId = usize;

enum Value {
    Int(i64),
    Bool(bool),
    Symbol(String),
    Nil,
    Pair(Rc<Value>, Rc<Value>),
    Lambda(Lambda),
    Builtin(BuiltinFn),
    LogicVar(VarId),
}
```

`Rc<Value>` is chosen over `Box<Value>` to allow shared structure in symbolic
data — two logic variables can unify to the same node without copying. Values in
substitutions are treated as immutable; `Rc` is for sharing, not mutation. There
is no `RefCell` inside `Value`.

No floats in v1. This is an intentional simplification, not an oversight.

---

## Environment

Environments form a chain of lexical scopes:

```
child env -> parent env -> parent env -> ...
```

`set!` must propagate mutation up the chain, which requires interior mutability:

```rust
struct Env {
    bindings: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Env>>>,
}
```

`Rc<RefCell<Env>>` is normal for interpreter implementations. The ownership
constraints around nested mutable scope chains make it a natural fit. Methods:
`new`, `with_parent`, `define`, `get`, `set_existing`.

Logic solver state does not live in `Env`.

---

## Goal Representation

Goals are plain data, not closures.

The initial intuition of `type Goal = Fn(State) -> Vec<State>` is
conceptually appealing but breaks down in Rust: closures have hidden unique
types, can't be stored in `Vec`s without boxing, can't be cloned or printed,
and recursive goal composition produces lifetime nightmares.

The correct representation is an enum:

```rust
enum Goal {
    Eq(Value, Value),
    Conj(Box<Goal>, Box<Goal>),
    Disj(Box<Goal>, Box<Goal>),
    Conde(Vec<Vec<Goal>>),
    Fresh(Arc<dyn Fn(Value) -> Goal + Send + Sync>),
}
```

`Fresh` is the one controlled exception. Introducing a new logic variable
is semantically a function from a fresh var to a goal, so a closure is
motivated here. It is quarantined in one enum arm rather than spread throughout
the system. Note that `Fresh` containing a closure means `Goal` cannot
automatically derive `Clone`, `Debug`, or `PartialEq` — those need manual
implementations or explicit omission.

The executor is a separate function:

```rust
fn run_goal(goal: &Goal, state: State) -> Stream;
```

Goals are declarative structure. `run_goal` is the engine that interprets them.

---

## Substitution State

miniKanren stores logical knowledge as a substitution map — bindings from
logic variables to values:

```rust
struct State {
    subst: HashMap<VarId, Value>,
    next_var: VarId,
}
```

`walk` follows substitution chains to find the ground value (or unbound var)
for any term. `unify` extends the substitution by binding logic vars, returning
`None` on contradiction.

---

## Stream / Search Strategy

This is a core architectural decision, not an implementation detail.

Naive DFS search causes starvation:

```scheme
(disj loop-forever (== q 1))
```

Under DFS, the right branch is never reached. miniKanren-style fair search
interleaves branches — one step left, one step right — so all branches get
explored.

The standard model uses a lazy stream with two cases:

```rust
enum Stream {
    Empty,
    Mature(State, Box<Stream>),
    Immature(Box<dyn FnOnce() -> Stream>),
}
```

`Mature` is a ready answer. `Immature` is a suspended thunk that produces more
stream when forced. Interleaving is implemented by forcing one step at a time
from each branch rather than exhausting one before starting the other.

This maps directly to the presentation in *The Reasoned Schemer*, which is
useful as a reference if explaining the design to others.

`Box<dyn Iterator<Item = State>>` is an alternative and simpler to implement,
but achieving correct interleaving with it requires more care. The thunk model
makes the interleaving structure explicit.

For v1, a `Vec<State>` can be used as a temporary scaffold to get the rest of
the system working, but the stream model should be in place before `conde`
is implemented.

---

## conj / disj / conde

`conj` and `disj` are the primitive logical combinators:

```
conj = logical AND (both goals must hold)
disj = logical OR  (either branch may succeed)
```

`conde` is the user-facing form: disjunction of conjunctions. In standard
miniKanren, `conde` is the primitive users reach for; `conj` and `disj` are
the implementation layer that `run_goal` uses internally.

Design decision: `conj` and `disj` are internal. Users see `conde`. Lower-level
combinators may be exposed later for advanced usage, but not in v1.

---

## Reification

Reification is not formatting — it is a semantically meaningful step.

Given a query variable and a final substitution, reify must:

1. Walk the value fully, resolving all substitution chains
2. Collect all remaining unbound logic variables
3. Assign stable, human-readable names in order of first appearance (`_.0`, `_.1`, ...)
4. Return a fully ground `Value` with no internal `VarId`s visible

Concrete example:

```
State subst:  { Var(3) -> Pair(Var(7), Nil), Var(7) -> Int(42) }
Query var:    Var(3)

Walk Var(3)  -> Pair(Var(7), Nil)
Walk Var(7)  -> Int(42)
Result:       Pair(Int(42), Nil)
Printed:      (42)
```

Unbound variable example:

```
State subst:  { Var(3) -> Var(9) }
Query var:    Var(3)

Walk Var(3)  -> Var(9)
Var(9) is unbound, assign name _.0
Result:       _.0
```

Constraint reification (for later versions) also lives here.

---

## Equality Semantics

Scheme distinguishes `eq?` (reference identity), `eqv?` (value identity for
primitives), and `equal?` (structural equality).

v1 implements `equal?` only. This is a deliberate simplification. It will be
documented clearly so users know what they're getting. `eq?` and `eqv?` are
out of scope until there is a reason to add them.

---

## Error Reporting

Errors should include source location from the start. Retrofitting span
tracking later is painful.

Target format:

```
ParseError at line 4, column 12: unexpected token ')'
```

Even minimal span tracking (line and column on tokens) is a meaningful quality
signal for a portfolio piece. `error.rs` defines `LispError` and a `Result`
alias used throughout.

---

## Explicitly Out of Scope (v1)

These are conscious deferrals, not forgotten features:

- **Floats** — no numeric tower in v1
- **`eq?` / `eqv?`** — `equal?` only
- **Constraint systems** — CLP(FD) etc. are post-v1
- **Macros** — not in the core
- **Parallel search** — threading miniKanren search naively creates serious
  problems around fairness and synchronization; marked as 2.x research territory
- **TUI / curses REPL** — would be excellent, will be goblin-bait; 2.0
- **Lazy values, symbolic optimizers, search visualizer** — later

---

## Implementation Order

### Lisp Core

1. `error.rs` — `LispError`, spans, `Result` alias
2. `value.rs` — `Value` enum, helpers, `VarId`
3. `printer.rs` — `Display`, dotted list printing, round-trip correctness
4. `lexer.rs` — source to tokens
5. `parser.rs` — tokens to `Value`, proper/improper lists, quote syntax
6. REPL skeleton — read/print parsed values only, multiline input
7. `env.rs` — `Rc<RefCell<Env>>`, lexical scope, `set_existing`
8. `eval.rs` — self-evaluating forms, symbol lookup, application
9. `builtins.rs` — arithmetic, comparisons, list ops, type predicates
10. `forms.rs` — `quote`, `if`, `define`, `lambda`, `let`, `begin`, `set!`, `and`, `or`
11. Lambdas and closures — capture env correctly, test recursion

### miniKanren Layer

12. `State` — substitution map, `next_var`
13. `walk` — substitution chain following
14. `unify` — extend substitution or fail
15. `Goal` enum — `Eq`, `Conj`, `Disj`, `Conde`, `Fresh`
16. `run_goal` — interpret `Goal` against `State`, produce `Stream`
17. `Stream` — `Mature` / `Immature` thunk model, interleaving
18. `reify` — full walk, unbound var naming, ground value output
19. `kanren_forms.rs` — `run`, `fresh`, `==`, `conde` as Lisp special forms
20. Integration testing — classic miniKanren examples working end-to-end
21. Polish, error messages, edge cases

---

## Key Risks

Two decisions shape everything else. Get these right on paper before writing
`kanren.rs`:

**Goal representation** — the enum model with `Fresh` as the one closure-bearing
arm is the right call. Deviating toward a pure-closure model causes Rust pain
that compounds throughout the system.

**Search strategy** — the `Mature` / `Immature` thunk stream gives correct
interleaving and maps cleanly to the literature. This needs to be in place
before `conde` is implemented; retrofitting fair search onto DFS later is
a significant rewrite.