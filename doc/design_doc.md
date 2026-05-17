
# Lisp REPL with miniKanren v1.3

## Core Philosophy

Build a small Lisp first, then layer relational programming on top without allowing the evaluator, parser, and solver to collapse into one large module.

The architecture enforces three boundaries:

```text
Lisp layer:     What did the user type?
Evaluator:      What does this expression mean?
Kanren engine:  Given a goal and a state, what possible states follow?
```

`eval_run` should build goals, invoke the relational engine, and reify answers. The solver itself belongs in the Kanren layer. Evaluation should not directly perform unification or search.

---

## File Layout

```text
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
kanren_forms.rs   // run, fresh, ==, conde — Lisp/Kanren bridge

error.rs          // LispError, spans, diagnostics
```

---

## Runtime Value Model

`Value` is recursive because Lisp lists are recursive. The representation needs indirection to satisfy Rust’s finite-size requirement for enums.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct VarId(usize);

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

`Rc<Value>` is chosen over `Box<Value>` to allow shared structure in symbolic data. Values in substitutions are immutable; `Rc` is for sharing, not mutation. There is no `RefCell` inside `Value`.

No floats in v1. This is intentional, not an oversight.

---

## Environment

Environments form a chain of lexical scopes:

```text
child env -> parent env -> parent env -> ...
```

`set!` must propagate mutation up the chain, which requires interior mutability:

```rust
struct Env {
    bindings: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Env>>>,
}
```

`Rc<RefCell<Env>>` is acceptable here because interpreters often need shared mutable lexical environments.

Methods:

```text
new
with_parent
define
get
set_existing
```

Logic solver state does not live in `Env`.

---

## Goal Representation

Goals are mostly plain data, not closures.

The initial intuition was:

```rust
type Goal = Fn(State) -> Vec<State>;
```

Conceptually, this means a goal takes a state and produces possible next states. That idea is correct mathematically, but painful in Rust because closures have hidden unique types, are hard to store in vectors, cannot easily be cloned or printed, and make recursive goal composition difficult.

The chosen representation is enum-based:

```rust
enum Goal {
    Eq(Value, Value),
    Conj(Box<Goal>, Box<Goal>),
    Disj(Box<Goal>, Box<Goal>),
    Conde(Vec<Vec<Goal>>),

    Fresh(Rc<dyn Fn(Value) -> Goal>),
}
```

Most goals are declarative data. `Fresh` is the controlled exception.

`fresh` is naturally function-shaped because it introduces a new logic variable and then builds a goal using that variable. For example, conceptually:

```text
fresh x:
  build goal using x
```

The choice is:

```text
Arc<dyn Fn(...) + Send + Sync>
```

versus:

```text
Rc<dyn Fn(...)>
```

The implementation chooses `Rc<dyn Fn(Value) -> Goal>` because parallel search is explicitly out of scope for v1. `Arc + Send + Sync` would support future thread-safe sharing, but it adds constraints and complexity that are unnecessary for the current design.

The executor is separate:

```rust
fn run_goal(goal: &Goal, state: State) -> Stream
```

Conceptually, this means:

```text
run_goal takes a Goal and a State,
then returns a stream of possible resulting States.
```

---

## Substitution State

miniKanren stores logical knowledge as a substitution map:

```rust
struct State {
    subst: HashMap<VarId, Value>,
    next_var: VarId,
}
```

The substitution map represents bindings from logic variables to values:

```text
VarId(0) -> Int(5)
VarId(1) -> Pair(Int(1), Int(2))
VarId(2) -> LogicVar(VarId(0))
```

`walk` follows substitution chains to find the current value of a term.

`unify` tries to make two values equal by extending the substitution. It returns failure on contradiction.

---

## Stream / Search Strategy

Search strategy is a core architectural decision.

Naive DFS causes starvation:

```scheme
(disj loop-forever (== q 1))
```

Under DFS, the right branch is never reached.

miniKanren-style fair search interleaves branches:

```text
left one step
right one step
left one step
right one step
```

The design uses a lazy stream model:

```rust
enum Stream {
    Empty,
    Mature(State, Box<Stream>),
    Immature(Box<dyn FnOnce() -> Stream>),
}
```

Meanings:

```text
Empty:
  no answers

Mature:
  one answer is ready now, with more stream afterward

Immature:
  a delayed computation that can be forced later
```

A delayed computation is a thunk: a small package of work that has not run yet.

Conceptually, `Stream` is a generator of `State` values. It may be described as `Stream<State>` in design discussion, even if the concrete enum is not generic.

A temporary `Vec<State>` runner is acceptable for early tests, but `conde` should not be considered complete until it runs on the fair stream model.

---

## conj / disj / conde

`conj` and `disj` are primitive logical combinators:

```text
conj = logical AND
disj = logical OR
```

`conde` is the user-facing form: a disjunction of conjunctions.

Design decision:

```text
Internal:
  conj / disj

User-facing:
  conde
```

Lower-level combinators may be exposed later, but not in v1.

---

## Reification

Reification is not formatting. It is the step that turns internal solver results into user-readable answers.

Given a query variable and a final substitution, reify must:

1. Walk the value fully
2. Resolve substitution chains
3. Collect remaining unbound logic variables
4. Assign stable readable names in order of first appearance
5. Return a user-readable value with no internal `VarId`s exposed

Important distinction:

```text
ground value:
  contains no unresolved variables

reified value:
  may still contain symbolic variables like _.0
```

Example:

```text
State subst:  { VarId(3) -> Pair(VarId(7), Nil), VarId(7) -> Int(42) }
Query var:    VarId(3)

Result:       Pair(Int(42), Nil)
Printed:      (42)
```

Unbound variable example:

```text
State subst:  { VarId(3) -> VarId(9) }
Query var:    VarId(3)

VarId(9) is unbound, assign name _.0
Printed:      _.0
```

Constraint reification is out of scope for v1.

---

## Equality Semantics

Scheme distinguishes:

```text
eq?     reference identity
eqv?    value identity for primitive-ish values
equal?  structural equality
```

v1 implements `equal?` only. This is deliberate and should be documented clearly.

---

## Example User Syntax

Example target behavior:

```scheme
(run 2 (q)
  (conde
    ((== q 1))
    ((== q 2))))
```

Expected result:

```scheme
(1 2)
```

Another example:

```scheme
(run 1 (q)
  (fresh (x)
    (== q (cons x x))))
```

Expected result:

```scheme
((_.0 . _.0))
```

These examples define the intended surface behavior without committing the implementation to a particular internal representation.

---

## Error Reporting

Errors should include source location from the start.

Target format:

```text
ParseError at line 4, column 12: unexpected token ')'
```

Even minimal span tracking is a meaningful quality signal. `error.rs` defines `LispError` and a `Result` alias used throughout.

---

## Correctness Invariants

The implementation should preserve these invariants:

```text
Values are immutable once constructed.

Rc<Value> is used for sharing, not mutation.

Env may be mutable through Rc<RefCell<Env>>, but solver State should be treated as persistent across search branches.

Unification never mutates existing Value structure.

walk must be applied before comparing or binding logic variables.

reify is the only place internal VarIds become user-facing symbolic names.

eval_run may construct goals and call the solver, but must not perform unification directly.

conde is not complete until it uses fair/interleaving search.

Parser and printer should round-trip proper and improper lists.
```

---

## Test Plan

### Parser / Printer

```scheme
()
(1 2 3)
(1 . 2)
(1 2 . 3)
'foo
'(1 2 3)
```

Parse and print should round-trip correctly, especially dotted pairs.

### Evaluator

```scheme
(+ 1 2)
(cons 1 2)
(car '(1 2))
(cdr '(1 2))
(if #t 1 2)
(begin (define x 5) x)
```

### Closures

```scheme
(define make-adder
  (lambda (x)
    (lambda (y) (+ x y))))

(define add5 (make-adder 5))
(add5 10)
```

Expected:

```scheme
15
```

### Unification

Test:

```text
unify x 5
unify (x . y) (1 . 2)
unify 1 2 fails
walk x through x -> y -> 5
```

### Reification

Test:

```text
unbound vars become _.0, _.1
repeated unbound var preserves identity
internal VarIds never print directly
```

### Search

Test:

```scheme
(run 2 (q)
  (conde
    ((== q 1))
    ((== q 2))))
```

Expected:

```scheme
(1 2)
```

Test fair search with an infinite branch before a successful branch once recursive relations exist.

---

## Explicitly Out of Scope v1

These are conscious deferrals:

```text
Floats / numeric tower
eq? and eqv?
Constraint systems such as CLP(FD)
Macros
Parallel search
TUI / curses REPL
Lazy language values
Symbolic optimizers
Search visualizer
```

Parallel search is specifically out of scope. The goal is correctness and fair single-threaded search first.

---

## Implementation Order

### Lisp Core

## Done
1. `error.rs` — `LispError`, spans, `Result` alias
2. `value.rs` — `Value`, `VarId`, helpers
3. `printer.rs` — `Display`, dotted list printing
4. `lexer.rs` — source to tokens
4.5. Tested the above
5. `parser.rs` — tokens to `Value`, proper/improper lists, quote syntax
6. REPL skeleton — read/print parsed values only, multiline input
7. `env.rs` — `Rc<RefCell<Env>>`, lexical scope, `set_existing`
8. `eval.rs` — self-evaluating forms, symbol lookup, application
9. `builtins.rs` — arithmetic, comparisons, list ops, type predicates
10. `forms.rs` — `quote`, `if`, `define`, `lambda`, `let`, `begin`, `set!`, `and`, `or`
11. Lambdas and closures — capture env correctly, test recursion



12. `State` — substitution map, `next_var`
13. `walk` — substitution chain following
14. `unify` — extend substitution or fail

## To do
### miniKanren Layer

14. examples and file read

14. parse_many()

15. `Goal` enum — `Eq`, `Conj`, `Disj`, `Conde`, `Fresh`
16. `run_goal` — interpret `Goal` against `State`
17. `Stream` — `Mature` / `Immature` thunk model
18. Fair interleaving search
19. `reify` — full walk, unbound var naming
20. `kanren_forms.rs` — `run`, `fresh`, `==`, `conde`
21. Integration testing — classic miniKanren examples
22. Polish, error messages, edge cases

---

## Key Risks

Two decisions shape the project:

```text
Goal representation:
  enum-based goals with Fresh as the only closure-bearing arm

Search strategy:
  fair stream-based search instead of naive DFS
```

These should be kept stable while implementing the first version. Reworking either later would likely require significant changes to the solver.


### Claude Review Comments

v1.3 is in good shape. The changes are all improvements — VarId as a newtype, Rc instead of Arc, the thunk explanation, the corrected reification language, the user syntax examples, and the test plan all make this read like a real engineering document.
A few small remaining notes, none of them blocking:

The Conde arm may be redundant

rustConde(Vec<Vec<Goal>>),

Conde is expressible as nested Disj/Conj. Having it as a dedicated arm is fine for user-facing ergonomics, but worth noting in the doc that it's a convenience form over the two primitives, not a primitive itself. Otherwise a reader might wonder why it gets special treatment.

run_goal signature omits the VarId for run
The doc shows:
rustfn run_goal(goal: &Goal, state: State) -> Stream
But run also needs a query variable to reify against. The final public-facing function is closer to:
rustfn run(n: usize, var: VarId, goal: Goal) -> Vec<Value>
Worth distinguishing run_goal (the internal engine step) from run (the user-facing entry point that calls run_goal, collects the stream, and reifies).
Test plan ordering
The unification tests currently come after the evaluator tests. Since walk and unify are implemented before eval hooks into the kanren layer, testing them in isolation first (unit tests on State directly, not through the REPL) is worth making explicit. Right now the test plan reads like integration tests throughout.

One thing that's genuinely done well

The invariants section is the strongest addition in v1.3. Specifically this one:

solver State should be treated as persistent across search branches

That's the insight that trips up most first-time miniKanren implementers — they mutate state instead of threading immutable copies through the search. Having it written down as an invariant means you'll catch it in code review rather than as a subtle bug.
The doc is ready to code from. These are polish, not blockers.