; symbolic-diff.lisp
; the wired can differentiate itself — symbolic calculus, no numerics needed
; a SICP classic. the machine knows what change means.

(begin
  ; expressions are s-expressions:
  ;   number   -> literal
  ;   symbol   -> variable
  ;   (+ e e)  -> sum
  ;   (* e e)  -> product
  ;   (expt e n) -> power (integer exponent)

  (define (variable? e) (symbol? e))
  (define (same-variable? e v) (and (variable? e) (eq? e v)))
  (define (sum? e)     (and (pair? e) (eq? (car e) '+)))
  (define (product? e) (and (pair? e) (eq? (car e) '*)))
  (define (power? e)   (and (pair? e) (eq? (car e) 'expt)))

  (define (addend e)    (cadr e))
  (define (augend e)    (caddr e))
  (define (multiplier e)   (cadr e))
  (define (multiplicand e) (caddr e))
  (define (base e)     (cadr e))
  (define (exponent e) (caddr e))

  ; smart constructors — simplify on construction
  (define (make-sum a b)
    (cond ((and (number? a) (number? b)) (+ a b))
          ((eqv? a 0) b)
          ((eqv? b 0) a)
          (else (list '+ a b))))

  (define (make-product a b)
    (cond ((and (number? a) (number? b)) (* a b))
          ((or (eqv? a 0) (eqv? b 0)) 0)
          ((eqv? a 1) b)
          ((eqv? b 1) a)
          (else (list '* a b))))

  (define (make-power base exp)
    (cond ((eqv? exp 0) 1)
          ((eqv? exp 1) base)
          (else (list 'expt base exp))))

  ; --- the differentiator ---
  (define (deriv expr var)
    (cond
      ((number? expr)   0)
      ((variable? expr) (if (same-variable? expr var) 1 0))
      ((sum? expr)
       (make-sum (deriv (addend expr) var)
                 (deriv (augend expr) var)))
      ((product? expr)
       (make-sum (make-product (multiplier expr)
                               (deriv (multiplicand expr) var))
                 (make-product (deriv (multiplier expr) var)
                               (multiplicand expr))))
      ((power? expr)
       (make-product (exponent expr)
                     (make-product (make-power (base expr)
                                               (- (exponent expr) 1))
                                   (deriv (base expr) var))))
      (else 'unknown-signal-form)))

  ; d/dx (x^3 + 2x^2 + 5x + 7)
  (define ghost-polynomial '(+ (expt x 3) (+ (* 2 (expt x 2)) (+ (* 5 x) 7))))

  (display "f(x)    = x^3 + 2x^2 + 5x + 7") (newline)
  (display "f'(x)   = ") (display (deriv ghost-polynomial 'x)) (newline)
  (display "d/dx x^5  = ") (display (deriv '(expt x 5) 'x)) (newline)
  (display "d/dx x*y  wrt x = ") (display (deriv '(* x y) 'x)) (newline)
  (display "d/dx x*y  wrt y = ") (display (deriv '(* x y) 'y)) (newline))