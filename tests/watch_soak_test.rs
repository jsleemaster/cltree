use notify::{Config as NotifyConfig, RecursiveMode};
use notify_debouncer_mini::{
    new_debouncer_opt, Config as DebounceConfig, DebounceEventResult, DebouncedEventKind,
};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::mpsc;

const WATCH_POLL_INTERVAL_MS: u64 = 75;
const WATCH_DEBOUNCE_TIMEOUT_MS: u64 = 50;

#[tokio::test]
#[ignore = "manual soak probe; run with SOAK_SECONDS=180 cargo test --test watch_soak_test -- --ignored --nocapture"]
async fn soak_poll_watcher_for_event_misses() {
    let soak_seconds: u64 = std::env::var("SOAK_SECONDS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(120);
    let op_interval = Duration::from_millis(110);

    let tmp = TempDir::new().expect("failed to create temp dir");
    let root = tmp.path().to_path_buf();
    let (tx, mut rx) = mpsc::unbounded_channel();

    let notify_cfg =
        NotifyConfig::default().with_poll_interval(Duration::from_millis(WATCH_POLL_INTERVAL_MS));
    let debounce_cfg = DebounceConfig::default()
        .with_timeout(Duration::from_millis(WATCH_DEBOUNCE_TIMEOUT_MS))
        .with_notify_config(notify_cfg);

    let mut debouncer = new_debouncer_opt::<_, notify::PollWatcher>(
        debounce_cfg,
        move |result: DebounceEventResult| {
            if let Ok(events) = result {
                for fs_event in events {
                    if matches!(
                        fs_event.kind,
                        DebouncedEventKind::Any | DebouncedEventKind::AnyContinuous
                    ) {
                        let _ = tx.send(fs_event.path);
                    }
                }
            }
        },
    )
    .expect("failed to create poll watcher");

    debouncer
        .watcher()
        .watch(&root, RecursiveMode::Recursive)
        .expect("failed to watch soak root");

    // Warm-up one cycle.
    tokio::time::sleep(Duration::from_millis(300)).await;

    let deadline = Instant::now() + Duration::from_secs(soak_seconds);
    let mut iteration: u64 = 0;
    let mut notified_iterations: u64 = 0;
    let mut total_events: u64 = 0;
    let mut max_silent_gap = Duration::ZERO;
    let mut last_event_at = Instant::now();

    while Instant::now() < deadline {
        let file_a = root.join(format!("soak_{iteration}_a.txt"));
        let file_b = root.join(format!("soak_{iteration}_b.txt"));

        std::fs::write(&file_a, b"x").expect("write failed");
        std::fs::rename(&file_a, &file_b).expect("rename failed");
        std::fs::remove_file(&file_b).expect("remove failed");

        let drain_until = Instant::now() + Duration::from_millis(220);
        let mut iteration_notified = false;
        loop {
            match tokio::time::timeout(Duration::from_millis(25), rx.recv()).await {
                Ok(Some(_path)) => {
                    total_events += 1;
                    iteration_notified = true;
                    let now = Instant::now();
                    let gap = now.saturating_duration_since(last_event_at);
                    if gap > max_silent_gap {
                        max_silent_gap = gap;
                    }
                    last_event_at = now;
                    if Instant::now() >= drain_until {
                        break;
                    }
                }
                Ok(None) => panic!("watch channel closed unexpectedly"),
                Err(_) => {
                    if Instant::now() >= drain_until {
                        break;
                    }
                }
            }
        }

        if iteration_notified {
            notified_iterations += 1;
        }

        iteration += 1;
        tokio::time::sleep(op_interval).await;
    }

    // Final drain window
    let final_deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < final_deadline {
        if let Ok(Some(_path)) = tokio::time::timeout(Duration::from_millis(30), rx.recv()).await {
            total_events += 1;
            let now = Instant::now();
            let gap = now.saturating_duration_since(last_event_at);
            if gap > max_silent_gap {
                max_silent_gap = gap;
            }
            last_event_at = now;
        }
    }

    let iteration_coverage = if iteration == 0 {
        1.0
    } else {
        notified_iterations as f64 / iteration as f64
    };
    eprintln!(
        "watch soak summary: duration={}s iterations={} notified_iterations={} iteration_coverage={:.3} total_events={} max_silent_gap_ms={:.0}",
        soak_seconds,
        iteration,
        notified_iterations,
        iteration_coverage,
        total_events,
        max_silent_gap.as_secs_f64() * 1000.0
    );

    assert!(
        iteration_coverage >= 0.25,
        "iteration notification coverage too low: {} / {} ({:.3})",
        notified_iterations,
        iteration,
        iteration_coverage
    );
    assert!(
        max_silent_gap <= Duration::from_secs(3),
        "max silent gap too high: {:.3}s",
        max_silent_gap.as_secs_f64()
    );
}
