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
  root: usize,
  active_leaf: usize
}

impl SuffixTree {
  /// Creates new suffix tree.
  pub fn new() -> Self {
    let node = Node::new();
    let tree_nodes = vec![node];
    Self {
      tree_nodes: tree_nodes,
      root: 0,
      active_leaf: 0
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

  /// Adds the specified `index` to the GST under the given `key`.
  pub fn put(&mut self, key: &[u8], index: usize) {
    self.active_leaf = self.root;
    let mut s = self.root;

    let mut text = Vec::new();
    for i in 0..key.len() {
      text.push(key[i]);
      let node_id = self.update(s, &mut text, &key[i..], index);
      s = self.canonize(node_id, &mut text);
    }

    if !self.get_node(self.active_leaf).has_suffix() &&
        self.active_leaf != self.root && self.active_leaf != s {
      let active_leaf_id = self.active_leaf;
      self.get_node_mut(active_leaf_id).set_suffix(s);
    }
  }

  /// Returns reference for a node.
  fn get_node(&self, node_id: usize) -> &Node {
    &self.tree_nodes[node_id]
  }

  /// Returns mutable reference for a node.
  fn get_node_mut(&mut self, node_id: usize) -> &mut Node {
    self.tree_nodes.get_mut(node_id).expect("No node found")
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

  /// Updates the tree starting from inputNode and by adding stringPart.
  ///
  /// Returns a node_id for the string that has been added so far.
  /// This means:
  /// - the Node will be the Node that can be reached by the longest path string (S1)
  ///   that can be obtained by concatenating consecutive edges in the tree and
  ///   that is a substring of the string added so far to the tree.
  /// - the String will be the remainder that must be added to S1 to get the string
  ///   added so far.
  fn update(&self, s: usize, part: &mut Vec<u8>, rest: &[u8], index: usize) -> usize {
    unimplemented!();
  }

  /// Return a node_id (n) such that n is a farthest descendant of s (the input node)
  /// that can be reached by following a path of edges denoting a prefix of inputstr and
  /// remainder will be string that must be appended to the concatenation of labels from
  /// s to n to get inpustr.
  fn canonize(&self, s: usize, input: &mut Vec<u8>) -> usize {
    let len = input.len();
    if len == 0 {
      return s;
    }
    let mut curr_node_id = s;
    let mut idx = 0;
    let mut g = self.get_node(curr_node_id).get_edge(input[idx]);
    // Descend the tree as long as a proper label is found
    while let Some(edge) = g {
      let label = edge.get_label();
      if len - idx >= label.len() && &input[idx..idx + label.len()] == &label[..] {
        idx += label.len();
        curr_node_id = edge.get_dest_id();
        if idx < input.len() {
          g = self.get_node(curr_node_id).get_edge(input[idx]);
        }
      } else {
        break;
      }
    }

    // Update the input to the latest slice.
    if idx > 0 {
      for i in idx..len {
        input[i - idx] = input[i];
      }
      input.resize(len - idx, 0);
    }

    curr_node_id
  }
}
