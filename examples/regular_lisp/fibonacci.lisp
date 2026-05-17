; fibonacci.lisp
; The sequence that echoes through the wired — each node sum of its ancestors

(begin
  (define (fib n)
    (if (< n 2)
        n
        (+ (fib (- n 1)) (fib (- n 2)))))

  (define (fib-sequence limit)
    (define (loop i acc)
      (if (> i limit)
          acc
          (loop (+ i 1) (cons (fib i) acc))))
    (loop 0 '()))

  (display (fib 10))
  (newline)
  (display (fib 20))
  (newline)
  (display (fib-sequence 10))
  (newline)
  (display (fib-sequence 24))
  (newline))