# cltree

[![GitHub Release](https://img.shields.io/github/v/release/jsleemaster/cltree)](https://github.com/jsleemaster/cltree/releases)
[![npm](https://img.shields.io/npm/v/cltree)](https://www.npmjs.com/package/cltree)
[![Homebrew](https://img.shields.io/badge/homebrew-available-blue)](https://github.com/jsleemaster/homebrew-tap)
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

Claude Code CLIと一緒に使えるターミナルベースのファイルエクスプローラーです。分割画面TUIでプロジェクト構造を確認しながらClaude Codeを使用できます。

![cltree demo](demo.gif)

## 機能

- **分割画面TUI**: 左側にClaude Code、右側にファイルツリー
- **パッシブファイルツリー**: 常に展開された読み取り専用のプロジェクト構造表示
- **CWD追跡**: Claude Codeの現在の作業ディレクトリを●マーカーでハイライト
- **OSC 7 + vterm検出**: エスケープシーケンスによるディレクトリ変更の自動検出
- **gitignoreサポート**: `.gitignore`パターンを適用
- **ファイルアイコン**: ファイルタイプ別のビジュアルアイコン表示
- **ゼロ干渉**: すべてのキー入力がClaude Codeに直接転送

## インストール

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

## 使い方

```bash
cltree
```

## コントリビュート

コントリビューションを歓迎します！開発環境のセットアップとガイドラインについては[CONTRIBUTING.md](CONTRIBUTING.md)をご覧ください。

## ライセンス

このプロジェクトはMITライセンスの下で公開されています。詳細は[LICENSE](LICENSE)ファイルをご覧ください。

## 謝辞

- [ratatui](https://github.com/ratatui-org/ratatui) - ターミナルUIフレームワーク
- [Claude Code](https://claude.com) - AnthropicのAIコーディングアシスタント
- ranger、nnnなどのターミナルファイルマネージャーからインスピレーションを得ました
