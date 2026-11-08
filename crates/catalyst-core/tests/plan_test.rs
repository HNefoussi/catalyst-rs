mod support;

use catalyst_core::{DataType, Expr, LogicalPlan};
use support::{demo_plan, users_schema};

#[test]
fn prints_the_expected_plan() {
    let plan = demo_plan();
    assert_eq!(
        plan.to_string(),
        "Project: [name, age]\n    |\nFilter: age > 25\n    |\nScan: users"
    );
}

#[test]
fn box_tree_is_available_for_debugging() {
    let plan = demo_plan();
    assert_eq!(
        plan.tree_string(),
        "Project: [name, age]\n`- Filter: age > 25\n   `- Scan: users"
    );
}

#[test]
fn derives_the_output_schema() {
    let plan = demo_plan();
    let schema = plan.schema().unwrap();

    assert_eq!(schema.len(), 2);
    assert_eq!(schema.fields()[0].name, "name");
    assert_eq!(schema.fields()[0].data_type, DataType::Utf8);
    assert_eq!(schema.fields()[1].name, "age");
    assert_eq!(schema.fields()[1].data_type, DataType::Int32);
}

#[test]
fn project_uses_alias_names() {
    let plan = LogicalPlan::scan("users", users_schema())
        .project(vec![Expr::col("age").plus(Expr::lit(1)).alias("next_age")]);
    let schema = plan.schema().unwrap();

    assert_eq!(schema.fields()[0].name, "next_age");
    assert_eq!(schema.fields()[0].data_type, DataType::Int32);
}

#[test]
fn validates_filter_predicates() {
    let plan = LogicalPlan::scan("users", users_schema()).filter(Expr::col("age"));
    assert!(plan.schema().is_err());
}

#[test]
fn exposes_the_input_of_unary_operators() {
    let plan = demo_plan();
    let filter = plan.input().unwrap();
    assert_eq!(filter.display_header(), "Filter: age > 25");
    assert_eq!(filter.input().unwrap().display_header(), "Scan: users");
    assert!(filter.input().unwrap().input().is_none());
}
