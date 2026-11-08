use crate::Schema;

/// A base-table scan.
#[derive(Debug, Clone, PartialEq)]
pub struct Scan {
    pub table_name: String,
    pub schema: Schema,
}

impl Scan {
    pub fn new(table_name: impl Into<String>, schema: Schema) -> Self {
        Self {
            table_name: table_name.into(),
            schema,
        }
    }
}
