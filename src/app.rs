use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use std::path::PathBuf;

use crate::terminal::TerminalPane;
use crate::tree::FileTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
    Tree,
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Search,
}

pub struct App {
    pub tree: FileTree,
    pub terminal: TerminalPane,
    pub focused: FocusedPane,
    pub input_mode: InputMode,
    pub search_query: String,
    pub tree_width_percent: u16,
    pub show_help: bool,
    #[allow(dead_code)]
    pub should_quit: bool,
    pub status_message: Option<String>,
}

impl App {
    pub fn new(
        path: PathBuf,
        tree_width: u16,
        show_hidden: bool,
        max_depth: usize,
        claude_args: Vec<String>,
    ) -> Result<Self> {
        let canonical_path = path.canonicalize().unwrap_or(path);

        Ok(Self {
            tree: FileTree::new(&canonical_path, show_hidden, max_depth)?,
            terminal: TerminalPane::new(&canonical_path, &claude_args)?,
            focused: FocusedPane::Terminal,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            tree_width_percent: tree_width.clamp(10, 50),
            show_help: false,
            should_quit: false,
            status_message: None,
        })
    }

    pub fn tick(&mut self) -> bool {
        self.terminal.tick();
        self.terminal.is_process_exited()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Global shortcuts
        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                if self.focused == FocusedPane::Terminal {
                    self.terminal.send_interrupt();
                    return false;
                }
                return true;
            }
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => return true,
            _ => {}
        }

        // Help toggle
        if key.code == KeyCode::F(1)
            || (key.code == KeyCode::Char('?') && self.input_mode == InputMode::Normal)
        {
            self.show_help = !self.show_help;
            return false;
        }

        // Help dismissal
        if self.show_help {
            self.show_help = false;
            return false;
        }

        // Search mode
        if self.input_mode == InputMode::Search {
            return self.handle_search_input(key);
        }

        match self.focused {
            FocusedPane::Tree => self.handle_tree_key(key),
            FocusedPane::Terminal => self.handle_terminal_key(key),
        }
    }

    fn handle_search_input(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => {
                self.tree.search(&self.search_query);
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Esc => {
                self.search_query.clear();
                self.input_mode = InputMode::Normal;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
            }
            _ => {}
        }
        false
    }

    fn handle_tree_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => self.tree.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => self.tree.select_next(),
            KeyCode::Home | KeyCode::Char('g') => self.tree.select_first(),
            KeyCode::End | KeyCode::Char('G') => self.tree.select_last(),
            KeyCode::PageUp => self.tree.page_up(10),
            KeyCode::PageDown => self.tree.page_down(10),

            // Actions
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                if let Some(path) = self.tree.toggle_or_open() {
                    if path.is_dir() {
                        self.terminal.change_directory(&path);
                        self.set_status(format!("Changed to: {}", path.display()));
                    } else {
                        // Copy path to terminal input
                        let path_str = path.to_string_lossy();
                        self.terminal.insert_text(&format!("@{} ", path_str));
                        self.focused = FocusedPane::Terminal;
                    }
                }
            }
            KeyCode::Left | KeyCode::Char('h') => self.tree.collapse_or_parent(),
            KeyCode::Char(' ') => self.tree.toggle_expand(),

            // Refresh
            KeyCode::Char('r') | KeyCode::F(5) => {
                self.tree.refresh();
                self.set_status("Tree refreshed".to_string());
            }

            // Search
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.search_query.clear();
            }
            KeyCode::Char('n') => self.tree.search_next(),
            KeyCode::Char('N') => self.tree.search_prev(),

            // Toggle hidden files
            KeyCode::Char('.') => {
                self.tree.toggle_hidden();
                self.set_status(
                    if self.tree.show_hidden {
                        "Showing hidden files"
                    } else {
                        "Hiding hidden files"
                    }
                    .to_string(),
                );
            }

            // Switch pane
            KeyCode::Tab | KeyCode::Char('\t') => self.focused = FocusedPane::Terminal,
            KeyCode::Esc => self.focused = FocusedPane::Terminal,

            _ => {}
        }
        false
    }

    fn handle_terminal_key(&mut self, key: KeyEvent) -> bool {
        match (key.code, key.modifiers) {
            // Switch to tree pane
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.focused = FocusedPane::Tree;
            }
            (KeyCode::Char('t'), KeyModifiers::CONTROL) => {
                self.focused = FocusedPane::Tree;
            }
            // Pass all other keys to terminal
            _ => {
                self.terminal.handle_key(key);
            }
        }
        false
    }

    pub fn handle_mouse(&mut self, event: MouseEvent) {
        match event.kind {
            MouseEventKind::Down(_) => {
                // Determine which pane was clicked based on x position
                // This is a simplified version; actual implementation would
                // need to know the current layout dimensions
            }
            MouseEventKind::ScrollUp => {
                if self.focused == FocusedPane::Tree {
                    self.tree.select_previous();
                } else {
                    self.terminal.scroll_up();
                }
            }
            MouseEventKind::ScrollDown => {
                if self.focused == FocusedPane::Tree {
                    self.tree.select_next();
                } else {
                    self.terminal.scroll_down();
                }
            }
            _ => {}
        }
    }

    pub fn handle_resize(&mut self, _width: u16, _height: u16) {
        // Handle terminal resize if needed
    }

    pub fn handle_file_change(&mut self, path: PathBuf) {
        // Refresh tree if file changed
        if path.starts_with(self.tree.root_path()) {
            self.tree.refresh_path(&path);
        }
    }

    fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
    }

    #[allow(dead_code)]
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }
}
