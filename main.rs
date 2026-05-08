use std::collections::HashMap;

// Tiny logic variable. Not a value yet
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Var(usize);

// Tiny universe of values.
#[derive(Clone, Debug, PartialEq)]
enum Term {
    Var(Var),
    Int(i64),
    Sym(String),
    Pair(Box<Term>, Box<Term>),
    Nil,
}

// Current solver knowledge: Var(0) -> Int(42), etc.
type Subst = HashMap<Var, Term>;

// One possible timeline.
#[derive(Clone, Debug)]
struct State {
    subst: Subst,
    next_var: usize,
}

impl State {
    fn new() -> Self {
        Self {
            subst: HashMap::new(),
            next_var: 0,
        }
    }

    // Make a fresh unknown.
    fn fresh_var(&mut self) -> Var {
        let v = Var(self.next_var);
        self.next_var += 1;
        v
    }
}

// Build a Lisp-ish list: (1 2 3)
fn list(xs: Vec<Term>) -> Term {
    xs.into_iter().rev().fold(Term::Nil, |acc, x| {
        Term::Pair(Box::new(x), Box::new(acc))
    })
}

// Follow substitutions until the term stops changing.
// x -> y -> 5 means walk(x) = 5.
fn walk(term: &Term, subst: &Subst) -> Term {
    match term {
        Term::Var(v) => match subst.get(v) {
            Some(t) => walk(t, subst),
            None => term.clone(),
        },
        _ => term.clone(),
    }
}

// Can these two terms be made equal?
fn unify(a: &Term, b: &Term, subst: &Subst) -> Option<Subst> {
    let a = walk(a, subst);
    let b = walk(b, subst);

    match (a, b) {
        // Unknown meets thing. 
        (Term::Var(v), t) | (t, Term::Var(v)) => {
            let mut new_subst = subst.clone();
            new_subst.insert(v, t);
            Some(new_subst)
        }

        // Boring concrete equality.
        (Term::Int(x), Term::Int(y)) if x == y => Some(subst.clone()),
        (Term::Sym(x), Term::Sym(y)) if x == y => Some(subst.clone()),
        (Term::Nil, Term::Nil) => Some(subst.clone()),

        // Lists/pairs unify piece by piece.
        (Term::Pair(a1, d1), Term::Pair(a2, d2)) => {
            let s1 = unify(&a1, &a2, subst)?;
            unify(&d1, &d2, &s1)
        }

        // Reality rejected this branch.
        _ => None,
    }
}

// A goal takes one possible timeline and returns all timelines where it worked.
type Goal = Box<dyn Fn(State) -> Vec<State>>;

// Goal: u == v
fn eq(u: Term, v: Term) -> Goal {
    Box::new(move |state: State| match unify(&u, &v, &state.subst) {
        Some(subst) => vec![State {
            subst,
            next_var: state.next_var,
        }],
        None => vec![],
    })
}

// AND: run g1, then run g2 on every surviving timeline.
fn both(g1: Goal, g2: Goal) -> Goal {
    Box::new(move |state: State| {
        let mut out = Vec::new();

        for s in g1(state) {
            out.extend(g2(s));
        }

        out
    })
}

// OR: split reality. Both branches get a shot.
fn either(g1: Goal, g2: Goal) -> Goal {
    Box::new(move |state: State| {
        let mut out = Vec::new();

        out.extend(g1(state.clone()));
        out.extend(g2(state));

        out
    })
}

// membero(x, list)
// Means: "x is somewhere in list".
//
// x is a member if:
// - x is the head
// OR
// - x is in the tail
fn membero(x: Term, lst: Term) -> Goal {
    Box::new(move |state: State| {
        let mut s = state.clone();

        let head = Term::Var(s.fresh_var());
        let tail = Term::Var(s.fresh_var());

        // lst must look like (head . tail)
        let pair = Term::Pair(Box::new(head.clone()), Box::new(tail.clone()));

        let split = eq(lst.clone(), pair);

        let is_head = eq(x.clone(), head);
        let is_in_tail = membero(x.clone(), tail);

        let goal = both(split, either(is_head, is_in_tail));

        goal(s)
    })
}

// Resolve a term for printing.
fn reify(term: &Term, subst: &Subst) -> Term {
    match walk(term, subst) {
        Term::Pair(a, d) => Term::Pair(
            Box::new(reify(&a, subst)),
            Box::new(reify(&d, subst)),
        ),
        other => other,
    }
}

fn main() {
    let mut state = State::new();

    let x = Term::Var(state.fresh_var());

    let xs = list(vec![
        Term::Int(1),
        Term::Int(2),
        Term::Int(3),
    ]);

    let goal = membero(x.clone(), xs);

    let results = goal(state);

    println!("answers:");

    for s in results {
        println!("{:?}", reify(&x, &s.subst));
    }
}