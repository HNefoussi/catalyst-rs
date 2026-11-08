mod filter;
mod project;
mod scan;

pub use filter::Filter;
pub use project::Project;
pub use scan::Scan;

use std::fmt;

use crate::tree::invalid_arity;
use crate::{DataType, Error, Expr, Field, Result, Schema, TreeNode};

/// A relational operator tree.
///
/// Phase 1 covers the three operators needed to express a simple
/// `SELECT ... FROM ... WHERE ...` query. New operators (Join, Aggregate, ...)
/// can be added as variants without changing the tree framework.
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalPlan {
    Scan(Scan),
    Filter(Filter),
    Project(Project),
}

impl LogicalPlan {
    pub fn scan(table_name: impl Into<String>, schema: Schema) -> Self {
        Self::Scan(Scan::new(table_name, schema))
    }

    /// Wrap this plan in a filter.
    pub fn filter(self, predicate: Expr) -> Self {
        Self::Filter(Filter::new(self, predicate))
    }

    /// Wrap this plan in a projection.
    pub fn project(self, exprs: Vec<Expr>) -> Self {
        Self::Project(Project::new(self, exprs))
    }

    pub fn input(&self) -> Option<&LogicalPlan> {
        match self {
            Self::Scan(_) => None,
            Self::Filter(filter) => Some(filter.input.as_ref()),
            Self::Project(project) => Some(project.input.as_ref()),
        }
    }

    /// Derive and validate the output schema of this plan.
    pub fn schema(&self) -> Result<Schema> {
        match self {
            Self::Scan(scan) => Ok(scan.schema.clone()),
            Self::Filter(filter) => {
                let input_schema = filter.input.schema()?;
                let predicate_type = filter.predicate.data_type(&input_schema)?;
                if predicate_type != DataType::Boolean {
                    return Err(Error::type_mismatch(format!(
                        "filter predicate must be Boolean, found {predicate_type}"
                    )));
                }
                Ok(input_schema)
            }
            Self::Project(project) => {
                let input_schema = project.input.schema()?;
                let fields = project
                    .exprs
                    .iter()
                    .map(|expr| {
                        let data_type = expr.data_type(&input_schema)?;
                        let nullable = expr.nullable(&input_schema)?;
                        Ok(Field {
                            qualifier: None,
                            name: expr.output_name(),
                            data_type,
                            nullable,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(Schema::new(fields))
            }
        }
    }

    /// Validate the whole plan by deriving its schema.
    pub fn validate(&self) -> Result<()> {
        self.schema().map(|_| ())
    }

    /// The first line of this node when printed in a plan tree.
    pub fn display_header(&self) -> String {
        match self {
            Self::Scan(scan) => format!("Scan: {}", scan.table_name),
            Self::Filter(filter) => format!("Filter: {}", filter.predicate),
            Self::Project(project) => {
                let exprs = project
                    .exprs
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Project: [{exprs}]")
            }
        }
    }

    /// Render the plan as an indented box tree.
    pub fn tree_string(&self) -> String {
        crate::format::format_box_tree(self)
    }
}

impl fmt::Display for LogicalPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::format::format_plan(self))
    }
}

impl TreeNode for LogicalPlan {
    fn node_name(&self) -> &'static str {
        match self {
            Self::Scan(_) => "Scan",
            Self::Filter(_) => "Filter",
            Self::Project(_) => "Project",
        }
    }

    fn children(&self) -> Vec<&Self> {
        match self {
            Self::Scan(_) => Vec::new(),
            Self::Filter(filter) => vec![filter.input.as_ref()],
            Self::Project(project) => vec![project.input.as_ref()],
        }
    }

    fn with_new_children(&self, mut children: Vec<Self>) -> Result<Self> {
        let expected = self.arity();
        if children.len() != expected {
            return Err(invalid_arity(self.node_name(), expected, children.len()));
        }

        match self {
            Self::Scan(_) => Ok(self.clone()),
            Self::Filter(filter) => Ok(Self::Filter(Filter {
                predicate: filter.predicate.clone(),
                input: Box::new(children.remove(0)),
            })),
            Self::Project(project) => Ok(Self::Project(Project {
                exprs: project.exprs.clone(),
                input: Box::new(children.remove(0)),
            })),
        }
    }
}
