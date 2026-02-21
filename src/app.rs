use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::prelude::Rect;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::terminal::TerminalPane;
use crate::tree::FileTree;

pub struct Selection {
    pub start: (u16, u16), // (col, row) terminal-local coordinates
    pub end: (u16, u16),
}

pub struct App {
    pub tree: FileTree,
    pub terminal: TerminalPane,
    pub tree_width_percent: u16,
    pub tree_loading: bool,
    pub tree_area: Option<Rect>,
    pub terminal_area: Option<Rect>,
    pub selection: Option<Selection>,
    pub last_auto_scroll_cwd: Option<PathBuf>,
}

impl App {
    pub fn new(
        path: PathBuf,
        tree_width: u16,
        show_hidden: bool,
        max_depth: usize,
        claude_args: Vec<String>,
        pty_tx: mpsc::UnboundedSender<()>,
    ) -> Result<Self> {
        let canonical_path = path.canonicalize().unwrap_or(path);

        Ok(Self {
            tree: FileTree::new(&canonical_path, show_hidden, max_depth)?,
            terminal: TerminalPane::new(&canonical_path, &claude_args, pty_tx)?,
            tree_width_percent: tree_width.clamp(10, 50),
            tree_loading: true,
            tree_area: None,
            terminal_area: None,
            selection: None,
            last_auto_scroll_cwd: None,
        })
    }

    pub fn tick(&mut self) -> bool {
        self.terminal.tick();

        // CWD가 트리 루트 밖이면 트리 루트 갱신
        let cwd = self.terminal.cwd().to_path_buf();
        if !cwd.starts_with(self.tree.root_path()) {
            self.tree.set_root(cwd);
            self.last_auto_scroll_cwd = None;
        }

        // Process clipboard requests from vterm (OSC 52)
        {
            let requests = self.terminal.vterm_lock().take_clipboard_requests();
            for text in requests {
                copy_to_clipboard(&text);
            }
        }
        if self.tree_loading {
            self.tree_loading = false;
        }
        self.terminal.is_process_exited()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.selection = None;
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.terminal.send_interrupt();
                false
            }
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => true,
            _ => {
                self.terminal.handle_key(key);
                false
            }
        }
    }

    pub fn handle_paste(&mut self, text: String) {
        self.selection = None;
        self.terminal.handle_paste(text);
    }

    pub fn handle_mouse(&mut self, event: MouseEvent) {
        let in_tree = self.tree_area.is_some_and(|area| {
            event.column >= area.x
                && event.column < area.x + area.width
                && event.row >= area.y
                && event.row < area.y + area.height
        });

        let in_terminal = self.terminal_area.is_some_and(|area| {
            event.column >= area.x
                && event.column < area.x + area.width
                && event.row >= area.y
                && event.row < area.y + area.height
        });

        match event.kind {
            MouseEventKind::ScrollUp => {
                if in_tree {
                    let offset = self.tree.offset();
                    self.tree.set_offset(offset.saturating_sub(3));
                } else {
                    self.terminal.scroll_up();
                }
            }
            MouseEventKind::ScrollDown => {
                if in_tree {
                    let visible_height = self.tree_area.map(|a| a.height as usize).unwrap_or(1);
                    let max_offset = self.tree.nodes().len().saturating_sub(visible_height);
                    let offset = (self.tree.offset() + 3).min(max_offset);
                    self.tree.set_offset(offset);
                } else {
                    self.terminal.scroll_down();
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if in_terminal {
                    let area = self.terminal_area.unwrap();
                    let col = event.column.saturating_sub(area.x);
                    let row = event.row.saturating_sub(area.y);
                    self.selection = Some(Selection {
                        start: (col, row),
                        end: (col, row),
                    });
                } else {
                    self.selection = None;
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if let Some(ref mut sel) = self.selection {
                    if let Some(area) = self.terminal_area {
                        let col = event
                            .column
                            .saturating_sub(area.x)
                            .min(area.width.saturating_sub(1));
                        let row = event
                            .row
                            .saturating_sub(area.y)
                            .min(area.height.saturating_sub(1));
                        sel.end = (col, row);
                    }
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if let Some(sel) = self.selection.as_ref() {
                    // Only copy if the selection spans more than a single point
                    if sel.start != sel.end {
                        let text = self.terminal.extract_text(sel.start, sel.end);
                        if !text.is_empty() {
                            copy_to_clipboard(&text);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_file_change(&mut self, path: PathBuf) {
        // Refresh tree if file changed
        if path.starts_with(self.tree.root_path()) {
            self.tree.refresh();
        }
    }
}

pub(crate) fn copy_to_clipboard(text: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        try_clipboard_cmd("pbcopy", &[], text)
    }

    #[cfg(target_os = "linux")]
    {
        // Try xclip first, then xsel, then wl-copy (Wayland)
        try_clipboard_cmd("xclip", &["-selection", "clipboard"], text)
            || try_clipboard_cmd("xsel", &["--clipboard", "--input"], text)
            || try_clipboard_cmd("wl-copy", &[], text)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = text;
        false
    }
}

fn try_clipboard_cmd(program: &str, args: &[&str], text: &str) -> bool {
    use std::io::Write;
    use std::process::{Command, Stdio};

    match Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(mut child) => {
            if let Some(ref mut stdin) = child.stdin {
                let _ = stdin.write_all(text.as_bytes());
            }
            child.wait().map(|s| s.success()).unwrap_or(false)
        }
        Err(_) => false,
    }
}
