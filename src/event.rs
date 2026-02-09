use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
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
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    _tx: mpsc::UnboundedSender<Event>,
    // Keep the debouncer alive to prevent it from being dropped
    _debouncer: Option<Debouncer<notify::RecommendedWatcher>>,
}

impl EventHandler {
    pub fn new(tick_rate: u64, watch_path: Option<PathBuf>) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (tx, rx) = mpsc::unbounded_channel();
        let tx_clone = tx.clone();

        // Spawn event polling task
        tokio::spawn(async move {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            if tx_clone.send(Event::Key(key)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Mouse(mouse)) => {
                            if tx_clone.send(Event::Mouse(mouse)).is_err() {
                                break;
                            }
                        }
                        Ok(CrosstermEvent::Resize(width, height)) => {
                            if tx_clone.send(Event::Resize(width, height)).is_err() {
                                break;
                            }
                        }
                        Ok(_) => {}
                        Err(_) => break,
                    }
                } else {
                    // Send tick event
                    if tx_clone.send(Event::Tick).is_err() {
                        break;
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
            _tx: tx,
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
