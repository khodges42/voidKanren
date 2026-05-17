; lists.lisp
; data flows through the wired in streams — learn to shape them

(begin
  (define ghost-signal '(6 1 8 0 3 3 7 2 4 9))

  ; --- map: transform every node in the stream ---
  (define (signal-map f stream)
    (if (null? stream)
        '()
        (cons (f (car stream)) (signal-map f (cdr stream)))))

  ; --- filter: let only certain packets through ---
  (define (signal-filter pred stream)
    (cond ((null? stream) '())
          ((pred (car stream)) (cons (car stream) (signal-filter pred (cdr stream))))
          (else (signal-filter pred (cdr stream)))))

  ; --- fold: collapse the stream into a single value ---
  (define (signal-fold f init stream)
    (if (null? stream)
        init
        (signal-fold f (f init (car stream)) (cdr stream))))

  ; --- reverse: flip the data stream ---
  (define (signal-reverse stream)
    (signal-fold (lambda (acc x) (cons x acc)) '() stream))

  ; --- flatten: dissolve nested structure ---
  (define (flatten xs)
    (cond ((null? xs) '())
          ((pair? (car xs)) (append (flatten (car xs)) (flatten (cdr xs))))
          (else (cons (car xs) (flatten (cdr xs))))))

  (display "raw signal:       ") (display ghost-signal) (newline)
  (display "doubled:          ") (display (signal-map (lambda (x) (* x 2)) ghost-signal)) (newline)
  (display "evens only:       ") (display (signal-filter even? ghost-signal)) (newline)
  (display "sum:              ") (display (signal-fold + 0 ghost-signal)) (newline)
  (display "reversed:         ") (display (signal-reverse ghost-signal)) (newline)
  (display "flattened:        ") (display (flatten '((1 2) (3 (4 5)) (6)))) (newline))