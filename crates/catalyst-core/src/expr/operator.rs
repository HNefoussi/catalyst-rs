use std::fmt;

/// Binary operators supported by the Phase 1 expression model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

impl BinaryOperator {
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulo => "%",
            Self::Eq => "=",
            Self::NotEq => "<>",
            Self::Lt => "<",
            Self::LtEq => "<=",
            Self::Gt => ">",
            Self::GtEq => ">=",
            Self::And => "AND",
            Self::Or => "OR",
        }
    }

    /// Higher values bind more tightly. Used by the expression printer.
    pub fn precedence(self) -> u8 {
        match self {
            Self::Or => 1,
            Self::And => 2,
            Self::Eq | Self::NotEq | Self::Lt | Self::LtEq | Self::Gt | Self::GtEq => 3,
            Self::Plus | Self::Minus => 4,
            Self::Multiply | Self::Divide | Self::Modulo => 5,
        }
    }

    pub fn is_logical(self) -> bool {
        matches!(self, Self::And | Self::Or)
    }

    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            Self::Eq | Self::NotEq | Self::Lt | Self::LtEq | Self::Gt | Self::GtEq
        )
    }

    pub fn is_arithmetic(self) -> bool {
        matches!(
            self,
            Self::Plus | Self::Minus | Self::Multiply | Self::Divide | Self::Modulo
        )
    }

    /// Whether `a op (b op c)` can be rendered without parentheses.
    pub fn is_associative(self) -> bool {
        matches!(self, Self::Plus | Self::Multiply | Self::And | Self::Or)
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}
