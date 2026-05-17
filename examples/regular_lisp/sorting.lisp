; sorting.lisp
; chaos enters the wired — we impose order before it propagates

(begin
  (define corrupted-data '(42 7 19 3 88 15 4 66 23 1 99 37))

  ; --- insertion sort: one packet at a time, slotted into place ---
  (define (insert x sorted)
    (cond ((null? sorted) (list x))
          ((<= x (car sorted)) (cons x sorted))
          (else (cons (car sorted) (insert x (cdr sorted))))))

  (define (insertion-sort xs)
    (if (null? xs)
        '()
        (insert (car xs) (insertion-sort (cdr xs)))))

  ; --- merge sort: divide the signal, conquer the noise ---
  (define (merge a b)
    (cond ((null? a) b)
          ((null? b) a)
          ((<= (car a) (car b))
           (cons (car a) (merge (cdr a) b)))
          (else
           (cons (car b) (merge a (cdr b))))))

  (define (split xs)
    (if (or (null? xs) (null? (cdr xs)))
        (cons xs '())
        (let ((rest (split (cddr xs))))
          (cons (cons (car xs) (car rest))
                (cons (cadr xs) (cdr rest))))))

  (define (merge-sort xs)
    (if (or (null? xs) (null? (cdr xs)))
        xs
        (let ((halves (split xs)))
          (merge (merge-sort (car halves))
                 (merge-sort (cdr halves))))))

  ; --- quicksort: partition the wired ---
  (define (quicksort xs)
    (if (or (null? xs) (null? (cdr xs)))
        xs
        (let ((pivot (car xs))
              (rest  (cdr xs)))
          (let ((below (filter (lambda (x) (< x pivot)) rest))
                (above (filter (lambda (x) (>= x pivot)) rest)))
            (append (quicksort below) (list pivot) (quicksort above))))))

  (display "corrupted:  ") (display corrupted-data) (newline)
  (display "insertion:  ") (display (insertion-sort corrupted-data)) (newline)
  (display "merge:      ") (display (merge-sort corrupted-data)) (newline)
  (display "quick:      ") (display (quicksort corrupted-data)) (newline))