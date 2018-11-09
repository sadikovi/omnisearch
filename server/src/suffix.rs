use std::cmp;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Represents an Edge in the Suffix Tree.
/// It has a label and a destination Node.
#[derive(Clone, Debug)]
pub struct Edge {
  label: String,
  dest: Rc<Node>
}

impl Edge {
  /// Creates a new edge.
  pub fn new(label: String, dest: Rc<Node>) -> Self {
    Self { label, dest }
  }

  /// Sets label for the edge.
  pub fn set_label(&mut self, label: String) {
    self.label = label;
  }

  /// Sets destination node for the edge.
  pub fn set_dest(&mut self, node: Rc<Node>) {
    self.dest = node;
  }

  /// Returns edge label.
  pub fn get_label(&self) -> &str {
    &self.label
  }

  /// Returns reference counted for destination node.
  pub fn get_node(&self) -> Rc<Node> {
    self.dest.clone()
  }
}

/// Represents a node of the generalized suffix tree graph.
#[derive(Clone, Debug)]
pub struct Node {
  data: HashSet<usize>,
  suffix: Option<Rc<Node>>,
  edges: HashMap<char, Edge>
}

impl Node {
  /// Creates a new node.
  pub fn new() -> Self {
    Self {
      // Data contains only unique elements in increasing order.
      data: HashSet::new(),
      suffix: None,
      edges: HashMap::new()
    }
  }

  /// Returns the first `num_results` elements from the ones associated to this node.
  ///
  /// Gets data from the payload of both this node and its children, the string
  /// representation of the path to this node is a substring of the one of the children
  /// nodes.
  pub fn get_data(&self, num_results: usize) -> HashSet<usize> {
    let mut res = HashSet::with_capacity(num_results);
    self.get_data_recur(num_results, &mut res);
    res
  }

  /// Get data recursively, first inspecting the node and then its children.
  fn get_data_recur(&self, num_results: usize, res: &mut HashSet<usize>) {
    let mut num_to_take = cmp::min(num_results - res.len(), self.data.len());
    let mut iter = self.data.iter();

    while let Some(&value) = iter.next() {
      if num_to_take == 0 {
        break;
      }

      if res.insert(value) {
        num_to_take -= 1;
      }
    }

    for edge in self.edges.values() {
      if num_results > res.len() {
        edge.get_node().get_data_recur(num_results, res);
      } else {
        break;
      }
    }
  }

  /// Add data index to the node.
  pub fn add_index(&mut self, index: usize) {
    self.data.insert(index);
  }

  /// Adds edge for the key.
  pub fn add_edge(&mut self, key: char, value: Edge) {
    self.edges.insert(key, value);
  }

  /// Sets suffix for the node.
  pub fn set_suffix(&mut self, suffix: Rc<Node>) {
    self.suffix = Some(suffix);
  }

  /// Returns edge for the key, or None if no such key exists.
  pub fn get_edge(&self, key: char) -> Option<&Edge> {
    self.edges.get(&key)
  }

  /// Returns suffix of the node.
  pub fn get_suffix(&self) -> Option<Rc<Node>> {
    self.suffix.clone()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn node_with_values(mut values: Vec<usize>) -> Node {
    let mut node = Node::new();
    while let Some(elem) = values.pop() {
      node.add_index(elem);
    }
    node
  }

  #[test]
  fn test_node_get_data() {
    let mut node = Node::new();
    for i in 0..10 {
      node.add_index(i);
    }
    for i in 0..10 {
      assert_eq!(node.get_data(i).len(), i);
    }
    assert_eq!(node.get_data(100).len(), 10);
  }

  #[test]
  fn test_node_get_data_with_edges() {
    let mut node = Node::new();
    node.add_edge('a', Edge::new("a".to_owned(), Rc::new(node_with_values(vec![0, 4]))));
    node.add_edge('b', Edge::new("b".to_owned(), Rc::new(node_with_values(vec![0, 5]))));
    node.add_edge('c', Edge::new("c".to_owned(), Rc::new(node_with_values(vec![0, 6]))));
    node.add_edge('d', Edge::new("d".to_owned(), Rc::new(node_with_values(vec![4, 5]))));
    for i in 0..3 {
      node.add_index(i);
    }

    for i in 0..10 {
      assert_eq!(node.get_data(i).len(), cmp::min(i, 6));
    }
    assert_eq!(node.get_data(100).len(), 6);
  }
}
