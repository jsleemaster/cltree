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

一个与Claude Code CLI配合使用的终端文件浏览器。在分屏TUI中查看项目结构的同时使用Claude Code。

![cltree demo](demo.gif)

## 功能

- **分屏TUI**: 左侧Claude Code，右侧文件树
- **被动文件树**: 始终展开的只读项目结构显示
- **CWD追踪**: 用●标记高亮显示Claude Code的当前工作目录
- **OSC 7 + vterm检测**: 通过转义序列自动检测目录变更
- **gitignore支持**: 遵循`.gitignore`模式
- **文件图标**: 不同文件类型的可视化图标显示
- **零干扰**: 所有按键直接转发到Claude Code

## 安装

### npm / bun

```bash
npm install -g cltree
# or
bun install -g cltree
```

## 使用方法

```bash
cltree
```

## 贡献

欢迎贡献！有关开发环境设置和指南，请参阅[CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

本项目基于MIT许可证发布。详情请参阅[LICENSE](LICENSE)文件。

## 致谢

- [ratatui](https://github.com/ratatui-org/ratatui) - 终端UI框架
- [Claude Code](https://claude.com) - Anthropic的AI编程助手
- 灵感来源于ranger、nnn等终端文件管理器
