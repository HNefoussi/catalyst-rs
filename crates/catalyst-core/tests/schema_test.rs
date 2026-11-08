use catalyst_core::{DataType, Error, Field, Schema};

fn users_schema() -> Schema {
    Schema::new(vec![
        Field::new("id", DataType::Int64).not_null(),
        Field::new("name", DataType::Utf8),
        Field::new("age", DataType::Int32),
    ])
}

#[test]
fn resolves_columns_by_name() {
    let schema = users_schema();
    assert_eq!(schema.resolve(None, "id").unwrap(), 0);
    assert_eq!(schema.resolve(None, "name").unwrap(), 1);
    assert_eq!(schema.resolve(None, "age").unwrap(), 2);
}

#[test]
fn resolves_qualified_columns() {
    let schema = users_schema().with_qualifier("users");
    assert_eq!(schema.resolve(Some("users"), "age").unwrap(), 2);
    assert!(schema.resolve(Some("orders"), "age").is_err());
}

#[test]
fn resolves_qualifiers_against_unqualified_fields() {
    let schema = users_schema();
    assert_eq!(schema.resolve(Some("users"), "age").unwrap(), 2);
}

#[test]
fn reports_missing_columns() {
    let error = users_schema().resolve(None, "salary").unwrap_err();
    assert!(matches!(error, Error::ColumnNotFound { .. }));
}

#[test]
fn reports_ambiguous_columns() {
    let left = Schema::new(vec![Field::new("id", DataType::Int64)]);
    let right = Schema::new(vec![Field::new("id", DataType::Int64)]);
    let joined = left.join(&right);

    let error = joined.resolve(None, "id").unwrap_err();
    assert!(matches!(error, Error::AmbiguousColumn { .. }));
}

#[test]
fn projects_fields_by_index() {
    let schema = users_schema();
    let projected = schema.project(&[2, 1]).unwrap();

    assert_eq!(projected.fields()[0].name, "age");
    assert_eq!(projected.fields()[1].name, "name");
}

#[test]
fn rejects_out_of_bounds_projection() {
    assert!(users_schema().project(&[9]).is_err());
}
