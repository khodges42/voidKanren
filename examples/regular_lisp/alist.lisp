; alist.lisp
; association lists — the wired's oldest key-value store, no database required

(begin
  ; --- a-list: the dossier on active wired nodes ---
  (define node-registry
    '((lain    . "present-day-present-time")
       (alice   . "navi-jj-model-zero")
       (arisu   . "cyberia-regular")
       (eiri    . "protocol-seven")
       (mika    . "offline")
       (taro    . "cyberia-regular")))

  ; --- lookup: find a record in the dossier ---
  (define (registry-lookup key db)
    (let ((entry (assoc key db)))
      (if entry
          (cdr entry)
          'not-found-in-wired)))

  ; --- insert/update: add or overwrite a record ---
  (define (registry-set key val db)
    (cons (cons key val)
          (filter (lambda (entry) (not (eq? (car entry) key))) db)))

  ; --- delete: scrub a record from the registry ---
  (define (registry-delete key db)
    (filter (lambda (entry) (not (eq? (car entry) key))) db))

  ; --- keys and values ---
  (define (registry-keys db) (map car db))
  (define (registry-values db) (map cdr db))

  ; --- symbolic expression evaluator using alists as environments ---
  (define (eval-with-env expr env)
    (cond ((number? expr) expr)
          ((symbol? expr) (registry-lookup expr env))
          ((eq? (car expr) '+)
           (+ (eval-with-env (cadr expr) env)
              (eval-with-env (caddr expr) env)))
          ((eq? (car expr) '*)
           (* (eval-with-env (cadr expr) env)
              (eval-with-env (caddr expr) env)))
          (else 'unknown-opcode)))

  (define cipher-env
    '((voltage . 12) (resistance . 4) (frequency . 3)))

  (display "lain's node:          ") (display (registry-lookup 'lain node-registry)) (newline)
  (display "mika's node:          ") (display (registry-lookup 'mika node-registry)) (newline)
  (display "phantom node lookup:  ") (display (registry-lookup 'deus node-registry)) (newline)

  (define updated-registry (registry-set 'mika "back-online"))
  (display "mika updated:         ") (display (registry-lookup 'mika updated-registry)) (newline)

  (display "all node ids:         ") (display (registry-keys node-registry)) (newline)
  (display "voltage * resistance: ")
    (display (eval-with-env '(* voltage resistance) cipher-env)) (newline)
  (display "freq + resistance:    ")
    (display (eval-with-env '(+ frequency resistance) cipher-env)) (newline))