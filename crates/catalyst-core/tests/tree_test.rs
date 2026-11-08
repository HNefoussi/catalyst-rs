mod support;

use catalyst_core::{Column, Expr, LogicalPlan, TreeNode};
use support::demo_plan;

#[test]
fn traverses_nodes_in_pre_order() {
    let plan = demo_plan();
    let mut names = Vec::new();
    plan.foreach(|node| names.push(node.node_name()));
    assert_eq!(names, vec!["Project", "Filter", "Scan"]);
}

#[test]
fn collects_borrowed_nodes_with_foreach() {
    let plan = demo_plan();
    let mut scans = Vec::new();
    plan.foreach(|node| {
        if let LogicalPlan::Scan(scan) = node {
            scans.push(scan);
        }
    });

    assert_eq!(scans.len(), 1);
    assert_eq!(scans[0].table_name, "users");
}

#[test]
fn collects_nodes_matching_a_predicate() {
    let plan = demo_plan();

    let filters = plan.collect(|node| matches!(node, LogicalPlan::Filter(_)));
    assert_eq!(filters.len(), 1);

    let scans = plan.collect(|node| matches!(node, LogicalPlan::Scan(_)));
    assert_eq!(scans.len(), 1);
}

#[test]
fn finds_the_first_matching_node() {
    let plan = demo_plan();
    let scan = plan
        .find(|node| matches!(node, LogicalPlan::Scan(_)))
        .unwrap();

    match scan {
        LogicalPlan::Scan(scan) => assert_eq!(scan.table_name, "users"),
        _ => panic!("expected a scan"),
    }
}

#[test]
fn checks_whether_a_node_exists() {
    let plan = demo_plan();
    assert!(plan.exists(|node| node.node_name() == "Filter"));
    assert!(!plan.exists(|node| node.node_name() == "Join"));
}

#[test]
fn maps_direct_children() {
    let plan = demo_plan();
    let filter = plan.input().unwrap();

    let mapped = filter
        .map_children(|child| match child {
            LogicalPlan::Scan(scan) => Ok(LogicalPlan::scan("people", scan.schema.clone())),
            other => Ok(other),
        })
        .unwrap();

    assert!(mapped.to_string().contains("Scan: people"));
}

#[test]
fn transforms_logical_plans_bottom_up() {
    let plan = demo_plan();
    let rewritten = plan
        .transform_up(|node| match node {
            LogicalPlan::Scan(scan) => Ok(LogicalPlan::scan("people", scan.schema.clone())),
            other => Ok(other),
        })
        .unwrap();

    assert!(rewritten.to_string().contains("Scan: people"));
    assert!(!rewritten.to_string().contains("Scan: users"));
}

#[test]
fn transforms_expressions_with_the_same_framework() {
    let expr = Expr::col("age").plus(Expr::lit(1)).gt(Expr::lit(25));

    let rewritten = expr
        .transform_down(|node| match node {
            Expr::Column(column) if column.name == "age" => Ok(Expr::Column(Column::new("years"))),
            other => Ok(other),
        })
        .unwrap();

    assert_eq!(rewritten.to_string(), "years + 1 > 25");
}

#[test]
fn rejects_invalid_child_arity() {
    let expr = Expr::col("age");
    let error = expr.with_new_children(vec![Expr::lit(1)]).unwrap_err();
    assert!(error.to_string().contains("expected 0"));
}
