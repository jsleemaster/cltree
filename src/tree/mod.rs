mod file_node;

pub use file_node::FileNode;

use anyhow::Result;
use ignore::WalkBuilder;
use std::collections::HashMap;
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
        self.build_tree()?;
        Ok(())
    }

    fn build_tree(&mut self) -> Result<()> {
        let root = self.root.clone();

        // Single WalkBuilder traversal for the entire tree
        let walker = WalkBuilder::new(&root)
            .hidden(!self.show_hidden)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .max_depth(Some(self.max_depth))
            .build();

        // Collect entries grouped by parent directory
        let mut children_map: HashMap<PathBuf, Vec<(PathBuf, bool)>> = HashMap::new();

        for entry in walker.flatten() {
            let entry_path = entry.path().to_path_buf();
            let is_dir = entry.file_type().is_some_and(|ft| ft.is_dir());

            // Skip the root directory itself
            if entry_path == root {
                continue;
            }

            if let Some(parent) = entry_path.parent() {
                children_map
                    .entry(parent.to_path_buf())
                    .or_default()
                    .push((entry_path, is_dir));
            }
        }

        // Sort each group: directories first, then case-insensitive alphabetical
        for children in children_map.values_mut() {
            children.sort_by(|(a_path, a_is_dir), (b_path, b_is_dir)| {
                match (a_is_dir, b_is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => {
                        let a_name = a_path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_lowercase())
                            .unwrap_or_default();
                        let b_name = b_path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_lowercase())
                            .unwrap_or_default();
                        a_name.cmp(&b_name)
                    }
                }
            });
        }

        // Emit root node
        let root_name = root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| root.to_string_lossy().to_string());
        self.nodes
            .push(FileNode::new(root.clone(), root_name, 0, true));

        // DFS traversal using the collected and sorted children
        self.emit_children(&root, 1, &children_map);

        Ok(())
    }

    fn emit_children(
        &mut self,
        dir: &Path,
        depth: usize,
        children_map: &HashMap<PathBuf, Vec<(PathBuf, bool)>>,
    ) {
        if let Some(children) = children_map.get(dir) {
            for (child_path, is_dir) in children {
                let name = child_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| child_path.to_string_lossy().to_string());

                self.nodes
                    .push(FileNode::new(child_path.clone(), name, depth, *is_dir));

                if *is_dir {
                    self.emit_children(child_path, depth + 1, children_map);
                }
            }
        }
    }

    pub fn refresh(&mut self) {
        let _ = self.rebuild_visible_nodes();
    }
}
