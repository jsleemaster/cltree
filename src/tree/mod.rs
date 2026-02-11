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
        self.build_tree(&self.root.clone(), 0)?;
        Ok(())
    }

    fn build_tree(&mut self, path: &Path, depth: usize) -> Result<()> {
        if depth > self.max_depth {
            return Ok(());
        }

        let walker = WalkBuilder::new(path)
            .hidden(!self.show_hidden)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .max_depth(Some(1))
            .build();

        let mut entries: Vec<_> = walker.flatten().collect();
        entries.sort_by(|a, b| {
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

        for entry in entries {
            let entry_path = entry.path();

            // Skip the root itself when iterating
            if entry_path == path && depth > 0 {
                continue;
            }

            // At depth 0, only process the root entry itself.
            // Children will be added by the recursive call below.
            if depth == 0 && entry_path != path {
                continue;
            }

            let is_dir = entry_path.is_dir();
            let name = entry_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| entry_path.to_string_lossy().to_string());

            // Skip hidden files if not showing them
            if !self.show_hidden && name.starts_with('.') && depth > 0 {
                continue;
            }

            let node = FileNode::new(entry_path.to_path_buf(), name, depth, is_dir);
            self.nodes.push(node);

            // Always recurse into directories (all expanded)
            if is_dir {
                self.build_tree(entry_path, depth + 1)?;
            }
        }

        Ok(())
    }

    pub fn refresh(&mut self) {
        let _ = self.rebuild_visible_nodes();
    }
}
