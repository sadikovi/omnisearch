use std::cmp;
use std::collections::HashSet;

/// Represents an Edge in the Suffix Tree.
/// It has a label and a destination node.
#[derive(Clone, Debug)]
pub struct Edge {
  label: Vec<u8>,
  dest_node_id: usize
}

impl Edge {
  /// Creates a new edge.
  pub fn new(label: Vec<u8>, dest_node_id: usize) -> Self {
    Self { label, dest_node_id }
  }

  /// Sets label for the edge.
  pub fn set_label(&mut self, label: Vec<u8>) {
    self.label = label;
  }

  /// Sets label from slice.
  pub fn set_label_from_slice(&mut self, label: &[u8]) {
    self.label = label.to_vec();
  }

  /// Returns destination node id.
  pub fn get_dest_id(&self) -> usize {
    self.dest_node_id
  }

  /// Returns edge label.
  pub fn get_label(&self) -> &[u8] {
    &self.label
  }

  /// Appends byte to the label.
  pub fn add_byte(&mut self, elem: u8) {
    self.label.push(elem);
  }
}

/// Represents a node of the generalized suffix tree graph.
#[derive(Clone, Debug)]
pub struct Node {
  indices: HashSet<usize>,
  suffix: Option<usize>,
  edges: Vec<Option<Edge>>
}

impl Node {
  /// Creates a new node.
  pub fn new() -> Self {
    Self {
      indices: HashSet::new(),
      suffix: None,
      edges: vec![None; u8::max_value() as usize + 1]
    }
  }

  /// Add data index to the node.
  pub fn add_index(&mut self, index: usize) {
    self.indices.insert(index);
  }

  /// Adds edge for the key.
  pub fn add_edge(&mut self, key: u8, value: Edge) {
    self.edges[key as usize] = Some(value);
  }

  /// Sets suffix for the node.
  pub fn set_suffix(&mut self, suffix: usize) {
    self.suffix = Some(suffix);
  }

  /// Returns edge for the key, or None if no such key exists.
  pub fn get_edge(&self, key: u8) -> Option<&Edge> {
    self.edges[key as usize].as_ref()
  }

  /// Returns mutable reference for edge, or None if no such key exists.
  pub fn get_edge_mut(&mut self, key: u8) -> Option<&mut Edge> {
    self.edges[key as usize].as_mut()
  }

  /// Returns list of all edges in the node, including None.
  pub fn get_edges(&self) -> &[Option<Edge>] {
    &self.edges[..]
  }

  /// Returns reference to indices for the node.
  pub fn get_indices(&self) -> &HashSet<usize> {
    &self.indices
  }

  /// Returns suffix of the node.
  pub fn get_suffix(&self) -> Option<usize> {
    self.suffix
  }

  /// Returns true if node has suffix.
  pub fn has_suffix(&self) -> bool {
    self.suffix.is_some()
  }
}

pub struct SuffixTree {
  tree_nodes: Vec<Node>,
  root: usize
}

impl SuffixTree {
  /// Creates new suffix tree.
  pub fn new() -> Self {
    let node = Node::new();
    let tree_nodes = vec![node];
    Self {
      tree_nodes: tree_nodes,
      root: 0
    }
  }

  /// Searches for the given word within the GST and returns at most the given number of
  /// matches.
  pub fn search(&self, word: &[u8], num_results: usize) -> HashSet<usize> {
    let mut res = HashSet::new();
    if let Some(node) = self.search_node(word) {
      self.get_data_recur(node, num_results, &mut res);
    }
    res
  }

  /// Returns reference for a node.
  fn get_node(&self, node_id: usize) -> &Node {
    &self.tree_nodes[node_id]
  }

  /// Returns the tree node (if present) that corresponds to the given string.
  fn search_node(&self, word: &[u8]) -> Option<&Node> {
    let mut curr_node = self.root;
    let mut i = 0;
    while i < word.len() {
      if let Some(edge) = self.get_node(curr_node).get_edge(word[i]) {
        let label = edge.get_label();
        let len_to_match = cmp::min(label.len(), word.len() - i);

        if &word[i..i + len_to_match] != &label[0..len_to_match] {
          return None;
        }

        if label.len() >= word.len() - i {
          return Some(self.get_node(edge.get_dest_id()));
        }

        curr_node = edge.get_dest_id();
        i += len_to_match - 1;
      } else {
        return None;
      }
    }

    None
  }

  /// Get data recursively, first inspecting the node and then its children.
  fn get_data_recur(&self, node: &Node, num_results: usize, res: &mut HashSet<usize>) {
    let mut num_to_take = cmp::min(num_results - res.len(), node.get_indices().len());
    let mut iter = node.get_indices().iter();

    while let Some(&value) = iter.next() {
      if num_to_take == 0 {
        break;
      }

      if res.insert(value) {
        num_to_take -= 1;
      }
    }

    for edge in node.get_edges() {
      if let Some(edge) = edge {
        if num_results > res.len() {
          self.get_data_recur(self.get_node(edge.get_dest_id()), num_results, res);
        } else {
          break;
        }
      }
    }
  }
}
