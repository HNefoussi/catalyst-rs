use catalyst_core::{DataType, Expr, Field, Schema};

fn users_schema() -> Schema {
    Schema::new(vec![
        Field::new("id", DataType::Int64).not_null(),
        Field::new("name", DataType::Utf8),
        Field::new("age", DataType::Int32),
    ])
}

#[test]
fn renders_expressions_with_precedence() {
    let expr = Expr::col("age").plus(Expr::lit(1)).multiply(Expr::lit(2));
    assert_eq!(expr.to_string(), "(age + 1) * 2");

    let expr = Expr::col("a").minus(Expr::col("b").minus(Expr::col("c")));
    assert_eq!(expr.to_string(), "a - (b - c)");

    let expr = Expr::col("a").plus(Expr::col("b").plus(Expr::col("c")));
    assert_eq!(expr.to_string(), "a + b + c");

    let expr = Expr::col("a").or(Expr::col("b")).and(Expr::col("c"));
    assert_eq!(expr.to_string(), "(a OR b) AND c");
}

#[test]
fn renders_aliases_and_literals() {
    let expr = Expr::lit("O'Reilly").alias("publisher");
    assert_eq!(expr.to_string(), "'O''Reilly' AS publisher");
}

#[test]
fn infers_expression_types() {
    let schema = users_schema();

    let predicate = Expr::col("age").gt(Expr::lit(25));
    assert_eq!(predicate.data_type(&schema).unwrap(), DataType::Boolean);

    let sum = Expr::col("age").plus(Expr::lit(1.5));
    assert_eq!(sum.data_type(&schema).unwrap(), DataType::Float64);

    let alias = Expr::col("name").alias("username");
    assert_eq!(alias.data_type(&schema).unwrap(), DataType::Utf8);
    assert_eq!(alias.output_name(), "username");
}

#[test]
fn rejects_invalid_type_combinations() {
    let schema = users_schema();

    let expr = Expr::col("name").gt(Expr::lit(25));
    assert!(expr.data_type(&schema).is_err());

    let expr = Expr::col("age").and(Expr::lit(true));
    assert!(expr.data_type(&schema).is_err());
}

#[test]
fn collects_column_references() {
    let expr = Expr::col("age").plus(Expr::col("id")).gt(Expr::lit(18));

    let columns = expr.column_refs();
    let names: Vec<_> = columns.iter().map(|column| column.name.as_str()).collect();
    assert_eq!(names, vec!["age", "id"]);
}
