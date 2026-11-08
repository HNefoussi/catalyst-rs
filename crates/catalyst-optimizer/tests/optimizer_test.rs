use catalyst_core::{DataType, Expr, Field, LogicalPlan, Result, Schema, TreeNode};
use catalyst_optimizer::{Optimizer, Rule};

fn demo_plan() -> LogicalPlan {
    let schema = Schema::new(vec![
        Field::new("id", DataType::Int64).not_null(),
        Field::new("name", DataType::Utf8),
        Field::new("age", DataType::Int32),
    ]);

    LogicalPlan::scan("users", schema)
        .filter(Expr::col("age").gt(Expr::lit(25)))
        .project(vec![Expr::col("name"), Expr::col("age")])
}

struct RenameTableRule;

impl Rule for RenameTableRule {
    fn name(&self) -> &'static str {
        "RenameTable"
    }

    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        plan.transform_up(|node| match node {
            LogicalPlan::Scan(scan) => Ok(LogicalPlan::scan("people", scan.schema.clone())),
            other => Ok(other),
        })
    }
}

struct NoopRule;

impl Rule for NoopRule {
    fn name(&self) -> &'static str {
        "Noop"
    }

    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        Ok(plan)
    }
}

#[test]
fn applies_rules_until_a_fixed_point() {
    let optimizer = Optimizer::new().with_rule(RenameTableRule);
    let result = optimizer.optimize(demo_plan()).unwrap();

    assert_eq!(result.applied.len(), 1);
    assert_eq!(result.applied[0].name, "RenameTable");
    assert!(result.plan.to_string().contains("Scan: people"));
    assert_eq!(result.iterations, 2);
}

#[test]
fn reports_no_changes_for_noop_rules() {
    let optimizer = Optimizer::new().with_rule(NoopRule);
    let result = optimizer.optimize(demo_plan()).unwrap();

    assert!(result.applied.is_empty());
    assert!(!result.changed());
    assert_eq!(result.iterations, 1);
}

#[test]
fn skips_iterations_when_no_rules_are_registered() {
    let optimizer = Optimizer::new();
    let result = optimizer.optimize(demo_plan()).unwrap();

    assert!(result.applied.is_empty());
    assert_eq!(result.iterations, 0);
}

#[test]
fn exposes_registered_rule_names() {
    let optimizer = Optimizer::new()
        .with_rule(RenameTableRule)
        .with_rule(NoopRule);

    assert_eq!(optimizer.rule_names(), vec!["RenameTable", "Noop"]);
}
