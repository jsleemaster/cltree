# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-02-10

### Removed

- Focus switching between panes (Tab key)
- Interactive file tree navigation (j/k/g/G/h/l keys)
- File search functionality (/)
- Help popup (? / F1)
- @path insertion on Enter
- Directory cd on Enter

### Added

- CWD marker (●) showing Claude Code's current working directory in the file tree
- OSC 7 escape sequence support for CWD detection
- vterm buffer scanning as fallback CWD detection
- Debounced tree refresh on CWD changes
- Enhanced key handling — all keystrokes forwarded to terminal

### Changed

- File tree is now always fully expanded (read-only passive display)
- All key input is passed directly to Claude Code terminal
- Event loop rewritten with tokio select! for improved responsiveness

## [0.1.0] - 2025-02-05

### Added

- Initial release
- Split-pane TUI with file tree on the right, Claude Code terminal on the left
- File tree explorer with gitignore support
- PTY terminal integration for Claude Code CLI
- Interactive file navigation with expand/collapse directories
- Quick file reference: press Enter to insert `@path` in Claude Code
- Directory navigation: press Enter on directory to `cd` into it
- File search functionality with `/` command
- Vim-style keyboard navigation (`j`/`k`, `g`/`G`, `h`/`l`)
- File type icons for visual distinction
- Hidden files toggle with `.` key
- Configurable tree panel width (10-50%)
- Command line options for path, tree width, hidden files, and depth
- Help popup with `?` or `F1`

[Unreleased]: https://github.com/jsleemaster/claude-explorer/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/jsleemaster/claude-explorer/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jsleemaster/claude-explorer/releases/tag/v0.1.0
