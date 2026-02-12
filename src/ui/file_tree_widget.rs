use std::path::Path;

use ratatui::{prelude::*, widgets::StatefulWidget};

use super::FileTreeWidgetState;
use crate::tree::FileTree;

pub struct FileTreeWidget<'a> {
    tree: &'a FileTree,
    cwd: Option<&'a Path>,
}

impl<'a> FileTreeWidget<'a> {
    pub fn new(tree: &'a FileTree, cwd: Option<&'a Path>) -> Self {
        Self { tree, cwd }
    }
}

impl<'a> StatefulWidget for FileTreeWidget<'a> {
    type State = FileTreeWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let nodes = self.tree.nodes();
        let visible_height = area.height as usize;

        // Calculate visible range
        let start = state.offset;
        let end = (start + visible_height).min(nodes.len());

        for (i, idx) in (start..end).enumerate() {
            if idx >= nodes.len() {
                break;
            }

            let node = &nodes[idx];
            let y = area.y + i as u16;

            if y >= area.y + area.height {
                break;
            }

            // Check if this node is the CWD
            let is_cwd = self.cwd.is_some_and(|cwd| node.is_dir && node.path == cwd);

            // Clear background for CWD item
            if is_cwd {
                for x in area.x..area.x + area.width {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_bg(Color::Rgb(80, 70, 30));
                    }
                }
            }

            let tree_style = Style::default().fg(Color::DarkGray);
            let node_style = if is_cwd {
                Style::default()
                    .bg(Color::Rgb(80, 70, 30))
                    .fg(Color::Rgb(255, 220, 100))
                    .bold()
            } else {
                let color = node.display_color();
                let mut s = Style::default().fg(color);
                if node.is_dir {
                    s = s.bold();
                }
                s
            };

            let mut x_offset = area.x;

            if node.depth == 0 {
                // Root node: icon + name, no tree prefix
                let icon = node.expanded_icon(true);
                let display = if is_cwd {
                    format!("{}● {}", icon, node.name)
                } else {
                    format!("{} {}", icon, node.name)
                };
                buf.set_string(x_offset, y, &display, node_style);
                x_offset += unicode_width::UnicodeWidthStr::width(display.as_str()) as u16;
            } else {
                // Draw ancestor connectors
                for &ancestor_is_last in &node.connector {
                    let connector_str = if ancestor_is_last { "    " } else { "│   " };
                    buf.set_string(x_offset, y, connector_str, tree_style);
                    x_offset += 4;
                }

                // Draw this node's branch connector
                let branch = if node.is_last {
                    "└── "
                } else {
                    "├── "
                };
                buf.set_string(x_offset, y, branch, tree_style);
                x_offset += 4;

                // Draw icon + name
                let icon = node.expanded_icon(true);
                let display = if is_cwd {
                    format!("{}● {}", icon, node.name)
                } else {
                    format!("{} {}", icon, node.name)
                };
                buf.set_string(x_offset, y, &display, node_style);
                x_offset += unicode_width::UnicodeWidthStr::width(display.as_str()) as u16;
            }

            // Truncate if too long
            let total_width = x_offset.saturating_sub(area.x);
            if total_width > area.width {
                if let Some(x) = area.x.checked_add(area.width.saturating_sub(1)) {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_symbol("…");
                    }
                }
            }
        }

        // Show scroll indicator if needed
        if nodes.len() > visible_height {
            let scrollbar_height =
                visible_height as f32 / nodes.len() as f32 * visible_height as f32;
            let scrollbar_height = scrollbar_height.max(1.0) as u16;
            let scrollbar_pos =
                (state.offset as f32 / nodes.len() as f32 * visible_height as f32) as u16;

            let scrollbar_x = area.x + area.width - 1;
            for y in 0..visible_height as u16 {
                let ch = if y >= scrollbar_pos && y < scrollbar_pos + scrollbar_height {
                    "█"
                } else {
                    "░"
                };
                if let Some(cell) = buf.cell_mut((scrollbar_x, area.y + y)) {
                    cell.set_symbol(ch);
                    cell.set_fg(Color::DarkGray);
                }
            }
        }
    }
}
