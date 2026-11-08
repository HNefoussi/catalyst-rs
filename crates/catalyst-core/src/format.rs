use crate::{LogicalPlan, TreeNode};

/// Format a plan as the compact vertical chain used in the README and CLI.
///
/// For unary plans this produces:
///
/// ```text
/// Project: [name, age]
///     |
/// Filter: age > 25
///     |
/// Scan: users
/// ```
///
/// Plans with branching (for example a future `Join`) fall back to a box tree.
pub fn format_plan(plan: &LogicalPlan) -> String {
    let mut output = String::new();
    if is_chain(plan) {
        write_chain(plan, &mut output);
    } else {
        write_box(plan, "", true, true, &mut output);
    }
    output.trim_end().to_string()
}

/// Format a plan as an indented box tree. Useful for debugging arbitrary plans.
pub fn format_box_tree(plan: &LogicalPlan) -> String {
    let mut output = String::new();
    write_box(plan, "", true, true, &mut output);
    output.trim_end().to_string()
}

fn is_chain(plan: &LogicalPlan) -> bool {
    let mut current = plan;
    loop {
        let children = current.children();
        match children.len() {
            0 => return true,
            1 => current = children[0],
            _ => return false,
        }
    }
}

fn write_chain(plan: &LogicalPlan, output: &mut String) {
    let mut current = plan;
    loop {
        output.push_str(&current.display_header());
        output.push('\n');

        let children = current.children();
        if children.len() == 1 {
            output.push_str("    |\n");
            current = children[0];
        } else {
            break;
        }
    }
}

fn write_box(node: &LogicalPlan, prefix: &str, is_last: bool, is_root: bool, output: &mut String) {
    if is_root {
        output.push_str(&node.display_header());
        output.push('\n');
    } else {
        output.push_str(prefix);
        output.push_str(if is_last { "`- " } else { "+- " });
        output.push_str(&node.display_header());
        output.push('\n');
    }

    let children = node.children();
    let next_prefix = if is_root {
        String::new()
    } else {
        format!("{}{}", prefix, if is_last { "   " } else { "|  " })
    };

    for (index, child) in children.iter().enumerate() {
        write_box(
            child,
            &next_prefix,
            index + 1 == children.len(),
            false,
            output,
        );
    }
}
