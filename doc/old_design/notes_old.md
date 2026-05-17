## Lisp REPL with MiniKanren style Thingy

main.rs
lexer.rs          // tokenization
parser.rs         // tokens -> AST / Value
value.rs          // runtime values
env.rs            // lexical environment
eval.rs           // core evaluator
forms.rs          // special forms: if, define, lambda, let, quote, begin
builtins.rs       // +, -, cons, car, cdr, map, etc.
kanren.rs         // State, Goal, walk, unify, reify
kanren_forms.rs   // run, fresh, ==, conde, conj/disj syntax
printer.rs        // Display / pretty printing
error.rs          // LispError

main.rs        -> repl.rs / driver.rs
parser.rs      -> reader.rs if you want Lisp vibes
value.rs       -> datum.rs / object.rs / value.rs
evaluator.rs   -> eval.rs
environment.rs -> env.rs
kanren.rs      -> logic.rs / relational.rs / microkanren.rs

1. ints, bools, symbols, lists
2. quote
3. cons/car/cdr/list
4. define
5. lambda + closures
6. if/begin/let
7. equal?
8. logic vars
9. unify/walk
10. goals
11. run/fresh/conde
12. reify


Keep boundaries clear!
Lisp syntax layer:
  "what did the user type?"

Kanren engine:
  "how do goals produce states?"


* Main
    * Flow ->
        * Set up the env, io stuff
        * intro thing (should be cool... shit do we use curses?)
            * Maybe we have a curses option. have a dope ui?
                * chill thats 2.0
        * big loop.
            * Parse, eval, that loop.
        * Helper for balanced parens (what didnt I have that helper elsewhere?)
* Parser
    * Token enums
        * Tokenize
            * Parses them
    * Parser (tokens: Vec<Token>, pos:usize (parser’s cursor/index into the token stream))
    * Parse expressions, parse lists
        * Int, symbols, parens, quoetes, bools, dot (improper pair/list (not (cons 1(cons 2))))
    * peek token, get next token
    * Helper to find if a delimeter (maybe we can strip the source instead? Maybe thats bad.)
    
* Value
    * Our types of values...
        * Ints, bools, nils, lambdas, symbols
        * pairs (for kanren), builtins,logic variables
        * For lambdas we can have it be a struct with params, body, closure (env)
    * probably some conversions (lists, to list, to int, to string?, boolean stuff)
    * Partial Eq for value
    * fmt::Debug for value, fmt::Display for value
    * helper to write_list
* Evaluator
    * Something to create the base env.
        * There will be a big mapping from values to functions (builtins)
            * Maybe thats not appropriate to do here idk, seems like its ok
    * eval
        * a match, int, symbols, whatever
    * eval list
    * apply (again matching func)
        * handle lambdas here
    * eval quotes
    * eval ifs
    * conds, ands, ors,
    * probably have define here
    * eval_set, eval_lambda, eval_let, eval_begin, eval_run,
        * Do not make eval_run contain the solver.
    * goal handling eval
        * I know this will be huge, this is the whole solver. Maybe this doesnt go here?
    * all of our eval_whatever is here. Do I like that? Where else does it go.
        * Maybe we can make this more modular and readable?
    * All of our bultins are here (no. move them to builtins.rs, and call install_builtins(env))
        * builtin_add
        * builtin_sub
        * mul, div, mod, number equals, lessthan, greaterthan,  lte, gte, cons, car, cdr, list
        * booleans for is type, equal, logic, test stufffff, append
        * map goes here too.
        * if, quote, lambda, define, let, begin, run, fresh, maybe conde probably cannot be normal builtins unless you use macros/lazy wrappers.
* Environment
    * Inner env (bindings)
    * Env impl
        * can create it with a parent for closures
        * get, set
        * Anything for relational?
            No, put that elsewhere.
* Kanren (Relational Features)
    * State
        * Can create fresh variable
    * concept of goal
    * Value -> Walk
    * Unify (make these two things equal, possibly by binding logic variables.)
    * Reify (turns symbolic state into readable)
    * Cnj, disj(?), eq goal