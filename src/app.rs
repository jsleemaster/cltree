use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::terminal::TerminalPane;
use crate::tree::FileTree;

pub struct App {
    pub tree: FileTree,
    pub terminal: TerminalPane,
    pub tree_width_percent: u16,
    pub tree_loading: bool,
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
        })
    }

    pub fn tick(&mut self) -> bool {
        self.terminal.tick();
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

    pub fn handle_mouse(&mut self, event: MouseEvent) {
        match event.kind {
            MouseEventKind::ScrollUp => {
                self.terminal.scroll_up();
            }
            MouseEventKind::ScrollDown => {
                self.terminal.scroll_down();
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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
