use ratatui::{prelude::*, widgets::Widget};

use crate::terminal::TerminalPane;

pub struct TerminalWidget<'a> {
    terminal: &'a TerminalPane,
}

impl<'a> TerminalWidget<'a> {
    pub fn new(terminal: &'a TerminalPane) -> Self {
        Self { terminal }
    }
}

impl<'a> Widget for TerminalWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let vterm = self.terminal.vterm().lock().unwrap();
        let grid = vterm.grid();
        let scrollback = vterm.scrollback();
        let scroll_offset = vterm.scroll_offset();

        if scroll_offset == 0 {
            // Normal mode: render the grid directly
            let rows_to_render = (area.height as usize).min(grid.len());
            let cols_to_render = (area.width as usize).min(vterm.cols());

            for row_idx in 0..rows_to_render {
                if let Some(row) = grid.get(row_idx) {
                    for (col_idx, cell) in row.iter().enumerate().take(cols_to_render) {
                        let x = area.x + col_idx as u16;
                        let y = area.y + row_idx as u16;
                        if x < area.x + area.width && y < area.y + area.height {
                            if let Some(buf_cell) = buf.cell_mut((x, y)) {
                                buf_cell.set_symbol(&cell.ch.to_string());
                                buf_cell.set_style(cell.style);
                            }
                        }
                    }
                }
            }

            // Render cursor (inverted style)
            let cursor = vterm.cursor();
            if cursor.visible {
                let cx = area.x + cursor.x as u16;
                let cy = area.y + cursor.y as u16;
                if cx < area.x + area.width && cy < area.y + area.height {
                    if let Some(cell) = buf.cell_mut((cx, cy)) {
                        let current_style = cell.style();
                        cell.set_style(current_style.add_modifier(Modifier::REVERSED));
                    }
                }
            }
        } else {
            // Scrollback mode: mix scrollback + grid
            let visible_height = area.height as usize;
            let cols_to_render = (area.width as usize).min(vterm.cols());
            let total_lines = scrollback.len() + grid.len();

            // scroll_offset is how many lines above the bottom of the grid we are
            let bottom = total_lines.saturating_sub(scroll_offset);
            let top = bottom.saturating_sub(visible_height);

            for (screen_row, line_idx) in (top..bottom).enumerate() {
                let row_data = if line_idx < scrollback.len() {
                    scrollback.get(line_idx)
                } else {
                    grid.get(line_idx - scrollback.len())
                };

                if let Some(row) = row_data {
                    for (col_idx, cell) in row.iter().enumerate().take(cols_to_render) {
                        let x = area.x + col_idx as u16;
                        let y = area.y + screen_row as u16;
                        if x < area.x + area.width && y < area.y + area.height {
                            if let Some(buf_cell) = buf.cell_mut((x, y)) {
                                buf_cell.set_symbol(&cell.ch.to_string());
                                buf_cell.set_style(cell.style);
                            }
                        }
                    }
                }
            }
        }
    }
}
