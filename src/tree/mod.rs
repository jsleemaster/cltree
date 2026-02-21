mod file_node;

pub use file_node::FileNode;

use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub struct FileTree {
    root: PathBuf,
    nodes: Vec<FileNode>,
    pub show_hidden: bool,
    max_depth: usize,
    offset: usize,
}

impl FileTree {
    pub fn new(root: &Path, show_hidden: bool, max_depth: usize) -> Result<Self> {
        let mut tree = Self {
            root: root.to_path_buf(),
            nodes: Vec::new(),
            show_hidden,
            max_depth,
            offset: 0,
        };

        tree.rebuild_visible_nodes()?;

        Ok(tree)
    }

    pub fn root_path(&self) -> &Path {
        &self.root
    }

    pub fn nodes(&self) -> &[FileNode] {
        &self.nodes
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    fn rebuild_visible_nodes(&mut self) -> Result<()> {
        self.nodes.clear();
        self.build_tree(&self.root.clone(), 0, &[])?;
        Ok(())
    }

    fn build_tree(&mut self, path: &Path, depth: usize, connector: &[bool]) -> Result<()> {
        if depth > self.max_depth {
            return Ok(());
        }

        if depth == 0 {
            // Root node
            let is_dir = path.is_dir();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string());

            let node = FileNode::new(path.to_path_buf(), name, 0, is_dir, true, vec![]);
            self.nodes.push(node);

            if is_dir {
                self.build_tree(path, depth + 1, &[])?;
            }
            return Ok(());
        }

        let walker = WalkBuilder::new(path)
            .hidden(!self.show_hidden)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .max_depth(Some(1))
            .build();

        // Collect children (skip the directory itself)
        let mut children: Vec<_> = walker
            .flatten()
            .filter(|entry| entry.path() != path)
            .filter(|entry| {
                if self.show_hidden {
                    return true;
                }
                let name = entry.file_name().to_string_lossy();
                !name.starts_with('.')
            })
            .collect();

        children.sort_by(|a, b| {
            let a_is_dir = a.path().is_dir();
            let b_is_dir = b.path().is_dir();
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => {
                    let a_name = a.file_name().to_string_lossy().to_lowercase();
                    let b_name = b.file_name().to_string_lossy().to_lowercase();
                    a_name.cmp(&b_name)
                }
            }
        });

        let total = children.len();
        for (i, entry) in children.into_iter().enumerate() {
            let entry_path = entry.path().to_path_buf();
            let is_dir = entry_path.is_dir();
            let name = entry_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| entry_path.to_string_lossy().to_string());

            let is_last = i == total - 1;

            let node = FileNode::new(
                entry_path.clone(),
                name,
                depth,
                is_dir,
                is_last,
                connector.to_vec(),
            );
            self.nodes.push(node);

            // Recurse into directories with updated connector
            if is_dir {
                let mut child_connector = connector.to_vec();
                child_connector.push(is_last);
                self.build_tree(&entry_path, depth + 1, &child_connector)?;
            }
        }

        Ok(())
    }

    pub fn set_root(&mut self, new_root: PathBuf) {
        self.root = new_root;
        self.offset = 0;
        let _ = self.rebuild_visible_nodes();
    }

    pub fn refresh(&mut self) {
        let _ = self.rebuild_visible_nodes();
    }
}
