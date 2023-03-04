/// A simple trait defining behaviour that pertains to nodes in any tree-like structure.
pub trait Node<T: Node<T> + PartialEq> {
    /// Retrieve the parent of a node, if it exists.
    fn parent(&self) -> Option<T>;

    /// Retrieve the children of a node.
    fn children(&self) -> Vec<T>;

    /// Append a new node to the end of the node.
    fn append(&mut self, child: &T);

    /// Prepend a new node at the start of the node.
    fn prepend(&mut self, child: &T);

    /// Disconnect a node from its parent.
    /// If the node is not a child of the current node then nothing happens.
    fn remove(&mut self, node: &T);

    /// Check recursively if the current node is the ancestor of another.
    fn contains(&self, node: &T) -> bool {
        let mut is_contained = false;
        for element in &self.children() {
            if element == node {
                is_contained = true;
                break;
            } else {
                is_contained = element.contains(node);
                if is_contained {
                    break;
                }
            }
        }
        is_contained
    }
    /// Insert node directly after itself.
    /// # Panics
    /// Panics if only one node is allowed.
    fn after(&mut self, node: &T);
}
