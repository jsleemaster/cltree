mod app;
mod event;
mod terminal;
mod tree;
mod ui;
pub mod vterm;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::path::PathBuf;

use app::App;
use event::EventHandler;

struct Args {
    path: PathBuf,
    tree_width: u16,
    show_hidden: bool,
    depth: usize,
    claude_args: Vec<String>,
}

/// claude-explorer 자체 플래그만 꺼내고, 나머지는 모두 Claude Code CLI로 전달
fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().skip(1).collect();

    let mut path = PathBuf::from(".");
    let mut tree_width: u16 = 30;
    let mut show_hidden = false;
    let mut depth: usize = 10;
    let mut claude_args = Vec::new();

    // Known flags that take a value
    let value_flags: &[&[&str]] = &[
        &["-p", "--path"],
        &["-w", "--tree-width"],
        &["-d", "--depth"],
    ];

    let mut i = 0;
    while i < raw.len() {
        let arg = &raw[i];

        // Handle --help / --version ourselves
        if arg == "-h" || arg == "--help" {
            eprintln!(
                "A TUI file explorer for Claude Code CLI\n\n\
                 Usage: claude-explorer [OPTIONS] [CLAUDE_ARGS...]\n\n\
                 Options:\n\
                 \x20 -p, --path <PATH>         Working directory [default: .]\n\
                 \x20 -w, --tree-width <WIDTH>   Tree panel width %% (10-50) [default: 30]\n\
                 \x20 -a, --show-hidden          Show hidden files\n\
                 \x20 -d, --depth <DEPTH>        Max tree depth [default: 10]\n\
                 \x20 -h, --help                 Print help\n\
                 \x20 -V, --version              Print version\n\n\
                 All other arguments are passed through to Claude Code CLI.\n\
                 Example: claude-explorer --resume\n\
                 Example: claude-explorer -p /my/project --continue"
            );
            std::process::exit(0);
        }
        if arg == "-V" || arg == "--version" {
            eprintln!("claude-explorer {}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }

        // Check if it's one of our value flags (e.g. -p /path or --path=/path)
        let mut matched_value_flag = false;
        for names in value_flags {
            // Handle --flag=value form
            for name in *names {
                if let Some(val) = arg.strip_prefix(&format!("{name}=")) {
                    match *name {
                        "-p" | "--path" => path = PathBuf::from(val),
                        "-w" | "--tree-width" => tree_width = val.parse().unwrap_or(30),
                        "-d" | "--depth" => depth = val.parse().unwrap_or(10),
                        _ => {}
                    }
                    matched_value_flag = true;
                    break;
                }
            }
            if matched_value_flag {
                break;
            }

            // Handle --flag value form
            if names.contains(&arg.as_str()) {
                let val = raw.get(i + 1).cloned().unwrap_or_default();
                match names[1] {
                    "--path" => path = PathBuf::from(&val),
                    "--tree-width" => tree_width = val.parse().unwrap_or(30),
                    "--depth" => depth = val.parse().unwrap_or(10),
                    _ => {}
                }
                i += 2;
                matched_value_flag = true;
                break;
            }
        }

        if matched_value_flag {
            if !arg.contains('=') {
                continue; // already incremented i by 2
            }
            i += 1;
            continue;
        }

        // Boolean flag
        if arg == "-a" || arg == "--show-hidden" {
            show_hidden = true;
            i += 1;
            continue;
        }

        // Everything else goes to Claude Code
        claude_args.push(arg.clone());
        i += 1;
    }

    Args {
        path,
        tree_width,
        show_hidden,
        depth,
        claude_args,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_args();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(
        args.path,
        args.tree_width,
        args.show_hidden,
        args.depth,
        args.claude_args,
    )?;

    // Create event handler with file watching enabled for the tree root
    let watch_path = Some(app.tree.root_path().to_path_buf());
    let event_handler = EventHandler::new(250, watch_path);

    // Run the app
    let result = run_app(&mut terminal, &mut app, event_handler).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    mut event_handler: EventHandler,
) -> Result<()> {
    loop {
        // Draw UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Handle events
        match event_handler.next().await? {
            event::Event::Tick => {
                if app.tick() {
                    return Ok(());
                }
            }
            event::Event::Key(key_event) => {
                if app.handle_key(key_event) {
                    return Ok(());
                }
            }
            event::Event::Mouse(mouse_event) => {
                app.handle_mouse(mouse_event);
            }
            event::Event::Resize(width, height) => {
                app.handle_resize(width, height);
            }
            event::Event::FileChange(path) => {
                app.handle_file_change(path);
            }
        }
    }
}
