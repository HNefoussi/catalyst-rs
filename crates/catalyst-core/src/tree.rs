use crate::{Error, Result};

/// A generic tree abstraction shared by expressions and logical plans.
///
/// A node only has to describe its children and how to rebuild itself with a
/// new set of children. Everything else (traversal, collection, search, and
/// transformations) is provided by default methods.
///
/// This mirrors the role of Catalyst's `TreeNode` in Spark: optimization rules
/// can be written once against the traversal API and reused across plan and
/// expression trees.
pub trait TreeNode: Sized + Clone {
    /// A short name used in diagnostics, e.g. `Filter` or `BinaryExpr`.
    fn node_name(&self) -> &'static str;

    /// Borrow the direct children of this node.
    fn children(&self) -> Vec<&Self>;

    /// Rebuild this node with new children.
    fn with_new_children(&self, children: Vec<Self>) -> Result<Self>;

    fn arity(&self) -> usize {
        self.children().len()
    }

    /// Visit every node in pre-order (parent before children).
    fn foreach<'a, F>(&'a self, mut f: F)
    where
        F: FnMut(&'a Self),
    {
        self.foreach_impl(&mut f);
    }

    #[doc(hidden)]
    fn foreach_impl<'a, F>(&'a self, f: &mut F)
    where
        F: FnMut(&'a Self),
    {
        f(self);
        for child in self.children() {
            child.foreach_impl(&mut *f);
        }
    }

    /// Collect every node for which `f` returns `true`, in pre-order.
    fn collect<F>(&self, mut f: F) -> Vec<Self>
    where
        F: FnMut(&Self) -> bool,
    {
        let mut result = Vec::new();
        self.collect_impl(&mut f, &mut result);
        result
    }

    #[doc(hidden)]
    fn collect_impl<F>(&self, f: &mut F, result: &mut Vec<Self>)
    where
        F: FnMut(&Self) -> bool,
    {
        if f(self) {
            result.push(self.clone());
        }
        for child in self.children() {
            child.collect_impl(&mut *f, result);
        }
    }

    /// Return the first node matching `f`, in pre-order.
    fn find<F>(&self, f: F) -> Option<&Self>
    where
        F: Fn(&Self) -> bool,
    {
        self.find_impl(&f)
    }

    #[doc(hidden)]
    fn find_impl<F>(&self, f: &F) -> Option<&Self>
    where
        F: Fn(&Self) -> bool,
    {
        if f(self) {
            return Some(self);
        }
        for child in self.children() {
            if let Some(found) = child.find_impl(f) {
                return Some(found);
            }
        }
        None
    }

    fn exists<F>(&self, f: F) -> bool
    where
        F: Fn(&Self) -> bool,
    {
        self.find(f).is_some()
    }

    /// Apply `f` to each direct child and rebuild the node.
    fn map_children<F>(&self, mut f: F) -> Result<Self>
    where
        F: FnMut(Self) -> Result<Self>,
    {
        let mut new_children = Vec::with_capacity(self.arity());
        for child in self.children() {
            new_children.push(f(child.clone())?);
        }
        self.with_new_children(new_children)
    }

    /// Top-down transformation: apply `f` to a node, then recurse into the
    /// children of the result.
    fn transform_down<F>(&self, mut f: F) -> Result<Self>
    where
        F: FnMut(Self) -> Result<Self>,
    {
        self.transform_down_impl(&mut f)
    }

    #[doc(hidden)]
    fn transform_down_impl<F>(&self, f: &mut F) -> Result<Self>
    where
        F: FnMut(Self) -> Result<Self>,
    {
        let node = f(self.clone())?;
        let children: Vec<Self> = node.children().into_iter().cloned().collect();
        let mut new_children = Vec::with_capacity(children.len());
        for child in children {
            new_children.push(child.transform_down_impl(&mut *f)?);
        }
        node.with_new_children(new_children)
    }

    /// Bottom-up transformation: recurse into children first, then apply `f`
    /// to the rebuilt node.
    fn transform_up<F>(&self, mut f: F) -> Result<Self>
    where
        F: FnMut(Self) -> Result<Self>,
    {
        self.transform_up_impl(&mut f)
    }

    #[doc(hidden)]
    fn transform_up_impl<F>(&self, f: &mut F) -> Result<Self>
    where
        F: FnMut(Self) -> Result<Self>,
    {
        let children: Vec<Self> = self.children().into_iter().cloned().collect();
        let mut new_children = Vec::with_capacity(children.len());
        for child in children {
            new_children.push(child.transform_up_impl(&mut *f)?);
        }
        let node = self.with_new_children(new_children)?;
        f(node)
    }
}

/// Helper for nodes that received the wrong number of children.
pub(crate) fn invalid_arity(node: &'static str, expected: usize, actual: usize) -> Error {
    Error::InvalidArity {
        node,
        expected,
        actual,
    }
}
