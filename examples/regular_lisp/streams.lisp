; streams.lisp
; infinite sequences — the wired has no end, only evaluation boundaries
; simulating lazy streams with thunks (lambda () ...)

(begin
  ; a stream is a pair: (head . thunk-for-tail)
  ; the tail is not evaluated until forced

  (define-syntax stream-cons
    (syntax-rules ()
      ((_ h t) (cons h (lambda () t)))))

  (define stream-car car)
  (define (stream-cdr s) ((cdr s)))
  (define stream-null '())
  (define (stream-null? s) (null? s))

  ; --- take: pull n elements from the signal ---
  (define (stream-take n s)
    (if (or (= n 0) (stream-null? s))
        '()
        (cons (stream-car s) (stream-take (- n 1) (stream-cdr s)))))

  ; --- map over a stream ---
  (define (stream-map f s)
    (if (stream-null? s)
        stream-null
        (stream-cons (f (stream-car s))
                     (stream-map f (stream-cdr s)))))

  ; --- filter a stream ---
  (define (stream-filter pred s)
    (cond ((stream-null? s) stream-null)
          ((pred (stream-car s))
           (stream-cons (stream-car s) (stream-filter pred (stream-cdr s))))
          (else (stream-filter pred (stream-cdr s)))))

  ; --- infinite integer stream starting at n ---
  (define (integers-from n)
    (stream-cons n (integers-from (+ n 1))))

  ; --- infinite fibonacci stream ---
  (define (fib-stream a b)
    (stream-cons a (fib-stream b (+ a b))))

  ; --- sieve of eratosthenes over a stream ---
  (define (sieve s)
    (let ((p (stream-car s)))
      (stream-cons p
        (sieve (stream-filter (lambda (x) (not (= (remainder x p) 0)))
                              (stream-cdr s))))))

  (define nats     (integers-from 0))
  (define fibs     (fib-stream 0 1))
  (define primes   (sieve (integers-from 2)))

  (display "first 10 naturals:   ") (display (stream-take 10 nats))   (newline)
  (display "first 10 fibonaccis: ") (display (stream-take 10 fibs))   (newline)
  (display "first 15 primes:     ") (display (stream-take 15 primes)) (newline)
  (display "squares 1-8:         ")
    (display (stream-take 8 (stream-map (lambda (x) (* x x)) (integers-from 1))))
    (newline)
  (display "even naturals 1-12:  ")
    (display (stream-take 6 (stream-filter even? (integers-from 0))))
    (newline))