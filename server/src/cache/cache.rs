// File tree cache, used to store paths with the ability to traverse and update
// individual directories/files.

// Tree node
#[derive(Clone, Debug)]
pub enum TreeNode<'a> {
  Dirs(Vec<String>, Vec<TreeNode<'a>>),
  File(String, Option<&'a TreeNode<'a>>)
}

impl<'a> TreeNode<'a> {
  pub fn file(name: String, previous: Option<&'a TreeNode>) -> Self {
    if let Some(prev) = previous {
      assert!(prev.is_file());
    }
    TreeNode::File(name, previous)
  }

  // Is node a file?
  pub fn is_file(&self) -> bool {
    match self {
      TreeNode::Dirs(_, _) => false,
      TreeNode::File(_, _) => true
    }
  }

  // Is node a directory?
  pub fn is_dir(&self) -> bool {
    match self {
      TreeNode::Dirs(_, _) => true,
      TreeNode::File(_, _) => false
    }
  }
}
