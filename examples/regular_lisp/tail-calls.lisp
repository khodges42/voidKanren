; tail-calls.lisp
; tail position: the last whisper before the stack unwinds
; CPS: continuation-passing style — pass the future as a function

(begin
  ; --- tail-recursive factorial: no stack buildup ---
  (define (factorial-tc n)
    (define (loop n acc)
      (if (<= n 1)
          acc
          (loop (- n 1) (* n acc))))
    (loop n 1))

  ; --- tail-recursive fibonacci using two accumulators ---
  (define (fib-tc n)
    (define (loop n a b)
      (if (= n 0)
          a
          (loop (- n 1) b (+ a b))))
    (loop n 0 1))

  ; --- CPS factorial: the future is explicit ---
  (define (factorial-cps n k)
    (if (<= n 1)
        (k 1)
        (factorial-cps (- n 1)
                       (lambda (result) (k (* n result))))))

  ; --- CPS fibonacci ---
  (define (fib-cps n k)
    (if (< n 2)
        (k n)
        (fib-cps (- n 1)
                 (lambda (r1)
                   (fib-cps (- n 2)
                             (lambda (r2)
                               (k (+ r1 r2))))))))

  ; --- tail-recursive list operations ---
  (define (length-tc xs)
    (define (loop xs acc)
      (if (null? xs)
          acc
          (loop (cdr xs) (+ acc 1))))
    (loop xs 0))

  (define (reverse-tc xs)
    (define (loop xs acc)
      (if (null? xs)
          acc
          (loop (cdr xs) (cons (car xs) acc))))
    (loop xs '()))

  (define (sum-tc xs)
    (define (loop xs acc)
      (if (null? xs)
          acc
          (loop (cdr xs) (+ acc (car xs)))))
    (loop xs 0))

  (display "10! tail-call:       ") (display (factorial-tc 10)) (newline)
  (display "fib(30) tail-call:   ") (display (fib-tc 30))       (newline)
  (display "7! via CPS:          ") (factorial-cps 7 (lambda (x) (display x) (newline)))
  (display "fib(10) via CPS:     ") (fib-cps 10 (lambda (x) (display x) (newline)))

  (define payload '(1 2 3 4 5 6 7 8 9 10))
  (display "length (tail):       ") (display (length-tc payload))  (newline)
  (display "reversed (tail):     ") (display (reverse-tc payload)) (newline)
  (display "sum (tail):          ") (display (sum-tc payload))     (newline))