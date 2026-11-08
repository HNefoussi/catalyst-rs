# Catalyst-RS

> A miniature, Catalyst-inspired SQL query optimizer written in Rust.

Catalyst-RS is a learning-driven systems project that rebuilds the core ideas behind Apache Spark's Catalyst optimizer from scratch. Instead of wrapping an existing query engine, it models the pipeline explicitly: schemas, expressions, logical plans, tree transformations, rule-based optimization, physical planning, and eventually adaptive query execution.

The project is intentionally modular. Every layer has a small, explicit API so that new operators, expressions, rules, and execution strategies can be added without rewriting the engine.

**Phase 1 — Foundation and core abstractions is complete.**

```text
Project: [name, age]
    |
Filter: age > 25
    |
Scan: users
```

---

## Phase 1 scope

| Component | Status | Responsibility |
| --- | --- | --- |
| Rust workspace | ✅ | Modular crates for core, optimizer, and CLI |
| `Schema` | ✅ | Column names, qualifiers, data types, and nullability |
| `Expr` | ✅ | `Column`, `Literal`, `BinaryExpr`, and `Alias` |
| `LogicalPlan` | ✅ | `Scan`, `Filter`, and `Project` operator trees |
| Tree framework | ✅ | Traversal, collection, search, and top-down/bottom-up transforms |
| Plan printer | ✅ | Human-readable logical-plan trees |
| Tests | ✅ | Schema, expression, plan, tree, and optimizer-scaffold coverage |

Phase 1 deliberately does **not** include SQL parsing or concrete optimization rules. The demo plan is constructed with the Rust API; parsing and rules arrive in later phases.

---

## Quick start

```bash
cargo run -p catalyst-cli
```

Expected output:

```text
Catalyst-RS · Phase 1
=====================

Logical plan
------------
Project: [name, age]
    |
Filter: age > 25
    |
Scan: users

Debug tree
----------
Project: [name, age]
`- Filter: age > 25
   `- Scan: users

Output schema
-------------
[name: Utf8, age: Int32]

Programmatic inspection
-----------------------
Pre-order nodes : Project -> Filter -> Scan
Filter nodes    : 1
Scan nodes      : 1
Base table      : users

Expression demo
---------------
age + 1 AS next_age

Tree transformation demo (rename users -> people)
-------------------------------------------------
Project: [name, age]
    |
Filter: age > 25
    |
Scan: people

Optimizer scaffold
------------------
Rules registered : 0
Rules applied    : 0
Iterations       : 0
Next step        : add rules in Phase 2
```

---

## Workspace layout

```text
catalyst-rs/
├── crates/
│   ├── catalyst-core/       # schemas, expressions, logical plans, tree framework
│   ├── catalyst-optimizer/  # rule engine scaffold (concrete rules arrive in Phase 2)
│   └── catalyst-cli/        # runnable demo
├── Cargo.toml               # workspace manifest
└── README.md
```

### `catalyst-core`

The heart of the project:

- `schema.rs` — `DataType`, `Field`, `Schema`, and column resolution.
- `expr/` — expression tree, literals, operators, type inference, and printing.
- `plan/` — `LogicalPlan` plus the `Scan`, `Filter`, and `Project` operators.
- `tree.rs` — the generic `TreeNode` traversal and transformation framework.
- `format.rs` — the plan printer.
- `error.rs` — explicit, non-panicking error types.

### `catalyst-optimizer`

Defines the `Rule` trait and a fixpoint rule engine. No concrete rules are registered yet; Phase 2 will add Catalyst-inspired rules such as predicate pushdown, constant folding, projection pruning, and boolean simplification.

### `catalyst-cli`

A small runnable demo that builds a plan, prints it, inspects it programmatically, transforms it, and exercises the optimizer scaffold.

---

## Core abstractions

### Schema

A schema is an ordered list of typed fields. Fields can carry a table qualifier, which enables qualified references such as `users.age`.

```rust
use catalyst_core::{DataType, Field, Schema};

let schema = Schema::new(vec![
    Field::new("id", DataType::Int64).not_null(),
    Field::new("name", DataType::Utf8),
    Field::new("age", DataType::Int32),
]);

let age_index = schema.resolve(None, "age")?;
```

Resolution is deliberately strict:

- missing columns produce `Error::ColumnNotFound`;
- duplicate names produce `Error::AmbiguousColumn`;
- qualified references prefer an exact qualifier match.

### Expressions

Expressions form their own tree and support type inference against a schema.

```rust
use catalyst_core::Expr;

let predicate = Expr::col("age").gt(Expr::lit(25));
assert_eq!(predicate.to_string(), "age > 25");

let projection = Expr::col("age").plus(Expr::lit(1)).alias("next_age");
assert_eq!(projection.to_string(), "age + 1 AS next_age");
```

The printer is precedence-aware, so nested expressions keep their meaning:

```text
(age + 1) * 2
a - (b - c)
a + b + c
(a OR b) AND c
```

### Logical plans

A logical plan is a tree of relational operators. The builder API keeps construction readable:

```rust
use catalyst_core::{Expr, LogicalPlan, Schema};

let plan = LogicalPlan::scan("users", schema)
    .filter(Expr::col("age").gt(Expr::lit(25)))
    .project(vec![Expr::col("name"), Expr::col("age")]);

assert_eq!(
    plan.to_string(),
    "Project: [name, age]\n    |\nFilter: age > 25\n    |\nScan: users"
);
```

Schemas are derived from the plan, not stored redundantly:

- `Scan` exposes its table schema.
- `Filter` preserves its input schema and validates that the predicate is boolean.
- `Project` derives a new schema from its expressions and aliases.

```rust
let output = plan.schema()?;
assert_eq!(output.to_string(), "[name: Utf8, age: Int32]");
```

### Tree framework

Both `Expr` and `LogicalPlan` implement the same `TreeNode` trait. That means traversal and rewriting code is written once and reused across both trees.

```rust
use catalyst_core::{LogicalPlan, TreeNode};

// Pre-order traversal.
plan.foreach(|node| println!("{}", node.node_name()));

// Collect matching nodes.
let filters = plan.collect(|node| matches!(node, LogicalPlan::Filter(_)));

// Find the first matching node.
let scan = plan.find(|node| matches!(node, LogicalPlan::Scan(_)));

// Bottom-up rewrite: every node is visited after its children.
let rewritten = plan.transform_up(|node| match node {
    LogicalPlan::Scan(scan) => Ok(LogicalPlan::scan("people", scan.schema.clone())),
    other => Ok(other),
})?;

// Top-down rewrite: the parent is visited before its children.
let rewritten = plan.transform_down(|node| Ok(node))?;
```

The trait also provides `map_children`, `exists`, and `arity`. A node only needs to implement three things:

```rust
trait TreeNode: Sized + Clone {
    fn node_name(&self) -> &'static str;
    fn children(&self) -> Vec<&Self>;
    fn with_new_children(&self, children: Vec<Self>) -> Result<Self>;
}
```

---

## Extending the engine

### Adding a logical operator

1. Add a node struct under `crates/catalyst-core/src/plan/`.
2. Add a variant to `LogicalPlan`.
3. Implement its `children` and `with_new_children` arms in `plan/mod.rs`.
4. Add its schema derivation and display header.
5. Re-export it from `catalyst-core/src/lib.rs`.

Because the tree framework is generic, traversal and transformation support come for free.

### Adding an expression

1. Add a variant to `Expr`.
2. Handle it in `children`, `with_new_children`, `data_type`, `nullable`, and `output_name`.
3. Add its display formatting.

### Adding an optimization rule (Phase 2)

Rules are ordinary values that implement `Rule`:

```rust
use catalyst_core::{LogicalPlan, Result, TreeNode};
use catalyst_optimizer::{Optimizer, Rule};

struct RenameScanRule;

impl Rule for RenameScanRule {
    fn name(&self) -> &'static str {
        "RenameScan"
    }

    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        plan.transform_up(|node| match node {
            LogicalPlan::Scan(scan) => Ok(LogicalPlan::scan("people", scan.schema.clone())),
            other => Ok(other),
        })
    }
}

let optimizer = Optimizer::new().with_rule(RenameScanRule);
let result = optimizer.optimize(plan)?;

println!("applied: {:?}", result.applied);
println!("{}", result.plan);
```

The optimizer applies every rule in order and repeats the batch until the plan reaches a fixed point.

---

## Roadmap

- [x] **Phase 1 — Foundation and core abstractions**
  - Workspace, schema, expressions, logical plans, tree framework, plan printer, tests.
- [ ] **Phase 2 — SQL parsing and analysis**
  - Parse SQL into an AST, build logical plans, resolve columns, and validate types.
- [ ] **Phase 3 — Rule-based optimization**
  - Predicate pushdown, constant folding, projection pruning, boolean simplification, and rule-batch explain output.
- [ ] **Phase 4 — Physical planning**
  - Convert optimized logical plans into physical operators and cost-aware strategies.
- [ ] **Phase 5 — Query execution**
  - A small iterator-based execution engine for scans, filters, projections, joins, and aggregates.
- [ ] **Phase 6 — Adaptive Query Execution**
  - Runtime statistics, plan re-optimization, and adaptive shuffle/join strategies.

---

## Testing

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

The test suite covers schema resolution, expression type inference and printing, plan schema derivation, plan formatting, tree traversal, tree transformations, arity validation, and the optimizer fixpoint loop.

---

## Design principles

- **No external dependencies in the core.** Phase 1 is pure `std` Rust, which keeps the data structures easy to inspect and learn from.
- **Immutable plans.** Transformations return new trees, which makes rules predictable and testable.
- **Explicit errors.** Library code returns `Result` instead of panicking.
- **Extensible by construction.** Operators, expressions, and rules each have a single, documented extension point.
- **Readable output.** Plans are printed in a compact form that is easy to screenshot, diff, and explain.

---

## License

MIT
