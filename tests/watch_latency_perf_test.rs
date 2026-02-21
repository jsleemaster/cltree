use notify::{Config as NotifyConfig, RecursiveMode};
use notify_debouncer_mini::{
    new_debouncer_opt, Config as DebounceConfig, DebounceEventResult, DebouncedEventKind,
};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::mpsc;

const WATCH_POLL_INTERVAL_MS: u64 = 75;
const WATCH_DEBOUNCE_TIMEOUT_MS: u64 = 50;

fn percentile_index(len: usize, quantile: f64) -> usize {
    assert!(len > 0);
    let q = quantile.clamp(0.0, 1.0);
    ((q * ((len - 1) as f64)).round() as usize).min(len - 1)
}

#[tokio::test]
#[ignore = "manual perf probe; run with cargo test --test watch_latency_perf_test -- --ignored --nocapture"]
async fn measure_poll_watcher_file_create_latency() {
    const SAMPLES: usize = 30;
    let temp_dir = TempDir::new().expect("failed to create temp directory");
    let watch_root = temp_dir.path().to_path_buf();

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
    .expect("failed to create poll watcher debouncer");

    debouncer
        .watcher()
        .watch(&watch_root, RecursiveMode::Recursive)
        .expect("failed to start recursive watch");

    // Let first poll cycle settle before measuring.
    tokio::time::sleep(Duration::from_millis(700)).await;

    let mut latencies_ms: Vec<f64> = Vec::with_capacity(SAMPLES);
    for i in 0..SAMPLES {
        let file_path = watch_root.join(format!("latency_{:02}.txt", i));
        let started = Instant::now();
        std::fs::write(&file_path, b"x").expect("failed to write sample file");

        let deadline = tokio::time::sleep(Duration::from_secs(5));
        tokio::pin!(deadline);

        let observed = loop {
            tokio::select! {
                _ = &mut deadline => {
                    panic!("timed out waiting for event for {:?}", file_path);
                }
                maybe_path = rx.recv() => {
                    let path = maybe_path.expect("watch event channel closed");
                    if path == file_path {
                        break started.elapsed();
                    }
                }
            }
        };

        latencies_ms.push(observed.as_secs_f64() * 1000.0);
        tokio::time::sleep(Duration::from_millis(120)).await;
    }

    latencies_ms.sort_by(f64::total_cmp);
    let avg = latencies_ms.iter().sum::<f64>() / latencies_ms.len() as f64;
    let min = latencies_ms[0];
    let p50 = latencies_ms[percentile_index(latencies_ms.len(), 0.50)];
    let p90 = latencies_ms[percentile_index(latencies_ms.len(), 0.90)];
    let p95 = latencies_ms[percentile_index(latencies_ms.len(), 0.95)];
    let max = latencies_ms[latencies_ms.len() - 1];

    eprintln!(
        "poll watcher create-latency (ms): poll={} debounce={} samples={} min={:.1} p50={:.1} p90={:.1} p95={:.1} max={:.1} avg={:.1}",
        WATCH_POLL_INTERVAL_MS, WATCH_DEBOUNCE_TIMEOUT_MS, SAMPLES, min, p50, p90, p95, max, avg
    );
}
