use catalyst_core::{DataType, Expr, Field, LogicalPlan, Schema, TreeNode};
use catalyst_optimizer::Optimizer;

fn users_schema() -> Schema {
    Schema::new(vec![
        Field::new("id", DataType::Int64).not_null(),
        Field::new("name", DataType::Utf8),
        Field::new("age", DataType::Int32),
    ])
}

fn demo_plan() -> LogicalPlan {
    LogicalPlan::scan("users", users_schema())
        .filter(Expr::col("age").gt(Expr::lit(25)))
        .project(vec![Expr::col("name"), Expr::col("age")])
}

fn main() {
    let plan = demo_plan();

    println!("Catalyst-RS · Phase 1");
    println!("=====================\n");

    println!("Logical plan");
    println!("------------");
    println!("{plan}\n");

    println!("Debug tree");
    println!("----------");
    println!("{}\n", plan.tree_string());

    let output_schema = plan.schema().expect("the demo plan should be valid");
    println!("Output schema");
    println!("-------------");
    println!("{output_schema}\n");

    println!("Programmatic inspection");
    println!("-----------------------");

    let mut nodes = Vec::new();
    plan.foreach(|node| nodes.push(node.node_name()));
    println!("Pre-order nodes : {}", nodes.join(" -> "));

    let filters = plan.collect(|node| matches!(node, LogicalPlan::Filter(_)));
    println!("Filter nodes    : {}", filters.len());

    let scans = plan.collect(|node| matches!(node, LogicalPlan::Scan(_)));
    println!("Scan nodes      : {}", scans.len());

    if let Some(LogicalPlan::Scan(scan)) = plan.find(|node| matches!(node, LogicalPlan::Scan(_))) {
        println!("Base table      : {}", scan.table_name);
    }

    println!("\nExpression demo");
    println!("---------------");
    let expression = Expr::col("age").plus(Expr::lit(1)).alias("next_age");
    println!("{expression}");

    println!("\nTree transformation demo (rename users -> people)");
    println!("-------------------------------------------------");
    let rewritten = plan
        .clone()
        .transform_up(|node| match node {
            LogicalPlan::Scan(scan) => Ok(LogicalPlan::scan("people", scan.schema.clone())),
            other => Ok(other),
        })
        .expect("the transformation should succeed");
    println!("{rewritten}\n");

    println!("Optimizer scaffold");
    println!("------------------");
    let optimizer = Optimizer::new();
    let result = optimizer
        .optimize(plan)
        .expect("the optimizer should not fail");

    println!("Rules registered : {}", optimizer.rule_names().len());
    println!("Rules applied    : {}", result.applied.len());
    println!("Iterations       : {}", result.iterations);
    println!("Next step        : add rules in Phase 2");
}
