# cltree

A terminal-based file explorer designed to work alongside Claude Code CLI. View your project structure in a split-pane TUI while using Claude Code.

## Installation

```bash
npm install -g cltree
# or
bun install -g cltree
```

## Usage

```bash
# Start in current directory
cltree

# Start in specific directory
cltree --path /path/to/project

# Adjust tree width (10-50%)
cltree --tree-width 25

# Show hidden files
cltree --show-hidden
```

## How it works

This package downloads the pre-built native binary for your platform from [GitHub Releases](https://github.com/jsleemaster/cltree/releases) during installation. No Rust toolchain required.

### Supported platforms

- macOS (Apple Silicon / Intel)
- Linux (x86_64 / ARM64)
- Windows (x86_64)

## License

MIT - see [LICENSE](https://github.com/jsleemaster/cltree/blob/main/LICENSE)
