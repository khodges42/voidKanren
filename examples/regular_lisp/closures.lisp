; closures.lisp
; a closure is memory with edges — state sealed inside lambda, inaccessible from outside

(begin
  ; --- counter: mutable state trapped in a closure ---
  (define (make-netstalker-counter)
    (let ((packet-count 0))
      (lambda (msg)
        (cond ((eq? msg 'ping)
               (set! packet-count (+ packet-count 1))
               packet-count)
              ((eq? msg 'reset)
               (set! packet-count 0))
              ((eq? msg 'read)
               packet-count)))))

  ; --- accumulator: each call deepens the signal ---
  (define (make-accumulator seed)
    (lambda (x)
      (set! seed (+ seed x))
      seed))

  ; --- memoize: cache the wired's answers so we don't ask twice ---
  (define (memoize f)
    (let ((cache '()))
      (lambda args
        (let ((cached (assoc args cache)))
          (if cached
              (cdr cached)
              (let ((result (apply f args)))
                (set! cache (cons (cons args result) cache))
                result))))))

  ; --- compose: chain functions like relay nodes ---
  (define (compose . fns)
    (if (null? fns)
        (lambda (x) x)
        (lambda (x)
          ((car fns) ((apply compose (cdr fns)) x)))))

  ; --- partial application: lock in some arguments, leave others open ---
  (define (partial f . bound-args)
    (lambda rest-args
      (apply f (append bound-args rest-args))))

  (define counter (make-netstalker-counter))
  (counter 'ping) (counter 'ping) (counter 'ping)
  (display "packets intercepted: ") (display (counter 'read)) (newline)

  (define signal-acc (make-accumulator 0))
  (signal-acc 10) (signal-acc 33) (signal-acc 7)
  (display "accumulated signal:  ") (display (signal-acc 0)) (newline)

  (define fast-fib
    (memoize (lambda (n)
               (if (< n 2) n (+ (fast-fib (- n 1)) (fast-fib (- n 2)))))))
  (display "memoized fib(30):    ") (display (fast-fib 30)) (newline)

  (define hex->dec->double
    (compose (lambda (x) (* x 2))
             (lambda (x) (+ x 100))))
  (display "composed(5):         ") (display (hex->dec->double 5)) (newline)

  (define add-payload (partial + 1337))
  (display "add-payload(42):     ") (display (add-payload 42)) (newline))