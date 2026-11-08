mod literal;
mod operator;

pub use literal::Literal;
pub use operator::BinaryOperator;

use std::fmt;

use crate::tree::invalid_arity;
use crate::{DataType, Error, Result, Schema, TreeNode};

/// A reference to a column, optionally qualified by a table name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Column {
    pub qualifier: Option<String>,
    pub name: String,
}

impl Column {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            qualifier: None,
            name: name.into(),
        }
    }

    pub fn qualified(qualifier: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            qualifier: Some(qualifier.into()),
            name: name.into(),
        }
    }

    pub fn display_name(&self) -> String {
        match &self.qualifier {
            Some(qualifier) => format!("{qualifier}.{}", self.name),
            None => self.name.clone(),
        }
    }
}

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.qualifier {
            Some(qualifier) => write!(f, "{qualifier}.{}", self.name),
            None => f.write_str(&self.name),
        }
    }
}

/// A binary expression such as `age > 25` or `price * quantity`.
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: BinaryOperator,
    pub right: Box<Expr>,
}

impl BinaryExpr {
    pub fn new(left: Expr, op: BinaryOperator, right: Expr) -> Self {
        Self {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    pub fn data_type(&self, schema: &Schema) -> Result<DataType> {
        let left = self.left.data_type(schema)?;
        let right = self.right.data_type(schema)?;

        if self.op.is_logical() {
            if !left.is_boolean() {
                return Err(Error::type_mismatch(format!(
                    "operator `{}` expects Boolean operands, found {left} on the left",
                    self.op
                )));
            }
            if !right.is_boolean() {
                return Err(Error::type_mismatch(format!(
                    "operator `{}` expects Boolean operands, found {right} on the right",
                    self.op
                )));
            }
            return Ok(DataType::Boolean);
        }

        if self.op.is_comparison() {
            ensure_comparable(self.op, left, right)?;
            return Ok(DataType::Boolean);
        }

        if left == DataType::Null || right == DataType::Null {
            return Ok(DataType::Null);
        }

        if !left.is_numeric() {
            return Err(Error::type_mismatch(format!(
                "operator `{}` expects numeric operands, found {left} on the left",
                self.op
            )));
        }
        if !right.is_numeric() {
            return Err(Error::type_mismatch(format!(
                "operator `{}` expects numeric operands, found {right} on the right",
                self.op
            )));
        }

        Ok(promote_numeric(left, right))
    }
}

/// An expression with an output name, e.g. `age + 1 AS next_age`.
#[derive(Debug, Clone, PartialEq)]
pub struct Alias {
    pub expr: Box<Expr>,
    pub name: String,
}

impl Alias {
    pub fn new(expr: Expr, name: impl Into<String>) -> Self {
        Self {
            expr: Box::new(expr),
            name: name.into(),
        }
    }
}

/// The expression tree used by logical operators.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Column(Column),
    Literal(Literal),
    BinaryExpr(BinaryExpr),
    Alias(Alias),
}

impl Expr {
    pub fn col(name: impl Into<String>) -> Self {
        Self::Column(Column::new(name))
    }

    pub fn qualified_col(qualifier: impl Into<String>, name: impl Into<String>) -> Self {
        Self::Column(Column::qualified(qualifier, name))
    }

    pub fn lit(value: impl Into<Literal>) -> Self {
        Self::Literal(value.into())
    }

    pub fn binary(left: Expr, op: BinaryOperator, right: Expr) -> Self {
        Self::BinaryExpr(BinaryExpr::new(left, op, right))
    }

    pub fn alias(self, name: impl Into<String>) -> Self {
        Self::Alias(Alias::new(self, name))
    }

    pub fn eq_to(self, other: Expr) -> Self {
        Self::binary(self, BinaryOperator::Eq, other)
    }

    pub fn not_eq_to(self, other: Expr) -> Self {
        Self::binary(self, BinaryOperator::NotEq, other)
    }

    pub fn lt(self, other: Expr) -> Self {
        Self::binary(self, BinaryOperator::Lt, other)
    }

    pub fn lt_eq(self, other: Expr) -> Self {
        Self::binary(self, BinaryOperator::LtEq, other)
    }

    pub fn gt(self, other: Expr) -> Self {
        Self::binary(self, BinaryOperator::Gt, other)
    }

    pub fn gt_eq(self, other: Expr) -> Self {
        Self::binary(self, BinaryOperator::GtEq, other)
    }

    pub fn and(self, other: Expr) -> Self {
        Self::binary(self, BinaryOperator::And, other)
    }

    pub fn or(self, other: Expr) -> Self {
        Self::binary(self, BinaryOperator::Or, other)
    }

    pub fn plus(self, other: Expr) -> Self {
        Self::binary(self, BinaryOperator::Plus, other)
    }

    pub fn minus(self, other: Expr) -> Self {
        Self::binary(self, BinaryOperator::Minus, other)
    }

    pub fn multiply(self, other: Expr) -> Self {
        Self::binary(self, BinaryOperator::Multiply, other)
    }

    pub fn divide(self, other: Expr) -> Self {
        Self::binary(self, BinaryOperator::Divide, other)
    }

    pub fn modulo(self, other: Expr) -> Self {
        Self::binary(self, BinaryOperator::Modulo, other)
    }

    /// Infer the expression's result type against a schema.
    pub fn data_type(&self, schema: &Schema) -> Result<DataType> {
        match self {
            Self::Column(column) => {
                let index = schema.resolve(column.qualifier.as_deref(), &column.name)?;
                schema
                    .field(index)
                    .map(|field| field.data_type)
                    .ok_or_else(|| {
                        Error::invalid_schema(format!(
                            "resolved column `{}` has no field",
                            column.display_name()
                        ))
                    })
            }
            Self::Literal(literal) => Ok(literal.data_type()),
            Self::BinaryExpr(binary) => binary.data_type(schema),
            Self::Alias(alias) => alias.expr.data_type(schema),
        }
    }

    /// Infer whether the expression can produce `NULL`.
    pub fn nullable(&self, schema: &Schema) -> Result<bool> {
        match self {
            Self::Column(column) => {
                let index = schema.resolve(column.qualifier.as_deref(), &column.name)?;
                Ok(schema
                    .field(index)
                    .map(|field| field.nullable)
                    .unwrap_or(true))
            }
            Self::Literal(Literal::Null) => Ok(true),
            Self::Literal(_) => Ok(false),
            Self::BinaryExpr(binary) => {
                Ok(binary.left.nullable(schema)? || binary.right.nullable(schema)?)
            }
            Self::Alias(alias) => alias.expr.nullable(schema),
        }
    }

    /// The name this expression gives to a projected column.
    pub fn output_name(&self) -> String {
        match self {
            Self::Column(column) => column.name.clone(),
            Self::Literal(literal) => literal.to_string(),
            Self::BinaryExpr(_) => self.to_string(),
            Self::Alias(alias) => alias.name.clone(),
        }
    }

    /// Collect every column reference in the expression, in pre-order.
    pub fn column_refs(&self) -> Vec<&Column> {
        let mut references = Vec::new();
        self.foreach(|expr| {
            if let Self::Column(column) = expr {
                references.push(column);
            }
        });
        references
    }
}

impl From<Column> for Expr {
    fn from(value: Column) -> Self {
        Self::Column(value)
    }
}

impl From<Literal> for Expr {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Column(column) => write!(f, "{column}"),
            Self::Literal(literal) => write!(f, "{literal}"),
            Self::Alias(alias) => write!(f, "{} AS {}", alias.expr, alias.name),
            Self::BinaryExpr(binary) => {
                write_operand(f, &binary.left, binary.op, false)?;
                write!(f, " {} ", binary.op)?;
                write_operand(f, &binary.right, binary.op, true)
            }
        }
    }
}

impl TreeNode for Expr {
    fn node_name(&self) -> &'static str {
        match self {
            Self::Column(_) => "Column",
            Self::Literal(_) => "Literal",
            Self::BinaryExpr(_) => "BinaryExpr",
            Self::Alias(_) => "Alias",
        }
    }

    fn children(&self) -> Vec<&Self> {
        match self {
            Self::Column(_) | Self::Literal(_) => Vec::new(),
            Self::BinaryExpr(binary) => vec![binary.left.as_ref(), binary.right.as_ref()],
            Self::Alias(alias) => vec![alias.expr.as_ref()],
        }
    }

    fn with_new_children(&self, mut children: Vec<Self>) -> Result<Self> {
        let expected = self.arity();
        if children.len() != expected {
            return Err(invalid_arity(self.node_name(), expected, children.len()));
        }

        match self {
            Self::Column(_) | Self::Literal(_) => Ok(self.clone()),
            Self::BinaryExpr(binary) => {
                let left = children.remove(0);
                let right = children.remove(0);
                Ok(Self::BinaryExpr(BinaryExpr {
                    left: Box::new(left),
                    op: binary.op,
                    right: Box::new(right),
                }))
            }
            Self::Alias(alias) => {
                let expr = children.remove(0);
                Ok(Self::Alias(Alias {
                    expr: Box::new(expr),
                    name: alias.name.clone(),
                }))
            }
        }
    }
}

fn write_operand(
    f: &mut fmt::Formatter<'_>,
    expr: &Expr,
    parent: BinaryOperator,
    is_right: bool,
) -> fmt::Result {
    let needs_parentheses = match expr {
        Expr::BinaryExpr(child) => {
            let child_precedence = child.op.precedence();
            let parent_precedence = parent.precedence();
            child_precedence < parent_precedence
                || (is_right
                    && child_precedence == parent_precedence
                    && !(child.op == parent && child.op.is_associative()))
        }
        Expr::Alias(_) => true,
        _ => false,
    };

    if needs_parentheses {
        write!(f, "({expr})")
    } else {
        write!(f, "{expr}")
    }
}

fn ensure_comparable(op: BinaryOperator, left: DataType, right: DataType) -> Result<()> {
    if left == DataType::Null || right == DataType::Null {
        return Ok(());
    }
    if left == right {
        return Ok(());
    }
    if left.is_numeric() && right.is_numeric() {
        return Ok(());
    }
    Err(Error::type_mismatch(format!(
        "cannot compare {left} and {right} with `{op}`"
    )))
}

fn promote_numeric(left: DataType, right: DataType) -> DataType {
    match (left, right) {
        (DataType::Float64, _) | (_, DataType::Float64) => DataType::Float64,
        (DataType::Int64, _) | (_, DataType::Int64) => DataType::Int64,
        _ => DataType::Int32,
    }
}
