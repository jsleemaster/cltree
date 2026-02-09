mod file_tree_widget;
mod help_popup;
mod terminal_widget;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

use crate::app::{App, FocusedPane, InputMode};
use file_tree_widget::FileTreeWidget;
use help_popup::HelpPopup;
use terminal_widget::TerminalWidget;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Main layout: tree on right, terminal on left
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(100 - app.tree_width_percent),
            Constraint::Percentage(app.tree_width_percent),
        ])
        .split(size);

    // Terminal pane (left/main area)
    let terminal_area = chunks[0];
    let terminal_block = Block::default()
        .title(" Claude Code ")
        .title_style(Style::default().fg(Color::Cyan).bold())
        .borders(Borders::ALL)
        .border_style(if app.focused == FocusedPane::Terminal {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let terminal_inner = terminal_block.inner(terminal_area);
    frame.render_widget(terminal_block, terminal_area);

    // Resize PTY to match terminal area
    app.terminal
        .resize(terminal_inner.width, terminal_inner.height);

    let terminal_widget = TerminalWidget::new(&app.terminal);
    frame.render_widget(terminal_widget, terminal_inner);

    // File tree pane (right side)
    let tree_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(chunks[1]);

    let tree_area = tree_chunks[0];
    let status_area = tree_chunks[1];

    let tree_title = format!(
        " 📂 {} ",
        app.tree
            .root_path()
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| app.tree.root_path().to_string_lossy().to_string())
    );

    let tree_block = Block::default()
        .title(tree_title)
        .title_style(Style::default().fg(Color::Yellow).bold())
        .borders(Borders::ALL)
        .border_style(if app.focused == FocusedPane::Tree {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let tree_inner = tree_block.inner(tree_area);
    frame.render_widget(tree_block, tree_area);

    // Render file tree
    let file_tree_widget = FileTreeWidget::new(&app.tree);
    frame.render_stateful_widget(
        file_tree_widget,
        tree_inner,
        &mut FileTreeWidgetState {
            offset: app.tree.offset(),
        },
    );

    // Update scroll offset
    let visible_height = tree_inner.height as usize;
    let selected = app.tree.selected();
    let mut offset = app.tree.offset();

    if selected >= offset + visible_height {
        offset = selected - visible_height + 1;
    } else if selected < offset {
        offset = selected;
    }
    app.tree.set_offset(offset);

    // Status bar / search input
    let status_content = if app.input_mode == InputMode::Search {
        format!("/{}", app.search_query)
    } else if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        "Tab: switch pane | ?: help".to_string()
    };

    let status_style = if app.input_mode == InputMode::Search {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let status = Paragraph::new(status_content).style(status_style);
    frame.render_widget(status, status_area);

    // Help popup
    if app.show_help {
        let help = HelpPopup::new();
        let help_area = centered_rect(60, 70, size);
        frame.render_widget(help, help_area);
    }
}

pub struct FileTreeWidgetState {
    pub offset: usize,
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
