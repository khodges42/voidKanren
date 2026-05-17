# voidKanren

voidKanren is a small Lisp interpreter in Rust that is being built as a base for miniKanren-style relational programming experiments.

The project is intentionally split into layers: a Lisp reader/parser, an evaluator, and a relational engine. The long-term goal is to keep those boundaries clean enough that new relational Lisp ideas can be tried without turning the evaluator and solver into one tangled subsystem.

This is a core project for further relational Lisp experimentation, not a polished Scheme implementation and not a replacement for miniKanren.

## Status

The regular Lisp core is the most developed part of the project. It includes parsing, dotted pair printing, lexical environments, closures, common special forms, and a small set of eager builtins.

The miniKanren layer is still under construction. The design target includes:

- logic variables and substitutions
- `walk`, `unify`, and persistent solver state
- `run`, `fresh`, `==`, and `conde`
- reification of answers into readable values like `_.0`
- fair, interleaving search instead of naive depth-first search

Many examples in this repository are aspirational. Some are smoke tests for the Lisp core, some are design targets, and some require features that are not implemented yet. Treat failing examples as a map of future work, not as a sign that every listed behavior currently exists.

## Design

The main design notes live in [doc/design_doc.md](doc/design_doc.md).

The project follows three boundaries:

```text
Lisp layer:     What did the user type?
Evaluator:      What does this expression mean?
Kanren engine:  Given a goal and a state, what possible states follow?
```

The important constraint is that relational behavior belongs in the Kanren layer. The evaluator may build goals, invoke the engine, and reify answers, but it should not directly perform unification or search.

## Layout

```text
src/frontend/        lexer and parser
src/runtime/         values, environments, evaluator, forms, builtins
src/ui/              REPL, themes, highlighting, banner
src/kanren.rs        relational state, goals, unification, search work
src/kanren_forms.rs  Lisp-facing relational forms
doc/                 design and implementation notes
examples/            Lisp examples, many still aspirational
themes/              REPL themes
```

## Running

Start the REPL:

```sh
cargo run
```

Run a Lisp file:

```sh
cargo run -- examples/regular_lisp/fibonacci.lisp
```

The interpreter currently supports a small Lisp with integers, booleans, symbols, lists, lambdas, lexical scope, mutation through `set!`, and special forms such as `quote`, `if`, `define`, `lambda`, `let`, `begin`, `and`, and `or`.

## Examples

The files under [examples/regular_lisp](examples/regular_lisp) are partly tests, partly sketches of where the Lisp should go. Some depend on missing builtins or forms such as `apply`, `assoc`, `filter`, `define-syntax`, or `syntax-rules`.

When an example fails, check [examples/regular_lisp/README.md](examples/regular_lisp/README.md) for notes about the required features.

## Influences

voidKanren owes a lot to the miniKanren tradition and to the work of William Byrd and Daniel P. Friedman. Their work on relational programming, The Reasoned Schemer, and the broader miniKanren ecosystem is the obvious north star here.

Shoutouts also to Gerald Sussman, whose work shaped so much of the Lisp and Scheme world this project is learning from, and to Rich Hickey, whose ideas about simple systems and Lisp-family languages are always somewhere in the background of my thoughts.

Personal thanks to Brian BPM Murphy for piquing my curiosity about Lisp and Emacs years ago, and for being a generous mentor while I was asking a lot of beginner questions. Thanks also to Autumn West for putting up with me talking about Lisp at 2am every night.

If you want a mature relational programming system, use [miniKanren](https://minikanren.org/) or one of its established implementations. voidKanren is a learning and research playground for building toward that style of system from a small Rust Lisp.

## License

See [LICENSE](LICENSE).
