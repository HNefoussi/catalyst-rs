use crate::{Expr, LogicalPlan};

/// Produces a new set of columns from its input.
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub exprs: Vec<Expr>,
    pub input: Box<LogicalPlan>,
}

impl Project {
    pub fn new(input: LogicalPlan, exprs: Vec<Expr>) -> Self {
        Self {
            exprs,
            input: Box::new(input),
        }
    }
}
