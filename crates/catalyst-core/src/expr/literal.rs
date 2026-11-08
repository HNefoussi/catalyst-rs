use std::fmt;

use crate::DataType;

/// A literal value in an expression tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Null,
    Boolean(bool),
    Int32(i32),
    Int64(i64),
    Float64(f64),
    Utf8(String),
}

impl Literal {
    pub fn data_type(&self) -> DataType {
        match self {
            Self::Null => DataType::Null,
            Self::Boolean(_) => DataType::Boolean,
            Self::Int32(_) => DataType::Int32,
            Self::Int64(_) => DataType::Int64,
            Self::Float64(_) => DataType::Float64,
            Self::Utf8(_) => DataType::Utf8,
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "NULL"),
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Int32(value) => write!(f, "{value}"),
            Self::Int64(value) => write!(f, "{value}"),
            Self::Float64(value) => write!(f, "{value:?}"),
            Self::Utf8(value) => write!(f, "'{}'", value.replace('\'', "''")),
        }
    }
}

impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i32> for Literal {
    fn from(value: i32) -> Self {
        Self::Int32(value)
    }
}

impl From<i64> for Literal {
    fn from(value: i64) -> Self {
        Self::Int64(value)
    }
}

impl From<f64> for Literal {
    fn from(value: f64) -> Self {
        Self::Float64(value)
    }
}

impl From<&str> for Literal {
    fn from(value: &str) -> Self {
        Self::Utf8(value.to_string())
    }
}

impl From<String> for Literal {
    fn from(value: String) -> Self {
        Self::Utf8(value)
    }
}
