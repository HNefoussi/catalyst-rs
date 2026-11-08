use std::fmt;

use crate::{Error, Result};

/// Supported value types.
///
/// The set is deliberately small for Phase 1. New types can be added here and
/// then handled by expression type inference in `expr`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    Null,
    Boolean,
    Int32,
    Int64,
    Float64,
    Utf8,
}

impl DataType {
    pub fn is_numeric(self) -> bool {
        matches!(self, Self::Int32 | Self::Int64 | Self::Float64)
    }

    pub fn is_boolean(self) -> bool {
        matches!(self, Self::Boolean)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Null => "Null",
            Self::Boolean => "Boolean",
            Self::Int32 => "Int32",
            Self::Int64 => "Int64",
            Self::Float64 => "Float64",
            Self::Utf8 => "Utf8",
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A named, typed column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Optional table qualifier, e.g. `users` in `users.age`.
    pub qualifier: Option<String>,
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}

impl Field {
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            qualifier: None,
            name: name.into(),
            data_type,
            nullable: true,
        }
    }

    /// Mark the field as non-nullable.
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    /// Attach a table qualifier to the field.
    pub fn qualified(mut self, qualifier: impl Into<String>) -> Self {
        self.qualifier = Some(qualifier.into());
        self
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.qualifier {
            Some(qualifier) => write!(f, "{}.{}: {}", qualifier, self.name, self.data_type),
            None => write!(f, "{}: {}", self.name, self.data_type),
        }
    }
}

/// An ordered collection of [`Field`]s.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Schema {
    fields: Vec<Field>,
}

impl Schema {
    pub fn new(fields: Vec<Field>) -> Self {
        Self { fields }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn field(&self, index: usize) -> Option<&Field> {
        self.fields.get(index)
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Find the index of the first field with the given unqualified name.
    pub fn index_of_name(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|field| field.name == name)
    }

    /// Resolve a column reference to a field index.
    ///
    /// Resolution rules:
    /// - an unqualified name matches any field with that name;
    /// - a qualified name prefers an exact qualifier match, then falls back to
    ///   fields without a qualifier;
    /// - multiple matches are reported as an ambiguity.
    pub fn resolve(&self, qualifier: Option<&str>, name: &str) -> Result<usize> {
        let mut exact = Vec::new();
        let mut unqualified = Vec::new();

        for (index, field) in self.fields.iter().enumerate() {
            if field.name != name {
                continue;
            }

            match (qualifier, field.qualifier.as_deref()) {
                (Some(requested), Some(actual)) if requested == actual => exact.push(index),
                (Some(_), None) => unqualified.push(index),
                (None, _) => exact.push(index),
                _ => {}
            }
        }

        let matches = if exact.is_empty() { unqualified } else { exact };
        match matches.as_slice() {
            [index] => Ok(*index),
            [] => Err(Error::ColumnNotFound {
                name: column_label(qualifier, name),
            }),
            _ => Err(Error::AmbiguousColumn {
                name: column_label(qualifier, name),
            }),
        }
    }

    /// Select fields by index. Useful for projection pushdown later.
    pub fn project(&self, indices: &[usize]) -> Result<Self> {
        let mut fields = Vec::with_capacity(indices.len());
        for &index in indices {
            let field = self.fields.get(index).ok_or_else(|| {
                Error::invalid_schema(format!(
                    "column index {index} is out of bounds for a schema with {} fields",
                    self.fields.len()
                ))
            })?;
            fields.push(field.clone());
        }
        Ok(Self::new(fields))
    }

    /// Concatenate two schemas, preserving order.
    pub fn join(&self, other: &Self) -> Self {
        let mut fields = Vec::with_capacity(self.fields.len() + other.fields.len());
        fields.extend(self.fields.iter().cloned());
        fields.extend(other.fields.iter().cloned());
        Self::new(fields)
    }

    /// Return a copy of this schema with every field qualified by `qualifier`.
    pub fn with_qualifier(&self, qualifier: impl Into<String>) -> Self {
        let qualifier = qualifier.into();
        let fields = self
            .fields
            .iter()
            .cloned()
            .map(|field| field.qualified(qualifier.clone()))
            .collect();
        Self::new(fields)
    }
}

impl FromIterator<Field> for Schema {
    fn from_iter<T: IntoIterator<Item = Field>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (index, field) in self.fields.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{field}")?;
        }
        write!(f, "]")
    }
}

fn column_label(qualifier: Option<&str>, name: &str) -> String {
    match qualifier {
        Some(qualifier) => format!("{qualifier}.{name}"),
        None => name.to_string(),
    }
}
