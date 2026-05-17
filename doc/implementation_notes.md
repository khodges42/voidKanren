# Random implementation thoughts

* Wondering if the flat structure of src will become a problem.
* Would be cool to make Kanren more modular, but probably no point rn
* DO we want comments to even be semis?
    * Thats what scheme does
    * but its weird looking why cant we have //
    * Maybe thats something in scheme already

* When we parse read_atom we will check if it's an int and then otherwise make it a symbol. Is that inefficient or dumb?

* There's a lot goingon in the lexer in lexer impl

* we give symbol for any text, should we check at the lexer level? oh wait no because we dont build env yet


* Need (exit)
* Known limitation: tail-recursive Lisp programs still consume Rust stack because v1 evaluator does not implement TCO/trampolining.