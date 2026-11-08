#![allow(dead_code)]

use catalyst_core::{DataType, Expr, Field, LogicalPlan, Schema};

pub fn users_schema() -> Schema {
    Schema::new(vec![
        Field::new("id", DataType::Int64).not_null(),
        Field::new("name", DataType::Utf8),
        Field::new("age", DataType::Int32),
    ])
}

pub fn demo_plan() -> LogicalPlan {
    LogicalPlan::scan("users", users_schema())
        .filter(Expr::col("age").gt(Expr::lit(25)))
        .project(vec![Expr::col("name"), Expr::col("age")])
}
