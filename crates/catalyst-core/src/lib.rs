//! # Catalyst-RS Core
//!
//! Core abstractions for a miniature, Catalyst-inspired SQL optimizer:
//!
//! - [`Schema`] and [`DataType`] describe the shape of relational data.
//! - [`Expr`] models SQL expressions as a small, extensible tree.
//! - [`LogicalPlan`] models relational operators as a tree of plan nodes.
//! - [`TreeNode`] provides generic traversal and transformation over both
//!   expression trees and logical plan trees.
//!
//! The crate intentionally has no external dependencies. It is the foundation
//! that later phases (parsing, rule-based optimization, physical planning, and
//! execution) build on.

pub mod error;
pub mod expr;
pub mod format;
pub mod plan;
pub mod schema;
pub mod tree;

pub use error::{Error, Result};
pub use expr::{Alias, BinaryExpr, BinaryOperator, Column, Expr, Literal};
pub use plan::{Filter, LogicalPlan, Project, Scan};
pub use schema::{DataType, Field, Schema};
pub use tree::TreeNode;
