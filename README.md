# cltree

[![GitHub Release](https://img.shields.io/github/v/release/jsleemaster/cltree)](https://github.com/jsleemaster/cltree/releases)
[![npm](https://img.shields.io/npm/v/cltree)](https://www.npmjs.com/package/cltree)
[![Homebrew](https://img.shields.io/badge/homebrew-available-blue)](https://github.com/jsleemaster/homebrew-tap)
[![GitHub Stars](https://img.shields.io/github/stars/jsleemaster/cltree)](https://github.com/jsleemaster/cltree/stargazers)
[![GitHub Issues](https://img.shields.io/github/issues/jsleemaster/cltree)](https://github.com/jsleemaster/cltree/issues)
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

## Features

- **Split-pane TUI**: File tree on the right, Claude Code on the left
- **Passive file tree**: Always-expanded, read-only project structure display
- **CWD tracking**: Highlights Claude Code's current working directory with a ● marker
- **OSC 7 + vterm detection**: Automatically detects directory changes via escape sequences
- **gitignore support**: Respects `.gitignore` patterns
- **File icons**: Visual indicators for different file types
- **Zero interference**: All keystrokes are forwarded directly to Claude Code

## Installation

### npm / bun

```bash
npm install -g cltree
# or
bun install -g cltree
```

### Homebrew (macOS / Linux)

```bash
brew install jsleemaster/tap/cltree
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

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+Q` | Quit |

All other keystrokes are passed directly to Claude Code.

## Configuration

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

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Claude Code](https://claude.com) - AI coding assistant by Anthropic
- Inspired by ranger, nnn, and other terminal file managers
