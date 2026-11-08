use crate::{Expr, LogicalPlan};

/// Keeps only rows for which `predicate` evaluates to `true`.
#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    pub predicate: Expr,
    pub input: Box<LogicalPlan>,
}

impl Filter {
    pub fn new(input: LogicalPlan, predicate: Expr) -> Self {
        Self {
            predicate,
            input: Box::new(input),
        }
    }
}
