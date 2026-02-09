use ratatui::{prelude::*, widgets::StatefulWidget};

use super::FileTreeWidgetState;
use crate::tree::FileTree;

pub struct FileTreeWidget<'a> {
    tree: &'a FileTree,
}

impl<'a> FileTreeWidget<'a> {
    pub fn new(tree: &'a FileTree) -> Self {
        Self { tree }
    }
}

impl<'a> StatefulWidget for FileTreeWidget<'a> {
    type State = FileTreeWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let nodes = self.tree.nodes();
        let selected = self.tree.selected();
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

            // Build the line
            let indent = "  ".repeat(node.depth);
            let icon = node.expanded_icon(self.tree.is_expanded(&node.path));
            let line = format!("{}{} {}", indent, icon, node.name);

            // Determine style
            let style = if idx == selected {
                Style::default()
                    .bg(Color::Rgb(60, 60, 80))
                    .fg(Color::White)
                    .bold()
            } else if node.is_dir {
                Style::default().fg(Color::LightCyan).bold()
            } else {
                Style::default().fg(Color::White)
            };

            // Clear background for selected item
            if idx == selected {
                for x in area.x..area.x + area.width {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_bg(Color::Rgb(60, 60, 80));
                    }
                }
            }

            buf.set_string(area.x, y, &line, style);

            // Truncate if too long
            let display_width = unicode_width::UnicodeWidthStr::width(line.as_str());
            if display_width > area.width as usize {
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
