# lisp examples

A collection of example `.lisp` files for testing and showcasing the interpreter. Each file is self-contained, wrapped in a `begin` block, and prints its own output. They cover the classic curriculum — from basic recursion up through lazy infinite streams and symbolic calculus.

---

## files

### `fibonacci.lisp`
The obligatory entry point. Naive recursive Fibonacci, plus a sequence builder that collects the first `n` terms into a list. Good smoke test for basic recursion and arithmetic.

### `lists.lisp`
Hand-rolled list operations: `map`, `filter`, `fold`, `reverse`, and `flatten`. None of these rely on built-ins — they're all built up from `car`, `cdr`, `cons`, and `null?`. A solid test of recursive list traversal and higher-order functions.

### `church.lisp`
Church numerals — numbers encoded as pure lambda, no integers involved. Implements zero through three, then `succ`, `church-add`, and `church-mul`, with a `church->int` decoder to verify the math. Tests that your lambda application and closure semantics are working at the deepest level.

### `y-combinator.lisp`
The Y combinator, and three functions defined with it: factorial, Fibonacci, and list length — all anonymous, no `define` used for the recursion itself. If your interpreter handles this correctly, your lambda semantics are solid.

### `sorting.lisp`
Three classic sorting algorithms — insertion sort, merge sort, and quicksort — all operating on the same input list so you can verify they agree. Tests recursion depth, `append`, and `filter`.

> **Note:** relies on `filter` as a built-in. If you haven't added that yet, swap it for a hand-rolled version from `lists.lisp`.

### `closures.lisp`
Closure-heavy patterns: a stateful counter, an accumulator, a memoizer, function composition, and partial application. Tests `set!` for mutation, `let` for local scope, `apply` for variadic dispatch, and `assoc` for cache lookup.

> **Requires:** `set!`, `apply`, `assoc`

### `alist.lisp`
Association lists used as a key-value store — lookup, insert/update, delete, and key/value enumeration. Also includes a tiny symbolic expression evaluator that uses an alist as its environment. Good test of `assoc`, `filter`, `map`, and structural recursion.

### `trees.lisp`
Binary search tree built from scratch using lists as nodes. Implements insert, in-order traversal (which should return a sorted list), search, depth, and size. Tests mutual recursion and the `max` function.

> **Note:** uses a `define` inside a `let` body for the internal `build-tree` helper — worth checking that your scoping handles inner definitions.

### `tail-calls.lisp`
Tail-recursive versions of factorial, Fibonacci, length, reverse, and sum — all using explicit accumulator loops. Then the same functions again in continuation-passing style (CPS), where the "rest of the computation" is passed as a lambda. Tests whether your interpreter handles tail position correctly and can pass/call continuations as first-class values.

### `symbolic-diff.lisp`
Symbolic differentiation of algebraic expressions — a SICP classic. Expressions are plain s-expressions like `(+ (* 2 x) (expt x 3))`, and the differentiator walks them structurally, applying calculus rules and simplifying on the way back up. No numbers are harmed; the output is another s-expression. Tests deep structural recursion, `cond`, and smart constructor patterns.

### `streams.lisp`
Lazy infinite streams simulated with thunks — a `lambda` that produces the tail only when forced. Builds infinite sequences of naturals, Fibonacci numbers, and primes (via the Sieve of Eratosthenes), then pulls finite prefixes out with `stream-take`.

> **Requires:** `define-syntax` / `syntax-rules` for the `stream-cons` macro. If macros aren't wired up yet, you can manually expand `stream-cons` into `(cons h (lambda () t))` at every call site.

---

## compatibility notes

All files are wrapped in `(begin ...)` since the interpreter currently requires a single top-level expression for multi-expression files.

Files that require specific features beyond basic Lisp:

| Feature | Used in |
|---|---|
| `set!` mutation | `closures.lisp` |
| `apply` | `closures.lisp` |
| `filter` built-in | `sorting.lisp` |
| `assoc` built-in | `closures.lisp`, `alist.lisp` |
| `define-syntax` / `syntax-rules` | `streams.lisp` |
| `even?` built-in | `lists.lisp` |
| `remainder` built-in | `streams.lisp` |

If a file throws on a missing built-in, it's a good prompt to add it — or to implement it in Lisp itself first as an exercise.