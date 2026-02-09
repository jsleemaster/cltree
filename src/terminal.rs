use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::vterm::VirtualTerminal;

pub struct TerminalPane {
    pty_pair: Option<PtyPair>,
    pty_writer: Option<Box<dyn Write + Send>>,
    vterm: Arc<Mutex<VirtualTerminal>>,
    cwd: std::path::PathBuf,
    process_exited: Arc<AtomicBool>,
}

impl TerminalPane {
    pub fn new(cwd: &Path, claude_args: &[String]) -> anyhow::Result<Self> {
        let vterm = Arc::new(Mutex::new(VirtualTerminal::new(80, 24)));
        let process_exited = Arc::new(AtomicBool::new(false));

        // Try to create PTY and spawn claude process
        let (pty_pair, pty_writer) =
            match Self::try_spawn_claude(cwd, &vterm, claude_args, &process_exited) {
                Ok((pair, writer)) => (Some(pair), writer),
                Err(e) => {
                    // Store error message in vterm so user can see it
                    let msg = format!(
                        "Failed to start Claude Code: {}\r\n\r\n\
                     Make sure 'claude' CLI is installed and in your PATH.\r\n\
                     Install: npm install -g @anthropic-ai/claude-code\r\n",
                        e
                    );
                    vterm.lock().unwrap().feed(msg.as_bytes());
                    (None, None)
                }
            };

        Ok(Self {
            pty_pair,
            pty_writer,
            vterm,
            cwd: cwd.to_path_buf(),
            process_exited,
        })
    }

    fn try_spawn_claude(
        cwd: &Path,
        vterm: &Arc<Mutex<VirtualTerminal>>,
        claude_args: &[String],
        process_exited: &Arc<AtomicBool>,
    ) -> anyhow::Result<(PtyPair, Option<Box<dyn Write + Send>>)> {
        // Create PTY
        let pty_system = native_pty_system();
        let pty_pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Spawn claude process
        let mut cmd = CommandBuilder::new("claude");
        cmd.cwd(cwd);
        for arg in claude_args {
            cmd.arg(arg);
        }

        let mut child = pty_pair.slave.spawn_command(cmd)?;

        // Read output in background thread
        let mut reader = pty_pair.master.try_clone_reader()?;
        let vterm_clone = Arc::clone(vterm);
        let exited_clone = Arc::clone(process_exited);

        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let mut vt = vterm_clone.lock().unwrap();
                        vt.feed(&buf[..n]);
                    }
                    Err(_) => break,
                }
            }
            exited_clone.store(true, Ordering::SeqCst);
            let _ = child.wait();
        });

        // Take the writer from master PTY (can only be called once)
        let pty_writer = pty_pair.master.take_writer().ok();

        Ok((pty_pair, pty_writer))
    }

    pub fn tick(&mut self) {
        // Called on each tick - can be used for animations or updates
    }

    pub fn is_process_exited(&self) -> bool {
        self.process_exited.load(Ordering::SeqCst)
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        let bytes = match (key.code, key.modifiers) {
            (KeyCode::Char(c), KeyModifiers::NONE) => vec![c as u8],
            (KeyCode::Char(c), KeyModifiers::SHIFT) => vec![c.to_ascii_uppercase() as u8],
            (KeyCode::Char(c), KeyModifiers::CONTROL) => {
                // Ctrl+A = 1, Ctrl+B = 2, etc.
                let ctrl_char = (c.to_ascii_lowercase() as u8).wrapping_sub(b'a' - 1);
                vec![ctrl_char]
            }
            (KeyCode::Enter, _) => vec![b'\r'],
            (KeyCode::Backspace, _) => vec![127],
            (KeyCode::Delete, _) => vec![27, b'[', b'3', b'~'],
            (KeyCode::Tab, _) => vec![b'\t'],
            (KeyCode::Up, _) => vec![27, b'[', b'A'],
            (KeyCode::Down, _) => vec![27, b'[', b'B'],
            (KeyCode::Right, _) => vec![27, b'[', b'C'],
            (KeyCode::Left, _) => vec![27, b'[', b'D'],
            (KeyCode::Home, _) => vec![27, b'[', b'H'],
            (KeyCode::End, _) => vec![27, b'[', b'F'],
            (KeyCode::PageUp, _) => vec![27, b'[', b'5', b'~'],
            (KeyCode::PageDown, _) => vec![27, b'[', b'6', b'~'],
            (KeyCode::Esc, _) => vec![27],
            _ => return,
        };

        if let Some(ref mut writer) = self.pty_writer {
            let _ = writer.write_all(&bytes);
            let _ = writer.flush();
        }
    }

    pub fn send_interrupt(&mut self) {
        if let Some(ref mut writer) = self.pty_writer {
            let _ = writer.write_all(&[3]); // Ctrl+C
            let _ = writer.flush();
        }
    }

    pub fn insert_text(&mut self, text: &str) {
        if let Some(ref mut writer) = self.pty_writer {
            let _ = writer.write_all(text.as_bytes());
            let _ = writer.flush();
        }
    }

    pub fn change_directory(&mut self, path: &Path) {
        // Send cd command to the terminal
        let cmd = format!("cd {}\r", path.display());
        self.insert_text(&cmd);
        self.cwd = path.to_path_buf();
    }

    pub fn vterm(&self) -> &Arc<Mutex<VirtualTerminal>> {
        &self.vterm
    }

    pub fn scroll_up(&mut self) {
        let mut vt = self.vterm.lock().unwrap();
        let current = vt.scroll_offset();
        vt.set_scroll_offset(current + 3);
    }

    pub fn scroll_down(&mut self) {
        let mut vt = self.vterm.lock().unwrap();
        let current = vt.scroll_offset();
        vt.set_scroll_offset(current.saturating_sub(3));
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        // Resize the PTY
        if let Some(ref pty_pair) = self.pty_pair {
            let _ = pty_pair.master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            });
        }
        // Resize the virtual terminal grid
        let mut vt = self.vterm.lock().unwrap();
        vt.resize(cols as usize, rows as usize);
    }
}

impl Drop for TerminalPane {
    fn drop(&mut self) {
        // PTY will be cleaned up automatically
        self.pty_pair.take();
    }
}
