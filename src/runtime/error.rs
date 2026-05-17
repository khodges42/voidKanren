#[derive(Debug, Clone)]
pub enum LispError {
    Parse {
        message: String,
        line: usize,
        column: usize,
    },
    Eval {
        message: String,
    },
    Type {
        message: String,
    },
    Arity {
        message: String,
    },
    UnboundSymbol {
        name: String,
    },
}

pub type LispResult<T> = Result<T, LispError>;