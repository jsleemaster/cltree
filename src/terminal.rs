use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use tokio::sync::mpsc;

use crate::vterm::VirtualTerminal;

/// Lock a mutex, recovering from poison (prior thread panic).
fn lock_or_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub struct TerminalPane {
    pty_pair: Option<PtyPair>,
    pty_writer: Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    vterm: Arc<Mutex<VirtualTerminal>>,
    cwd: PathBuf,
    child_pid: Option<u32>,
    process_exited: Arc<AtomicBool>,
    last_cols: u16,
    last_rows: u16,
    // Debounce: to revert CWD to a shallower path, it must be the deepest
    // found for several consecutive ticks (prevents flickering)
    shallow_revert_count: u32,
}

impl TerminalPane {
    pub fn new(
        cwd: &Path,
        claude_args: &[String],
        pty_tx: mpsc::UnboundedSender<()>,
    ) -> anyhow::Result<Self> {
        let vterm = Arc::new(Mutex::new(VirtualTerminal::new(80, 24)));
        let process_exited = Arc::new(AtomicBool::new(false));

        let pty_writer: Arc<Mutex<Option<Box<dyn Write + Send>>>> = Arc::new(Mutex::new(None));

        // Try to create PTY and spawn claude process
        let (pty_pair, child_pid) = match Self::try_spawn_claude(
            cwd,
            &vterm,
            claude_args,
            &process_exited,
            pty_tx,
            &pty_writer,
        ) {
            Ok((pair, pid)) => (Some(pair), pid),
            Err(e) => {
                // Store error message in vterm so user can see it
                let msg = format!(
                    "Failed to start Claude Code: {}\r\n\r\n\
                     Make sure 'claude' CLI is installed and in your PATH.\r\n\
                     Install: npm install -g @anthropic-ai/claude-code\r\n",
                    e
                );
                lock_or_recover(&vterm).feed(msg.as_bytes());
                (None, None)
            }
        };

        Ok(Self {
            pty_pair,
            pty_writer,
            vterm,
            cwd: cwd.to_path_buf(),
            child_pid,
            process_exited,
            last_cols: 80,
            last_rows: 24,
            shallow_revert_count: 0,
        })
    }

    fn try_spawn_claude(
        cwd: &Path,
        vterm: &Arc<Mutex<VirtualTerminal>>,
        claude_args: &[String],
        process_exited: &Arc<AtomicBool>,
        pty_tx: mpsc::UnboundedSender<()>,
        pty_writer: &Arc<Mutex<Option<Box<dyn Write + Send>>>>,
    ) -> anyhow::Result<(PtyPair, Option<u32>)> {
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
        cmd.env("TERM", "xterm-256color");

        let mut child = pty_pair.slave.spawn_command(cmd)?;

        // Get child PID before moving child into the thread
        let child_pid = child.process_id();

        // Take the writer from master PTY (can only be called once)
        // Store it in the shared Arc<Mutex<>> so both main thread and reader thread can use it
        if let Ok(writer) = pty_pair.master.take_writer() {
            *lock_or_recover(pty_writer) = Some(writer);
        }

        // Read output in background thread
        let mut reader = pty_pair.master.try_clone_reader()?;
        let vterm_clone = Arc::clone(vterm);
        let exited_clone = Arc::clone(process_exited);
        let writer_clone = Arc::clone(pty_writer);

        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let mut vt = lock_or_recover(&vterm_clone);
                        vt.feed(&buf[..n]);
                        // Flush any DSR/CPR responses back to the PTY
                        let responses = vt.take_responses();
                        drop(vt); // Release lock before I/O
                        if !responses.is_empty() {
                            if let Ok(mut guard) = writer_clone.lock() {
                                if let Some(ref mut writer) = *guard {
                                    for resp in responses {
                                        let _ = writer.write_all(&resp);
                                    }
                                    let _ = writer.flush();
                                }
                            }
                        }
                        let _ = pty_tx.send(());
                    }
                    Err(_) => break,
                }
            }
            exited_clone.store(true, Ordering::SeqCst);
            let _ = child.wait();
        });

        Ok((pty_pair, child_pid))
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    pub fn tick(&mut self) {
        // Try OSC 7 first (shell-reported CWD)
        if let Ok(vt) = self.vterm.lock() {
            if let Some(reported) = vt.reported_cwd() {
                if reported != self.cwd {
                    self.cwd = reported.to_path_buf();
                    self.shallow_revert_count = 0;
                    return;
                }
            }

            // Scan vterm buffer for CWD path displayed by Claude Code.
            // Collect ALL valid paths and pick the deepest (most specific) one.
            let home = dirs::home_dir().unwrap_or_default();
            let mut best_path: Option<PathBuf> = None;
            let mut best_depth: usize = 0;

            for row in 0..8.min(vt.rows()) {
                let text = vt.row_text(row);
                let trimmed = text.trim();

                let resolved = if let Some(pos) = trimmed.find("~/") {
                    let rest = &trimmed[pos + 2..];
                    let path_part: String = rest
                        .chars()
                        .take_while(|c| {
                            c.is_alphanumeric() || matches!(*c, '/' | '.' | '-' | '_' | '+' | '@')
                        })
                        .collect();
                    if path_part.is_empty() {
                        None
                    } else {
                        Some(home.join(path_part.trim_end_matches('/')))
                    }
                } else if let Some(pos) = trimmed.find('/') {
                    let path_part: String = trimmed[pos..]
                        .chars()
                        .take_while(|c| {
                            c.is_alphanumeric() || matches!(*c, '/' | '.' | '-' | '_' | '+' | '@')
                        })
                        .collect();
                    if path_part.len() > 1 {
                        Some(PathBuf::from(path_part.trim_end_matches('/')))
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(path) = resolved {
                    if path.is_dir() {
                        let depth = path.components().count();
                        if depth > best_depth {
                            best_depth = depth;
                            best_path = Some(path);
                        }
                    }
                }
            }

            if let Some(path) = best_path {
                if path == self.cwd {
                    // Same as current — stable, reset counter
                    self.shallow_revert_count = 0;
                } else {
                    let new_depth = path.components().count();
                    let cur_depth = self.cwd.components().count();

                    if new_depth > cur_depth {
                        // Deeper path found — apply immediately
                        self.cwd = path;
                        self.shallow_revert_count = 0;
                    } else {
                        // Shallower path — require consistent detection before reverting
                        // (prevents flickering when deeper path disappears temporarily)
                        self.shallow_revert_count += 1;
                        if self.shallow_revert_count >= 16 {
                            // ~4 seconds of consistent shallow detection
                            self.cwd = path;
                            self.shallow_revert_count = 0;
                        }
                    }
                }
                return;
            }
        }

        // Fall back to polling the child process's actual CWD via OS API
        if let Some(pid) = self.child_pid {
            if let Some(proc_cwd) = get_process_cwd(pid) {
                if proc_cwd != self.cwd {
                    self.cwd = proc_cwd;
                    self.shallow_revert_count = 0;
                }
            }
        }
    }

    pub fn is_process_exited(&self) -> bool {
        self.process_exited.load(Ordering::SeqCst)
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Compute modifier parameter for CSI sequences (xterm style)
        // 1=none, 2=Shift, 3=Alt, 4=Shift+Alt, 5=Ctrl, 6=Ctrl+Shift, 7=Ctrl+Alt, 8=Ctrl+Shift+Alt
        let modifier_param = |mods: KeyModifiers| -> u8 {
            let mut param = 1u8;
            if mods.contains(KeyModifiers::SHIFT) {
                param += 1;
            }
            if mods.contains(KeyModifiers::ALT) {
                param += 2;
            }
            if mods.contains(KeyModifiers::CONTROL) {
                param += 4;
            }
            param
        };

        let bytes: Vec<u8> = match key.code {
            // --- Character keys ---
            KeyCode::Char(c) => {
                let mods = key.modifiers;
                if mods == KeyModifiers::NONE || mods == KeyModifiers::SHIFT {
                    // Normal or shifted character — send as UTF-8
                    let ch = if mods.contains(KeyModifiers::SHIFT) {
                        c.to_uppercase().next().unwrap_or(c)
                    } else {
                        c
                    };
                    let mut buf = [0u8; 4];
                    let s = ch.encode_utf8(&mut buf);
                    s.as_bytes().to_vec()
                } else if mods == KeyModifiers::CONTROL {
                    // Ctrl+A=1 .. Ctrl+Z=26
                    let ctrl_char = (c.to_ascii_lowercase() as u8).wrapping_sub(b'a' - 1);
                    vec![ctrl_char]
                } else if mods == KeyModifiers::ALT {
                    // Alt+char: ESC prefix + char
                    let mut v = vec![0x1b];
                    let mut buf = [0u8; 4];
                    let s = c.encode_utf8(&mut buf);
                    v.extend_from_slice(s.as_bytes());
                    v
                } else if mods == KeyModifiers::CONTROL | KeyModifiers::ALT {
                    // Ctrl+Alt+char: ESC prefix + ctrl char
                    let ctrl_char = (c.to_ascii_lowercase() as u8).wrapping_sub(b'a' - 1);
                    vec![0x1b, ctrl_char]
                } else {
                    // Fallback: send as UTF-8
                    let mut buf = [0u8; 4];
                    let s = c.encode_utf8(&mut buf);
                    s.as_bytes().to_vec()
                }
            }

            // --- Simple keys (no modifier variants) ---
            KeyCode::Enter => vec![b'\r'],
            KeyCode::Backspace => {
                if key.modifiers.contains(KeyModifiers::ALT) {
                    vec![0x1b, 127] // Alt+Backspace (delete word)
                } else {
                    vec![127]
                }
            }
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    vec![0x1b, b'[', b'Z'] // Shift+Tab (backtab)
                } else {
                    vec![b'\t']
                }
            }
            KeyCode::Esc => vec![0x1b],
            KeyCode::Insert => {
                let m = modifier_param(key.modifiers);
                if m == 1 {
                    vec![0x1b, b'[', b'2', b'~']
                } else {
                    format!("\x1b[2;{}~", m).into_bytes()
                }
            }

            // --- Arrow keys with modifier support ---
            KeyCode::Up => {
                let m = modifier_param(key.modifiers);
                if m == 1 {
                    vec![0x1b, b'[', b'A']
                } else {
                    format!("\x1b[1;{}A", m).into_bytes()
                }
            }
            KeyCode::Down => {
                let m = modifier_param(key.modifiers);
                if m == 1 {
                    vec![0x1b, b'[', b'B']
                } else {
                    format!("\x1b[1;{}B", m).into_bytes()
                }
            }
            KeyCode::Right => {
                let m = modifier_param(key.modifiers);
                if m == 1 {
                    vec![0x1b, b'[', b'C']
                } else {
                    format!("\x1b[1;{}C", m).into_bytes()
                }
            }
            KeyCode::Left => {
                let m = modifier_param(key.modifiers);
                if m == 1 {
                    vec![0x1b, b'[', b'D']
                } else {
                    format!("\x1b[1;{}D", m).into_bytes()
                }
            }

            // --- Navigation keys with modifier support ---
            KeyCode::Home => {
                let m = modifier_param(key.modifiers);
                if m == 1 {
                    vec![0x1b, b'[', b'H']
                } else {
                    format!("\x1b[1;{}H", m).into_bytes()
                }
            }
            KeyCode::End => {
                let m = modifier_param(key.modifiers);
                if m == 1 {
                    vec![0x1b, b'[', b'F']
                } else {
                    format!("\x1b[1;{}F", m).into_bytes()
                }
            }
            KeyCode::PageUp => {
                let m = modifier_param(key.modifiers);
                if m == 1 {
                    vec![0x1b, b'[', b'5', b'~']
                } else {
                    format!("\x1b[5;{}~", m).into_bytes()
                }
            }
            KeyCode::PageDown => {
                let m = modifier_param(key.modifiers);
                if m == 1 {
                    vec![0x1b, b'[', b'6', b'~']
                } else {
                    format!("\x1b[6;{}~", m).into_bytes()
                }
            }
            KeyCode::Delete => {
                let m = modifier_param(key.modifiers);
                if m == 1 {
                    vec![0x1b, b'[', b'3', b'~']
                } else {
                    format!("\x1b[3;{}~", m).into_bytes()
                }
            }

            // --- Function keys (F1-F12) with modifier support ---
            KeyCode::F(n) => {
                let m = modifier_param(key.modifiers);
                match n {
                    // F1-F4 use SS3 sequences (no modifier) or CSI with modifier
                    1 => {
                        if m == 1 {
                            vec![0x1b, b'O', b'P']
                        } else {
                            format!("\x1b[1;{}P", m).into_bytes()
                        }
                    }
                    2 => {
                        if m == 1 {
                            vec![0x1b, b'O', b'Q']
                        } else {
                            format!("\x1b[1;{}Q", m).into_bytes()
                        }
                    }
                    3 => {
                        if m == 1 {
                            vec![0x1b, b'O', b'R']
                        } else {
                            format!("\x1b[1;{}R", m).into_bytes()
                        }
                    }
                    4 => {
                        if m == 1 {
                            vec![0x1b, b'O', b'S']
                        } else {
                            format!("\x1b[1;{}S", m).into_bytes()
                        }
                    }
                    // F5-F12 use CSI number ~ sequences
                    5 => {
                        if m == 1 {
                            b"\x1b[15~".to_vec()
                        } else {
                            format!("\x1b[15;{}~", m).into_bytes()
                        }
                    }
                    6 => {
                        if m == 1 {
                            b"\x1b[17~".to_vec()
                        } else {
                            format!("\x1b[17;{}~", m).into_bytes()
                        }
                    }
                    7 => {
                        if m == 1 {
                            b"\x1b[18~".to_vec()
                        } else {
                            format!("\x1b[18;{}~", m).into_bytes()
                        }
                    }
                    8 => {
                        if m == 1 {
                            b"\x1b[19~".to_vec()
                        } else {
                            format!("\x1b[19;{}~", m).into_bytes()
                        }
                    }
                    9 => {
                        if m == 1 {
                            b"\x1b[20~".to_vec()
                        } else {
                            format!("\x1b[20;{}~", m).into_bytes()
                        }
                    }
                    10 => {
                        if m == 1 {
                            b"\x1b[21~".to_vec()
                        } else {
                            format!("\x1b[21;{}~", m).into_bytes()
                        }
                    }
                    11 => {
                        if m == 1 {
                            b"\x1b[23~".to_vec()
                        } else {
                            format!("\x1b[23;{}~", m).into_bytes()
                        }
                    }
                    12 => {
                        if m == 1 {
                            b"\x1b[24~".to_vec()
                        } else {
                            format!("\x1b[24;{}~", m).into_bytes()
                        }
                    }
                    _ => return, // F13+ not commonly used
                }
            }

            // --- BackTab (Shift+Tab reported as separate key by crossterm) ---
            KeyCode::BackTab => vec![0x1b, b'[', b'Z'],

            // Unknown keys — ignore rather than sending garbage
            _ => return,
        };

        if let Ok(mut guard) = self.pty_writer.lock() {
            if let Some(ref mut writer) = *guard {
                let _ = writer.write_all(&bytes);
                let _ = writer.flush();
            }
        }
    }

    pub fn send_interrupt(&mut self) {
        if let Ok(mut guard) = self.pty_writer.lock() {
            if let Some(ref mut writer) = *guard {
                let _ = writer.write_all(&[3]); // Ctrl+C
                let _ = writer.flush();
            }
        }
    }

    pub fn send_focus_event(&mut self, gained: bool) {
        let seq = if gained {
            b"\x1b[I" as &[u8]
        } else {
            b"\x1b[O"
        };
        if let Ok(mut guard) = self.pty_writer.lock() {
            if let Some(ref mut writer) = *guard {
                let _ = writer.write_all(seq);
                let _ = writer.flush();
            }
        }
    }

    /// Acquire a poison-safe lock on the virtual terminal.
    pub fn vterm_lock(&self) -> MutexGuard<'_, VirtualTerminal> {
        lock_or_recover(&self.vterm)
    }

    pub fn scroll_up(&mut self) {
        let mut vt = lock_or_recover(&self.vterm);
        let current = vt.scroll_offset();
        vt.set_scroll_offset(current + 3);
    }

    pub fn scroll_down(&mut self) {
        let mut vt = lock_or_recover(&self.vterm);
        let current = vt.scroll_offset();
        vt.set_scroll_offset(current.saturating_sub(3));
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        if cols == self.last_cols && rows == self.last_rows {
            return;
        }
        self.last_cols = cols;
        self.last_rows = rows;

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
        let mut vt = lock_or_recover(&self.vterm);
        vt.resize(cols as usize, rows as usize);
    }
}

impl Drop for TerminalPane {
    fn drop(&mut self) {
        // PTY will be cleaned up automatically
        self.pty_pair.take();
    }
}

/// Get the current working directory of a process by PID.
/// Uses macOS `proc_pidinfo` API or Linux `/proc/PID/cwd`.
#[cfg(target_os = "macos")]
fn get_process_cwd(pid: u32) -> Option<PathBuf> {
    use std::ffi::{c_int, c_void};
    use std::mem;

    const PROC_PIDVNODEPATHINFO: c_int = 9;
    const MAXPATHLEN: usize = 1024;

    #[repr(C)]
    struct VnodeInfoPath {
        // struct vnode_info (see Darwin sys/proc_info.h: vnode_info is 152 bytes)
        _vip_vi: [u8; 152],
        vip_path: [u8; MAXPATHLEN],
    }

    #[repr(C)]
    struct ProcVnodePathInfo {
        pvi_cdir: VnodeInfoPath,
        _pvi_rdir: VnodeInfoPath,
    }

    extern "C" {
        fn proc_pidinfo(
            pid: c_int,
            flavor: c_int,
            arg: u64,
            buffer: *mut c_void,
            buffersize: c_int,
        ) -> c_int;
    }

    unsafe {
        let mut info: ProcVnodePathInfo = mem::zeroed();
        let size = mem::size_of::<ProcVnodePathInfo>() as c_int;

        let ret = proc_pidinfo(
            pid as c_int,
            PROC_PIDVNODEPATHINFO,
            0,
            &mut info as *mut _ as *mut c_void,
            size,
        );

        if ret != size {
            return None;
        }

        let path_bytes = &info.pvi_cdir.vip_path;
        let len = path_bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(MAXPATHLEN);
        let path_str = std::str::from_utf8(&path_bytes[..len]).ok()?;

        if path_str.is_empty() {
            None
        } else {
            Some(PathBuf::from(path_str))
        }
    }
}

#[cfg(target_os = "linux")]
fn get_process_cwd(pid: u32) -> Option<PathBuf> {
    std::fs::read_link(format!("/proc/{}/cwd", pid)).ok()
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn get_process_cwd(_pid: u32) -> Option<PathBuf> {
    None
}
