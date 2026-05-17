; church.lisp
; before silicon, before voltage — numbers as pure lambda, the original ghost

(begin
  ; Church numerals: a number n is (lambda (f) (lambda (x) (f applied n times to x)))
  (define zero  (lambda (f) (lambda (x) x)))
  (define one   (lambda (f) (lambda (x) (f x))))
  (define two   (lambda (f) (lambda (x) (f (f x)))))
  (define three (lambda (f) (lambda (x) (f (f (f x))))))

  ; successor: jack in one more layer
  (define (succ n)
    (lambda (f) (lambda (x) (f ((n f) x)))))

  ; addition: merge two signal depths
  (define (church-add m n)
    (lambda (f) (lambda (x) ((m f) ((n f) x)))))

  ; multiplication: nested recursion in the wired
  (define (church-mul m n)
    (lambda (f) (n (m f))))

  ; decode: collapse a Church numeral back to an integer
  (define (church->int n)
    ((n (lambda (x) (+ x 1))) 0))

  (define four  (succ three))
  (define five  (church-add two three))
  (define six   (church-mul two three))

  (display "zero:        ") (display (church->int zero))  (newline)
  (display "three:       ") (display (church->int three)) (newline)
  (display "four:        ") (display (church->int four))  (newline)
  (display "2 + 3 = 5:   ") (display (church->int five))  (newline)
  (display "2 * 3 = 6:   ") (display (church->int six))   (newline))