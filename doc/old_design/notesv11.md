

# Lisp REPL with miniKanren-style thingy (v1.1)

## Core Philosophy

Build a small Lisp first, then layer relational programming on top without turning the evaluator into an eldritch junk drawer.

Main architectural boundary:

```text
Lisp layer:
  “What did the user type?”

Evaluator:
  “What does this expression mean?”

Kanren engine:
  “Given a goal and a state, what possible states follow?”
```

Important:

```text
Do not make eval_run contain the solver.
```

`eval_run` should:

* parse/build goals
* invoke the relational engine
* reify answers
* return printable values

The solver itself belongs in the Kanren layer.

---

# File Layout

```text
main.rs          // tiny entrypoint
repl.rs          // REPL loop, multiline input, intro text

lexer.rs         // source -> tokens
parser.rs        // tokens -> Value / AST

value.rs         // runtime values
printer.rs       // pretty printing / dotted pair printing

env.rs           // lexical environments / closures

eval.rs          // core evaluator/apply
forms.rs         // special forms
builtins.rs      // builtin functions

kanren.rs        // State, Goal, walk, unify, reify
kanren_forms.rs  // run, fresh, ==, conde bridge layer

error.rs         // LispError / spans / diagnostics
```

Possible alternate names:

```text
parser.rs -> reader.rs
value.rs  -> datum.rs
kanren.rs -> logic.rs / microkanren.rs
```

---

# Runtime Value Model

`Value` is recursive because Lisp lists are recursive.

Example:

```scheme
(1 2 3)
```

is internally:

```scheme
(cons 1 (cons 2 (cons 3 nil)))
```

Meaning values contain other values.

Likely runtime representation:

```rust
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

Using indirection (`Rc<Value>` or `Box<Value>`) is required because recursive enums otherwise have infinite size from Rust’s perspective.

Current leaning:

```text
Pair(Rc<Value>, Rc<Value>)
```

because shared symbolic structures are likely useful later.

---

# Logic Variables / Substitution State

miniKanren stores logical knowledge as substitutions.

Conceptually:

```text
x = 5
y = (1 . 2)
z = another variable
```

Likely representation:

```rust
HashMap<VarId, Value>
```

Where:

```rust
struct VarId(usize);
```

`usize` is Rust’s machine-sized unsigned integer type and is commonly used for IDs/indexes.

This substitution map is the core “known truths” structure of the logic engine.

---

# Goal Representation

Initial instinct:

```rust
type Goal = Fn(State) -> Vec<State>;
```

Conceptually this makes sense:

```text
A goal takes a state and produces possible next states.
```

However, Rust makes closure-heavy recursive structures painful because:

* every closure has a hidden unique type
* goals must be dynamically composable
* recursive closures/lifetimes become ugly
* storing goals in Vecs becomes difficult
* debugging/printing closures is awful

Current leaning:

```rust
enum Goal {
    Eq(Value, Value),

    Conj(Box<Goal>, Box<Goal>),
    Disj(Box<Goal>, Box<Goal>),

    Conde(Vec<Vec<Goal>>),

    // maybe later:
    Fresh(...)
}
```

Meaning:

```text
Goals are plain data (AST-like),
not executable closures.
```

Then evaluation becomes something like:

```rust
run_goal(goal: &Goal, state: State) -> Stream<State>
```

Where:

```text
Goal = declarative logic structure
run_goal = execution engine
```

This feels much more Rust-compatible and easier to debug/store/print.

---

# Streams / Search

This is core architecture, not an implementation detail.

Naive DFS search causes starvation problems:

```scheme
(disj
  loop-forever
  success)
```

would never reach `success`.

miniKanren-style search instead uses interleaving/fair search:

```text
left one step
right one step
left one step
right one step
```

instead of:

```text
left forever
```

This likely implies lazy streams/thunks rather than eager `Vec<State>` everywhere.

Current conceptual direction:

```rust
Stream<State>
```

possibly implemented later as something iterator-like:

```rust
Box<dyn Iterator<Item = State>>
```

or a custom stream structure.

Important realization:

```text
Search strategy fundamentally shapes the engine.
```

---

# Environments / Closures

Nested lexical scopes imply:

```text
child env -> parent env -> parent env
```

The tricky part is mutation via `set!`.

Current leaning:

```rust
Rc<RefCell<Env>>
```

for shared mutable environments.

This may be “less purely Rusty,” but appears normal/reasonable for interpreter implementations due to ownership constraints around nested mutable scope chains.

Need to sketch actual Env structure explicitly before implementation.

---

# Reification

Reification is more than formatting.

Responsibilities include:

* recursively walking values
* resolving substitutions
* detecting unresolved logic vars
* assigning stable readable names (`_.0`, `_.1`, ...)
* preserving ordering of first appearance
* eventually maybe handling constraints

Need concrete examples in the implementation notes.

Example:

```text
Internal:
  Var(17)

Readable:
  _.0
```

---

# conj / disj / conde

miniKanren fundamentally uses primitive combinators:

```text
conj = logical AND
disj = logical OR
```

These combine goals.

Examples:

```scheme
(conj
  (== x 5)
  (== y 10))
```

means both constraints must hold.

```scheme
(disj
  (== q 1)
  (== q 2))
```

means either branch may succeed.

Question is API design:

Should users directly see `conj/disj`, or only higher-level `conde`?

Current leaning:

```text
Internal:
  conj/disj

User-facing:
  conde
```

but possibly expose lower-level combinators later for advanced usage.

---

# Equality Semantics

Scheme distinguishes:

```text
eq?
eqv?
equal?
```

For v1:

```text
Only implement equal?
```

(structural equality)

and explicitly document this choice.

---

# Floats

No floats in v1 unless they become trivial to support naturally during arithmetic implementation.

This is currently an intentional simplification, not an oversight.

---

# Error Reporting / Spans

Want source locations early.

Instead of:

```text
Parse error
```

prefer:

```text
Parse error at line 4 column 12
```

Even lightweight span tracking is a meaningful quality improvement.

---

# Future / 2.x Weirdness

Not now, but possible future exploration:

* curses/TUI REPL
* parallel branch exploration
* constraint systems
* macros
* lazy values
* custom stream engine
* symbolic optimizations
* visualizer/debugger for relational search

Need to avoid premature wizardry.

---

# Implementation Order

## Lisp Core

1. `error.rs`
2. `value.rs`
3. `printer.rs`
4. `lexer.rs`
5. `parser.rs`
6. basic REPL
7. `env.rs`
8. `eval.rs`
9. `builtins.rs`
10. `forms.rs`
11. lambdas/closures

## miniKanren Layer

12. `LogicVar`
13. `State`
14. `walk`
15. `unify`
16. `Goal`
17. `conj/disj`
18. `reify`
19. `run`
20. `fresh`
21. `conde`
22. fair/interleaving search
23. polish/testing/debugging

---

# Main Takeaway

The architecture is intentionally trying to keep three concerns separate:

```text
Syntax representation
Expression evaluation
Relational search
```

The two biggest danger zones appear to be:

```text
Goal representation
Search strategy
```

Those decisions should be solid before deep implementation begins.



---- Claude review on 1.1

This is a genuinely good v1.1. The thinking is cleaner, the dangerous decisions are now flagged as decisions rather than glossed over, and the philosophy section earns its place. A few remaining thoughts:

---

## What got better

The Goal-as-enum section is now the strongest part of the doc. The reasoning is explicit: *why* closures hurt, *what* the alternative is, *what* that implies for the executor. That's the kind of thinking that reads as senior engineering judgment.

The Stream/Search section existing at all is a big improvement. It now correctly identifies this as architectural, not implementation detail.

---

## Still worth addressing

**The `Fresh` hole is load-bearing**

```rust
enum Goal {
    // maybe later:
    Fresh(...)
}
```

This isn't a minor TODO — `fresh` is how you introduce new logic variables, which means almost every non-trivial relational program needs it. Leaving it as `...` means you haven't resolved the hardest part of the Goal enum. The question is: what does `Fresh` hold?

The standard answer is something like:

```rust
Fresh(Arc<dyn Fn(Value) -> Goal + Send + Sync>)
```

which brings back closures in one specific controlled place. That's fine and intentional — `fresh` is the one place where a closure is semantically motivated. But it means your enum *does* contain a closure, just quarantined. Worth stating explicitly rather than leaving as `maybe later`, because it affects your `Clone`, `Debug`, and `PartialEq` derives on `Goal` immediately.

**`Rc<Value>` in the substitution map needs a note**

The doc establishes `Pair(Rc<Value>, Rc<Value>)` but then says the subst map is `HashMap<VarId, Value>`. When you walk a substitution and get back a `Value` that contains `Rc<Value>` pairs, cloning that value is cheap but the semantics get subtle — two vars might unify to the *same* `Rc` node, which is fine until someone mutates through it (they won't in pure miniKanren, but worth a line saying "values in substitutions are immutable, `Rc` is for sharing not mutation").

**The stream representation decision is still deferred**

The doc correctly identifies streams as foundational, then ends with `Box<dyn Iterator<Item = State>>` as "possibly." For a portfolio piece, I'd make the call now even if you revisit it. The two real options are:

- `Box<dyn Iterator<Item = State>>` — familiar, but interleaving requires careful implementation
- A custom `Stream` enum with a `Mature(State, Box<Stream>)` / `Immature(Box<dyn Fn() -> Stream>)` split — this is the classic miniKanren thunk model and maps directly to the literature

The second is more code but maps your implementation directly onto papers like *The Reasoned Schemer*, which is useful if you want to reference prior work in a portfolio context.

**One structural suggestion**

The "Future / 2.x Weirdness" section is fine but currently reads like a brainstorm list. For a portfolio doc, I'd rename it something like **"Explicitly Out of Scope (v1)"** and frame each item as a conscious deferral rather than a wishlist. The difference in register signals that you're managing scope deliberately, not just dreaming.

---

## Overall

You're past the point where the concern is "is this architecture sound." It is. The remaining gaps are the `Fresh` closure question and the stream type decision — both of which you need to resolve before touching `kanren.rs`, and both of which have known good answers you can just pick and document.