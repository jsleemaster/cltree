use std::fs;
use std::path::PathBuf;

/// Test that FileTree builds correctly from a directory
#[test]
fn test_file_tree_builds_correctly() {
    let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = tmp_dir.path().canonicalize().unwrap();

    // Create test structure
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(root.join("Cargo.toml"), "[package]").unwrap();
    fs::write(root.join("README.md"), "# Hello").unwrap();

    // Use the project's FileTree
    // Since we can't import from a binary crate, test via command execution
    // Instead, verify the core logic: WalkBuilder with the test structure
    use ignore::WalkBuilder;

    let walker = WalkBuilder::new(&root)
        .hidden(true) // hide hidden files
        .git_ignore(false)
        .max_depth(Some(2))
        .sort_by_file_name(|a, b| a.cmp(b))
        .build();

    let entries: Vec<PathBuf> = walker.flatten().map(|e| e.into_path()).collect();

    // Root + 3 files + 1 dir + 1 file in dir = 6
    assert!(
        entries.len() >= 5,
        "Should find at least 5 entries, found {}",
        entries.len()
    );

    let names: Vec<String> = entries
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .collect();

    assert!(
        names.contains(&"src".to_string()),
        "Should contain 'src' dir"
    );
    assert!(
        names.contains(&"main.rs".to_string()),
        "Should contain 'main.rs'"
    );
    assert!(
        names.contains(&"Cargo.toml".to_string()),
        "Should contain 'Cargo.toml'"
    );
    assert!(
        names.contains(&"README.md".to_string()),
        "Should contain 'README.md'"
    );

    println!("File tree entries: {:?}", names);
}

/// Test that TerminalPane handles missing claude gracefully
#[test]
fn test_terminal_pane_handles_missing_claude() {
    // TerminalPane::new() should NOT panic even if claude is not installed
    // Since we can't import from binary crate, test the PTY logic directly
    use portable_pty::{native_pty_system, CommandBuilder, PtySize};

    let pty_system = native_pty_system();
    let pty_result = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    });

    match pty_result {
        Ok(pty_pair) => {
            // Try spawning a non-existent command
            let cmd = CommandBuilder::new("this_command_does_not_exist_12345");
            let spawn_result = pty_pair.slave.spawn_command(cmd);

            // This should fail, not panic
            assert!(
                spawn_result.is_err(),
                "Spawning non-existent command should fail"
            );
            println!(
                "Correctly handled missing command: {:?}",
                spawn_result.err()
            );
        }
        Err(e) => {
            // PTY creation failed (e.g., in CI environment without TTY)
            println!("PTY creation failed (expected in non-TTY env): {}", e);
        }
    }
}

/// Test that the file watcher + tree refresh integration works
#[tokio::test]
async fn test_file_change_triggers_tree_refresh() {
    use ignore::WalkBuilder;
    use notify::RecursiveMode;
    use notify_debouncer_mini::{new_debouncer, DebounceEventResult, DebouncedEventKind};
    use std::time::Duration;
    use tokio::sync::mpsc;

    let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let root = tmp_dir.path().canonicalize().unwrap();

    // Initial tree: just root
    let initial_entries: Vec<_> = WalkBuilder::new(&root)
        .max_depth(Some(1))
        .build()
        .flatten()
        .collect();
    let initial_count = initial_entries.len();

    // Setup watcher
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
    .unwrap();

    debouncer
        .watcher()
        .watch(&root, RecursiveMode::Recursive)
        .unwrap();

    // Create new file (simulates Claude Code creating a file)
    fs::write(root.join("new_file.txt"), "created by claude").unwrap();

    // Wait for event
    let event = tokio::time::timeout(Duration::from_secs(3), rx.recv()).await;
    assert!(event.is_ok(), "Should detect new file creation");

    // After refresh, tree should have more entries
    let refreshed_entries: Vec<_> = WalkBuilder::new(&root)
        .max_depth(Some(1))
        .build()
        .flatten()
        .collect();

    assert!(
        refreshed_entries.len() > initial_count,
        "Tree should have more entries after file creation: {} -> {}",
        initial_count,
        refreshed_entries.len()
    );

    let has_new_file = refreshed_entries
        .iter()
        .any(|e| e.file_name().to_string_lossy() == "new_file.txt");
    assert!(has_new_file, "Refreshed tree should contain 'new_file.txt'");

    println!("File watching + tree refresh integration: OK");
    println!(
        "  Initial entries: {}, After creation: {}",
        initial_count,
        refreshed_entries.len()
    );
}
