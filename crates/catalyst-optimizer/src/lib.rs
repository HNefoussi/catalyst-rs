//! # Catalyst-RS Optimizer
//!
//! The rule engine scaffold for Catalyst-RS.
//!
//! Phase 1 deliberately ships no concrete optimization rules. The goal of this
//! crate is to define the extension point that Phase 2 will populate with
//! Catalyst-inspired rules such as predicate pushdown, constant folding,
//! projection pruning, and boolean simplification.
//!
//! A rule is just a function from a logical plan to a new logical plan:
//!
//! ```
//! use catalyst_core::{LogicalPlan, Result, TreeNode};
//! use catalyst_optimizer::Rule;
//!
//! struct RenameScanRule;
//!
//! impl Rule for RenameScanRule {
//!     fn name(&self) -> &'static str {
//!         "RenameScan"
//!     }
//!
//!     fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
//!         plan.transform_up(|node| match node {
//!             LogicalPlan::Scan(scan) => Ok(LogicalPlan::scan("people", scan.schema.clone())),
//!             other => Ok(other),
//!         })
//!     }
//! }
//! ```

use catalyst_core::{LogicalPlan, Result};

/// A single logical-plan rewrite.
///
/// Rules must be deterministic and should return the input unchanged when they
/// do not apply. The optimizer uses plan equality to detect real changes.
pub trait Rule: Send + Sync {
    fn name(&self) -> &'static str;
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan>;
}

/// A record of one rule application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedRule {
    pub name: String,
    pub iteration: usize,
}

/// The result of running the optimizer.
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizeResult {
    pub plan: LogicalPlan,
    pub applied: Vec<AppliedRule>,
    pub iterations: usize,
}

impl OptimizeResult {
    pub fn changed(&self) -> bool {
        !self.applied.is_empty()
    }
}

/// Applies a batch of rules until the plan reaches a fixed point.
///
/// This mirrors the shape of Catalyst's rule batches: every rule is applied in
/// order, and the whole batch is repeated until no rule changes the plan (or
/// the iteration limit is reached).
pub struct Optimizer {
    rules: Vec<Box<dyn Rule>>,
    max_iterations: usize,
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Optimizer {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            max_iterations: 10,
        }
    }

    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations.max(1);
        self
    }

    pub fn with_rule<R: Rule + 'static>(mut self, rule: R) -> Self {
        self.add_rule(rule);
        self
    }

    pub fn add_rule<R: Rule + 'static>(&mut self, rule: R) {
        self.rules.push(Box::new(rule));
    }

    pub fn rule_names(&self) -> Vec<&str> {
        self.rules.iter().map(|rule| rule.name()).collect()
    }

    pub fn max_iterations(&self) -> usize {
        self.max_iterations
    }

    /// Run all rules to a fixed point.
    pub fn optimize(&self, plan: LogicalPlan) -> Result<OptimizeResult> {
        if self.rules.is_empty() {
            return Ok(OptimizeResult {
                plan,
                applied: Vec::new(),
                iterations: 0,
            });
        }

        let mut current = plan;
        let mut applied = Vec::new();
        let mut iterations = 0;

        for iteration in 0..self.max_iterations {
            iterations = iteration + 1;
            let mut changed = false;

            for rule in &self.rules {
                let next = rule.apply(current.clone())?;
                if next != current {
                    applied.push(AppliedRule {
                        name: rule.name().to_string(),
                        iteration,
                    });
                    current = next;
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }

        Ok(OptimizeResult {
            plan: current,
            applied,
            iterations,
        })
    }
}
