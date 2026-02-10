use anyhow::Result;
use crossterm::event::{EventStream, KeyEvent, MouseEvent};
use futures::StreamExt;
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, DebouncedEventKind, Debouncer};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    FileChange(PathBuf),
    PtyOutput,
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    // Keep the debouncer alive to prevent it from being dropped
    _debouncer: Option<Debouncer<notify::RecommendedWatcher>>,
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
                }
            }
        });

        // Initialize file watcher if watch_path is provided
        let debouncer = watch_path.and_then(|path| {
            let fs_tx = tx.clone();
            let mut debouncer = new_debouncer(
                Duration::from_millis(300),
                move |result: DebounceEventResult| {
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
                },
            )
            .ok()?;

            debouncer
                .watcher()
                .watch(&path, RecursiveMode::Recursive)
                .ok()?;

            Some(debouncer)
        });

        Self {
            rx,
            _debouncer: debouncer,
        }
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Event channel closed"))
    }
}
