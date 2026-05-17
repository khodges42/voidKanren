; y-combinator.lisp
; you don't need a name to recurse — the fixed point always finds you

(begin
  ; The Y combinator: recursion from pure lambda, no names required
  ; Y = λf.(λx.f(x x))(λx.f(x x))
  (define Y
    (lambda (f)
      ((lambda (x) (f (lambda (v) ((x x) v))))
       (lambda (x) (f (lambda (v) ((x x) v)))))))

  ; factorial without ever naming itself
  (define phantom-factorial
    (Y (lambda (self)
         (lambda (n)
           (if (<= n 1)
               1
               (* n (self (- n 1))))))))

  ; fibonacci — nameless, haunting the call stack
  (define phantom-fib
    (Y (lambda (self)
         (lambda (n)
           (if (< n 2)
               n
               (+ (self (- n 1)) (self (- n 2))))))))

  ; anonymous list length — no identity, only function
  (define phantom-length
    (Y (lambda (self)
         (lambda (xs)
           (if (null? xs)
               0
               (+ 1 (self (cdr xs))))))))

  (display "phantom 5!  = ") (display (phantom-factorial 5))  (newline)
  (display "phantom 10! = ") (display (phantom-factorial 10)) (newline)
  (display "phantom fib(10) = ") (display (phantom-fib 10))  (newline)
  (display "phantom length of (a b c d e) = ")
    (display (phantom-length '(a b c d e))) (newline))