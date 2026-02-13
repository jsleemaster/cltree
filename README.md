# cltree

[![GitHub Release](https://img.shields.io/github/v/release/jsleemaster/cltree)](https://github.com/jsleemaster/cltree/releases)
[![npm](https://img.shields.io/npm/v/cltree)](https://www.npmjs.com/package/cltree)
[![GitHub Stars](https://img.shields.io/github/stars/jsleemaster/cltree)](https://github.com/jsleemaster/cltree/stargazers)
[![GitHub Issues](https://img.shields.io/github/issues/jsleemaster/cltree)](https://github.com/jsleemaster/cltree/issues)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Roadmap](https://img.shields.io/badge/roadmap-view-blue)](https://github.com/jsleemaster/cltree/issues?q=label%3Aroadmap)

<p align="center">
  <a href="README.md">English</a> |
  <a href="README.ko.md">한국어</a> |
  <a href="README.ja.md">日本語</a> |
  <a href="README.zh-CN.md">中文</a>
</p>

A terminal-based file explorer designed to work alongside Claude Code CLI. View your project structure in a split-pane TUI while using Claude Code.

![cltree demo](demo.gif)

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

## Usage

```bash
cltree
```

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [Claude Code](https://claude.com) - AI coding assistant by Anthropic
- Inspired by ranger, nnn, and other terminal file managers
