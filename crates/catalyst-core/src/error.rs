use std::fmt;

/// Errors produced while building, validating, or transforming plans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A column reference could not be resolved against a schema.
    ColumnNotFound { name: String },
    /// A column reference matched more than one field.
    AmbiguousColumn { name: String },
    /// An expression combined incompatible types.
    TypeMismatch { message: String },
    /// A tree node received the wrong number of children.
    InvalidArity {
        node: &'static str,
        expected: usize,
        actual: usize,
    },
    /// A schema operation was invalid (for example, an out-of-bounds index).
    InvalidSchema { message: String },
    /// A feature is not implemented yet.
    Unsupported { feature: String },
    /// An optimizer-level failure.
    Optimizer { message: String },
}

impl Error {
    pub fn type_mismatch(message: impl Into<String>) -> Self {
        Self::TypeMismatch {
            message: message.into(),
        }
    }

    pub fn invalid_schema(message: impl Into<String>) -> Self {
        Self::InvalidSchema {
            message: message.into(),
        }
    }

    pub fn unsupported(feature: impl Into<String>) -> Self {
        Self::Unsupported {
            feature: feature.into(),
        }
    }

    pub fn optimizer(message: impl Into<String>) -> Self {
        Self::Optimizer {
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ColumnNotFound { name } => write!(f, "column not found: {name}"),
            Self::AmbiguousColumn { name } => {
                write!(f, "column reference is ambiguous: {name}")
            }
            Self::TypeMismatch { message } => write!(f, "type mismatch: {message}"),
            Self::InvalidArity {
                node,
                expected,
                actual,
            } => write!(
                f,
                "invalid number of children for {node}: expected {expected}, got {actual}"
            ),
            Self::InvalidSchema { message } => write!(f, "invalid schema: {message}"),
            Self::Unsupported { feature } => write!(f, "unsupported feature: {feature}"),
            Self::Optimizer { message } => write!(f, "optimizer error: {message}"),
        }
    }
}

impl std::error::Error for Error {}

/// Convenience alias used throughout the workspace.
pub type Result<T> = std::result::Result<T, Error>;
