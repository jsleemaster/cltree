use anyhow::Result;
use crossterm::event::{EventStream, KeyEvent, MouseEvent};
use futures::StreamExt;
use notify::{Config as NotifyConfig, PollWatcher, RecursiveMode};
use notify_debouncer_mini::{
    new_debouncer_opt, Config as DebounceConfig, DebounceEventResult, DebouncedEventKind, Debouncer,
};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

// Tuned for faster UI reflection while keeping duplicate event noise manageable.
const WATCH_POLL_INTERVAL_MS: u64 = 75;
const WATCH_DEBOUNCE_TIMEOUT_MS: u64 = 50;

#[derive(Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    Resize(u16, u16),
    FocusGained,
    FocusLost,
    FileChange(PathBuf),
    PtyOutput,
    Signal,
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    // Keep the debouncer alive to prevent it from being dropped
    debouncer: Option<Debouncer<PollWatcher>>,
    watched_path: Option<PathBuf>,
}

impl EventHandler {
    pub fn new(
        tick_rate: u64,
        watch_path: Option<PathBuf>,
        pty_rx: mpsc::UnboundedReceiver<()>,
    ) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn async event loop using EventStream + select!
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let mut crossterm_events = EventStream::new();
            let mut pty_rx = pty_rx;
            let mut tick_interval = tokio::time::interval(tick_rate);

            loop {
                tokio::select! {
                    // Crossterm terminal events (key, mouse, resize)
                    maybe_event = crossterm_events.next() => {
                        #[allow(unreachable_patterns)]
                        match maybe_event {
                            Some(Ok(crossterm::event::Event::Key(key))) => {
                                if tx_clone.send(Event::Key(key)).is_err() {
                                    break;
                                }
                            }
                            Some(Ok(crossterm::event::Event::Mouse(mouse))) => {
                                if tx_clone.send(Event::Mouse(mouse)).is_err() {
                                    break;
                                }
                            }
                            Some(Ok(crossterm::event::Event::Resize(w, h))) => {
                                if tx_clone.send(Event::Resize(w, h)).is_err() {
                                    break;
                                }
                            }
                            Some(Ok(crossterm::event::Event::FocusGained)) => {
                                if tx_clone.send(Event::FocusGained).is_err() {
                                    break;
                                }
                            }
                            Some(Ok(crossterm::event::Event::FocusLost)) => {
                                if tx_clone.send(Event::FocusLost).is_err() {
                                    break;
                                }
                            }
                            Some(Ok(crossterm::event::Event::Paste(text))) => {
                                if tx_clone.send(Event::Paste(text)).is_err() {
                                    break;
                                }
                            }
                            Some(Ok(_)) => {}
                            Some(Err(_)) => break,
                            None => break,
                        }
                    }
                    // PTY output notification — triggers immediate redraw
                    maybe_pty = pty_rx.recv() => {
                        match maybe_pty {
                            Some(()) => {
                                // Drain any additional pending notifications to coalesce redraws
                                while pty_rx.try_recv().is_ok() {}
                                if tx_clone.send(Event::PtyOutput).is_err() {
                                    break;
                                }
                            }
                            None => {
                                // PTY channel closed (process exited), keep running for other events
                            }
                        }
                    }
                    // Periodic tick for housekeeping (process exit check, etc.)
                    _ = tick_interval.tick() => {
                        if tx_clone.send(Event::Tick).is_err() {
                            break;
                        }
                    }
                    // SIGTERM handler (Unix only)
                    _ = async {
                        #[cfg(unix)]
                        {
                            let mut sig = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("failed to register SIGTERM handler");
                            sig.recv().await;
                        }
                        #[cfg(not(unix))]
                        {
                            std::future::pending::<()>().await;
                        }
                    } => {
                        let _ = tx_clone.send(Event::Signal);
                    }
                }
            }
        });

        let mut handler = Self {
            rx,
            debouncer: Self::build_debouncer(tx.clone()).ok(),
            watched_path: None,
        };
        handler.update_watch_path(watch_path);
        handler
    }

    fn build_debouncer(
        fs_tx: mpsc::UnboundedSender<Event>,
    ) -> notify::Result<Debouncer<PollWatcher>> {
        // Use PollWatcher explicitly because FSEvent can miss events in sandboxed/virtualized environments.
        let notify_cfg = NotifyConfig::default()
            .with_poll_interval(Duration::from_millis(WATCH_POLL_INTERVAL_MS));
        let debounce_cfg = DebounceConfig::default()
            .with_timeout(Duration::from_millis(WATCH_DEBOUNCE_TIMEOUT_MS))
            .with_notify_config(notify_cfg);

        new_debouncer_opt::<_, PollWatcher>(debounce_cfg, move |result: DebounceEventResult| {
            if let Ok(events) = result {
                for fs_event in events {
                    if matches!(
                        fs_event.kind,
                        DebouncedEventKind::Any | DebouncedEventKind::AnyContinuous
                    ) {
                        let _ = fs_tx.send(Event::FileChange(fs_event.path));
                    }
                }
            }
        })
    }

    pub fn update_watch_path(&mut self, watch_path: Option<PathBuf>) {
        let normalized = watch_path.map(|path| path.canonicalize().unwrap_or(path));
        if self.watched_path == normalized {
            return;
        }

        let Some(debouncer) = self.debouncer.as_mut() else {
            return;
        };

        if let Some(old) = self.watched_path.take() {
            let _ = debouncer.watcher().unwatch(&old);
        }

        if let Some(path) = normalized {
            if debouncer
                .watcher()
                .watch(&path, RecursiveMode::Recursive)
                .is_ok()
            {
                self.watched_path = Some(path);
            }
        }
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Event channel closed"))
    }
}
