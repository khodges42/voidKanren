; trees.lisp
; the wired is a tree — every node a branch, every signal a traversal

(begin
  ; A binary tree node: (value left right)
  ; '() represents a null/leaf

  (define (make-node val left right) (list val left right))
  (define (node-val  t) (car t))
  (define (node-left t) (cadr t))
  (define (node-right t) (caddr t))
  (define (leaf? t) (null? t))

  ; --- insert into a binary search tree ---
  (define (bst-insert t val)
    (cond ((leaf? t) (make-node val '() '()))
          ((< val (node-val t))
           (make-node (node-val t) (bst-insert (node-left t) val) (node-right t)))
          ((> val (node-val t))
           (make-node (node-val t) (node-left t) (bst-insert (node-right t) val)))
          (else t))) ; already present

  ; --- in-order traversal: sorted signal extraction ---
  (define (bst-inorder t)
    (if (leaf? t)
        '()
        (append (bst-inorder (node-left t))
                (list (node-val t))
                (bst-inorder (node-right t)))))

  ; --- search ---
  (define (bst-search t val)
    (cond ((leaf? t) #f)
          ((= val (node-val t)) #t)
          ((< val (node-val t)) (bst-search (node-left t) val))
          (else (bst-search (node-right t) val))))

  ; --- depth: how deep does the wired go? ---
  (define (tree-depth t)
    (if (leaf? t)
        0
        (+ 1 (max (tree-depth (node-left t))
                  (tree-depth (node-right t))))))

  ; --- count nodes ---
  (define (tree-size t)
    (if (leaf? t)
        0
        (+ 1 (tree-size (node-left t)) (tree-size (node-right t)))))

  ; build a BST from the signal stream
  (define signal-stream '(50 30 70 20 40 60 80 10 25 35 45))

  (define wired-tree
    (define (build-tree vals)
      (if (null? vals)
          '()
          (bst-insert (build-tree (cdr vals)) (car vals))))
    (build-tree signal-stream))

  (display "sorted signal:   ") (display (bst-inorder wired-tree))  (newline)
  (display "depth of tree:   ") (display (tree-depth wired-tree))   (newline)
  (display "node count:      ") (display (tree-size wired-tree))    (newline)
  (display "search for 40:   ") (display (bst-search wired-tree 40)) (newline)
  (display "search for 99:   ") (display (bst-search wired-tree 99)) (newline))