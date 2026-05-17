## Lisp REPL with miniKanren-style thingy

Goal:
Build a small Lisp first, then bolt on miniKanren-style relational programming without turning eval.rs into the trash dimension.

Main boundary:
- Lisp syntax layer: what did the user type?
- Evaluator: what does this expression mean?
- Kanren engine: how do goals produce possible states?

Do not make eval_run contain the solver. eval_run should build goals, call the solver, then reify answers.

Files:
- main.rs: tiny entrypoint
- repl.rs: REPL loop, multiline input, intro text
- lexer.rs: source -> tokens
- parser.rs / reader.rs: tokens -> Value/AST
- value.rs: runtime values
- printer.rs: Display/pretty printing
- env.rs: lexical env and closures
- eval.rs: core eval/apply
- forms.rs: special forms
- builtins.rs: normal eager functions
- kanren.rs or logic/: State, Goal, walk, unify, reify
- kanren_forms.rs: run, fresh, ==, conde, conj/disj syntax bridge
- error.rs: LispError and Result alias

Main / REPL:
- Set up env
- Install builtins
- Print cool intro
- Big loop:
  - read input
  - handle multiline via balanced parens helper
  - parse
  - eval
  - print
- Curses UI would be sick, but that is 2.0 goblin bait

Lexer:
- Token enum:
  - LParen
  - RParen
  - Quote
  - Dot
  - Bool
  - Int
  - Symbol
- Tokenize source
- Delimiter helper is fine here
- Do not strip meaningful syntax

Parser / reader:
- Parser has tokens and pos
- pos is the cursor/index into the token stream
- Helpers:
  - peek
  - next
  - parse_expr
  - parse_list
- Parse:
  - ints
  - bools
  - symbols
  - nil / empty list
  - proper lists
  - dotted pairs / improper lists
  - quote syntax
- Dot means improper pair/list:
  - (1 . 2) means cons 1 2
  - (1 2 . 3) means cons 1 (cons 2 3)

Value:
- Runtime values:
  - Int
  - Bool
  - Symbol
  - Nil
  - Pair
  - Lambda
  - Builtin
  - LogicVar
- Lambda struct:
  - params
  - body
  - closure env
- Conversions/helpers:
  - list construction
  - list iteration
  - expect_int
  - expect_pair
  - truthiness
- Debug is for me
- Display is for the language user
- write_list handles proper and dotted list printing
- Be careful with PartialEq. Rust equality and Lisp equal? may not be exactly the same thing.

Environment:
- Env has bindings and optional parent
- Used for lexical scope and closures
- Methods:
  - new
  - with_parent
  - define
  - get
  - set_existing
- Do not put relational solver state in Env

Evaluator:
- Keep eval.rs boring
- eval:
  - self-evaluating values return themselves
  - symbols lookup in env
  - lists dispatch to special forms or function application
- apply:
  - Builtin
  - Lambda
- eval_list:
  - recognize special forms
  - otherwise evaluate function and args
- Do not put all builtins here
- Do not put the solver here

Special forms:
- quote
- if
- define
- lambda
- begin
- let
- set!
- and/or
- maybe cond later
- run/fresh/conde are special too, but maybe live in kanren_forms.rs

Builtins:
- install_builtins(env)
- Arithmetic:
  - +
  - -
  - *
  - /
  - mod
- Comparisons:
  - =
  - <
  - >
  - <=
  - >=
- Pairs/lists:
  - cons
  - car
  - cdr
  - list
  - null?
  - pair?
  - append
- Type checks:
  - number?
  - symbol?
  - boolean?
  - procedure?
- Equality:
  - equal?
- Maybe map later

Kanren:
- State:
  - subst
  - next_var
  - maybe constraints later
- fresh_var creates a new LogicVar
- walk follows substitutions
- unify makes two values equal, possibly by binding logic vars
- reify turns internal symbolic vars into readable names
- Goal is a function from State -> possible States
- eq goal uses unify
- conj means AND
- disj means OR
- conde is disjunction of conjunctions



---

### Claude reviewed my design

Real concerns
1. Goal as a function type will hurt you
Defining Goal as Fn(State) -> impl Iterator<Item = State> (or similar) sounds elegant but creates serious Rust pain: you can't store Goals in a Vec, can't clone them without Arc, and conde becomes a lifetime nightmare. The standard miniKanren Rust approach is to make Goal an enum or a trait object:
```rust
enum Goal {
    Eq(Value, Value),
    Fresh(Arc<dyn Fn(Value) -> Goal>),
    Conj(Box<Goal>, Box<Goal>),
    Disj(Box<Goal>, Box<Goal>),
    Conde(Vec<Vec<Goal>>),
}
```
This is more verbose but you can actually print it, debug it, and store it. This is the decision I'd think hardest about before writing a line of kanren code.
2. The substitution map key needs a decision
subst maps logic vars to values — but what's the key? If LogicVar is a usize (just an ID), the map is HashMap<usize, Value>. That's fine. But your Value enum will need Clone everywhere, and Pair is recursive, so you'll almost certainly need Rc<RefCell<...>> or just Box for the pair arms. The doc doesn't mention this and it will shape everything in value.rs. Decide it explicitly before you start.
3. set_existing on Env suggests a mutable-env design — be careful
set! in Scheme mutates an existing binding. If your Env is Rc<RefCell<HashMap>> with parent chaining, that works fine. But if you went with a simpler owned-map design, set_existing that propagates up to parent scopes is surprisingly tricky to implement correctly in Rust without unsafe or a lot of cloning. Worth sketching the Env struct fields explicitly in the doc.
4. reify scope is underspecified
The doc says "turns internal symbolic vars into readable names" but this hides real complexity. Reification needs to: walk the value fully, collect all unbound vars, assign them human-readable names (_0, _1, ...) in order of first appearance, and substitute. If you also want to reify constraints later (which you said "maybe"), the reifier is where that logic lives and it's nontrivial. I'd write a concrete example in the doc: given this substitution map and this query variable, what does reify return?
5. conde semantics vs. disj — draw the line clearly
Right now the doc lists both conde (disjunction of conjunctions) and disj (OR). In standard miniKanren, conde is the primitive and disj/conj are implementation-level combinators, not user-facing. Exposing both to the user creates confusion about which to use. Decide: is disj internal-only, or are you deliberately exposing a lower-level API too? Either is fine, but be explicit.
6. Missing: stream/search strategy
miniKanren's power comes from interleaving search (the "mature stream" model) rather than DFS. The doc doesn't mention this at all. If you implement disj as "try left branch fully, then right branch," you get DFS and (run* q (disj (== q 1) loop-forever)) will hang. The interleaving/inc thunk pattern is what makes miniKanren miniKanren. This deserves its own section.

Smaller notes

Float is conspicuously absent from Value. Fine if intentional — just make sure it's a decision, not an oversight.
write_list handling dotted pairs: make sure your printer round-trips correctly. (1 2 . 3) should parse and re-print identically. Easy to get subtly wrong.
equal? vs. eq? vs. eqv? — Scheme distinguishes these. If you're only implementing equal? (structural), note that explicitly so you don't accidentally make eq? structural too.
Error messages: error.rs is listed but there's no mention of source location / span tracking. For a portfolio project, even line numbers in errors is a meaningful quality signal.


Summary verdict
The architecture is genuinely good — better than most toy Lisp designs I see. The two things that could quietly derail you are the Goal representation (make it an enum, not a closure) and the search strategy (interleaving vs DFS). Get those two right on paper before touching kanren.rs and the rest will flow naturally.


### Thoughts on the review

* Goal as a function type will hurt me.
I think I somewhat follow? So conceptually I have a type Goal as fn(State) -> Vec<State>, a function that takes in state and returns a Vec<State>.This makes sense to me, but its a rust issue

The issue, from what I understand is that every closure in rust has a hidden type, so every one of these will have a different type, so I cant just do like Vec<Goal> because rust wants my function type.

This matters ecause miniKanren constantly builds trees of goals.I want to store and combine them dynamically, and closures became an issue.

So the suggested solution is to make goals lain data like an ast. So goals are just trees. Then I can do something like run_goal(goal: &Goal, state: State) -> Stream<State> (a stream being like a iter in python, itll be like something like Box<dyn Iterator<Item = State>>)

* Substitution map key needs a decision
miniKanren stores facts like x = y ... y = (1 . 2) internally HashMap<LogicVar, Value>

Logicvar I was thinking just struct varId(usize)

this matters because value is recursive, rust will not understand the size. So I need indirection, like Pair(Box<Value>, Box<Value>) or Pair(Rc<Value>, Rc<Value>)

* Mutable env design, be careful
Rust ownership pain.

I want nested scopes, which means child env -> parent env -> parent env

the problen is set!, searching upward through parent scopes and mutating existing binding. In rust, mutating shared parent structures is annoying unless you use Rc<RefCall<Env>> which is maybe not rusty? Maybe its ok here.

* Reify is underspecified
Walking is difficult, walking recursively through nested lists/pairs, detecting unboundv ars, assigning stable readable names... It's not just a format. I will be careful to take care for this to avoid problems.

* Conde semantics vs disj
This is api design.
MiniKanren fundamentally uses conj = AND, disj = OR these are primative combinators
Reviewer is asking if I want user to see disj or just cond? I think I want to expose both eventually but user facing conde and internal conj, disj.

* Missing search strategy
I recognize naive dfs doesn't work in some cases

Interleaving search is the solve here, like instead of left left left left... forever, it does left one step, right one step, left one step, and so on. So infiite branches don't starve others.

mad science question, can we do threading here? (and not create a world of pain?) - not now but 2.0?


* Streams
    Yeah I want to handle this earlier, like a generator. I recognize search strategy isn't an implementation detail, but core to this.


* Floats
    No floats in v1... though is it that heavy to add?

* equal? vs eq?
Maybe just equal? possibly dumb question, what is the difference in scheme?

* Span tracking
Yeah definitely this, big win for quality improvement.

---
### Random notes

substitution map is the map for minikanren to understand what variables mean right now.


So the ENTIRE miniKanren engine is basically:
Goal:
  transform states

State:
  substitution map + fresh variable counter

Substitution map:
  current known truths about logic variables

#### Your Lisp interpreter manipulates:

Values

#### Your miniKanren engine manipulates:

Knowledge ABOUT values

That distinction is the philosophical split.

Lisp:
(+ 1 2)

Compute value.

miniKanren:
(== q (+ 1 2))

Constrain possible worlds.