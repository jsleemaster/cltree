# 🦀 Claude Explorer

[![GitHub Release](https://img.shields.io/github/v/release/jsleemaster/claude-explorer)](https://github.com/jsleemaster/claude-explorer/releases)
[![npm](https://img.shields.io/npm/v/claude-explorer)](https://www.npmjs.com/package/claude-explorer)
[![Crates.io Version](https://img.shields.io/crates/v/claude-explorer)](https://crates.io/crates/claude-explorer)
[![Crates.io Downloads](https://img.shields.io/crates/d/claude-explorer)](https://crates.io/crates/claude-explorer)
[![Homebrew](https://img.shields.io/badge/homebrew-available-blue)](https://github.com/jsleemaster/homebrew-tap)
[![GitHub Stars](https://img.shields.io/github/stars/jsleemaster/claude-explorer)](https://github.com/jsleemaster/claude-explorer/stargazers)
[![GitHub Issues](https://img.shields.io/github/issues/jsleemaster/claude-explorer)](https://github.com/jsleemaster/claude-explorer/issues)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A terminal-based file explorer designed to work alongside Claude Code CLI. View your project structure in a split-pane TUI while using Claude Code.

```
┌─────────────────────────────────────────────┬──────────────────────┐
│                                             │ 📂 my-project        │
│  Claude Code                                │ ├── 📁 src           │
│                                             │ │   ├── 🦀 main.rs   │
│  > Help me refactor this function           │ │   ├── 🦀 app.rs    │
│                                             │ │   └── 📁 ui        │
│  I'll analyze the code structure...         │ ├── 📋 Cargo.toml    │
│                                             │ ├── 📖 README.md     │
│                                             │ └── 📄 .gitignore    │
│                                             │                      │
│                                             │ ● src/ui             │
└─────────────────────────────────────────────┴──────────────────────┘
```

## ✨ Features

- **Split-pane TUI**: File tree on the right, Claude Code on the left
- **Passive file tree**: Always-expanded, read-only project structure display
- **CWD tracking**: Highlights Claude Code's current working directory with a ● marker
- **OSC 7 + vterm detection**: Automatically detects directory changes via escape sequences
- **gitignore support**: Respects `.gitignore` patterns
- **File icons**: Visual indicators for different file types
- **Zero interference**: All keystrokes are forwarded directly to Claude Code

## 📦 Installation

### npm / bun

```bash
npm install -g claude-explorer
# or
bun install -g claude-explorer
```

### Homebrew (macOS / Linux)

```bash
brew install jsleemaster/tap/claude-explorer
```

### From crates.io

```bash
cargo install claude-explorer
```

### From source

```bash
git clone https://github.com/jsleemaster/claude-explorer.git
cd claude-explorer
cargo install --path .
```

## 🚀 Usage

```bash
# Start in current directory
claude-explorer

# Start in specific directory
claude-explorer --path /path/to/project

# Adjust tree width (10-50%)
claude-explorer --tree-width 25

# Show hidden files
claude-explorer --show-hidden
```

## ⌨️ Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+Q` | Quit |

All other keystrokes are passed directly to Claude Code.

## 🔧 Configuration

### Command line options

```
Options:
  -p, --path <PATH>          Working directory [default: .]
  -w, --tree-width <WIDTH>   Tree panel width percentage (10-50) [default: 30]
  -a, --show-hidden          Show hidden files
  -d, --depth <DEPTH>        Max tree depth [default: 10]
  -h, --help                 Print help
  -V, --version              Print version
```

## 🛠️ Development

```bash
# Clone
git clone https://github.com/jsleemaster/claude-explorer.git
cd claude-explorer

# Run in development
cargo run

# Run tests
cargo test

# Build release
cargo build --release
```

## 📋 Requirements

- Rust 1.70+
- Claude Code CLI installed and in PATH
- Terminal with UTF-8 and true color support

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Claude Code](https://claude.com) - AI coding assistant by Anthropic
- Inspired by ranger, nnn, and other terminal file managers
