use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, DebouncedEventKind};

/// Test that notify debouncer detects file creation and sends events through mpsc channel
#[tokio::test]
async fn test_file_watcher_detects_creation() {
    let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    // Canonicalize to resolve symlinks (e.g. /var -> /private/var on macOS)
    let watch_path = tmp_dir
        .path()
        .canonicalize()
        .expect("Failed to canonicalize");

    let (tx, mut rx) = mpsc::unbounded_channel::<PathBuf>();

    // Setup watcher (same pattern as EventHandler)
    let mut debouncer = new_debouncer(
        Duration::from_millis(100),
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
    .expect("Failed to create debouncer");

    debouncer
        .watcher()
        .watch(&watch_path, RecursiveMode::Recursive)
        .expect("Failed to watch path");

    // Create a file
    let test_file = watch_path.join("test_file.txt");
    fs::write(&test_file, "hello").expect("Failed to write test file");

    // Wait for event (with timeout)
    let event = tokio::time::timeout(Duration::from_secs(3), rx.recv()).await;

    assert!(
        event.is_ok(),
        "Should receive file change event within timeout"
    );
    let received_path = event.unwrap().expect("Channel should not be closed");
    // The event path should be within the watched directory
    assert!(
        received_path.starts_with(&watch_path),
        "Event path {:?} should be within watch dir {:?}",
        received_path,
        watch_path
    );
}

/// Test that notify debouncer detects file deletion
#[tokio::test]
async fn test_file_watcher_detects_deletion() {
    let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let watch_path = tmp_dir
        .path()
        .canonicalize()
        .expect("Failed to canonicalize");

    // Create file before starting watcher
    let test_file = watch_path.join("to_delete.txt");
    fs::write(&test_file, "delete me").expect("Failed to write file");

    let (tx, mut rx) = mpsc::unbounded_channel::<PathBuf>();

    let mut debouncer = new_debouncer(
        Duration::from_millis(100),
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
    .expect("Failed to create debouncer");

    debouncer
        .watcher()
        .watch(&watch_path, RecursiveMode::Recursive)
        .expect("Failed to watch path");

    // Small delay to let watcher settle
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Delete the file
    fs::remove_file(&test_file).expect("Failed to remove file");

    // Wait for event
    let event = tokio::time::timeout(Duration::from_secs(3), rx.recv()).await;

    assert!(
        event.is_ok(),
        "Should receive file deletion event within timeout"
    );
}

/// Test that notify debouncer detects changes in subdirectories (recursive)
#[tokio::test]
async fn test_file_watcher_recursive() {
    let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let watch_path = tmp_dir
        .path()
        .canonicalize()
        .expect("Failed to canonicalize");
    let sub_dir = watch_path.join("subdir");
    fs::create_dir(&sub_dir).expect("Failed to create subdir");

    let (tx, mut rx) = mpsc::unbounded_channel::<PathBuf>();

    let mut debouncer = new_debouncer(
        Duration::from_millis(100),
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
    .expect("Failed to create debouncer");

    debouncer
        .watcher()
        .watch(&watch_path, RecursiveMode::Recursive)
        .expect("Failed to watch path");

    // Small delay to let watcher settle
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create file in subdirectory
    let nested_file = sub_dir.join("nested.txt");
    fs::write(&nested_file, "nested content").expect("Failed to write nested file");

    // Wait for event
    let event = tokio::time::timeout(Duration::from_secs(3), rx.recv()).await;

    assert!(
        event.is_ok(),
        "Should detect file changes in subdirectories"
    );
    let received_path = event.unwrap().expect("Channel should not be closed");
    assert!(
        received_path.starts_with(&watch_path),
        "Event path should be within watch dir"
    );
}
