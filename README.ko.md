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

Claude Code CLI와 함께 사용할 수 있는 터미널 기반 파일 탐색기입니다. 분할 화면 TUI에서 프로젝트 구조를 보면서 Claude Code를 사용할 수 있습니다.

![cltree demo](demo.gif)

## 기능

- **분할 화면 TUI**: 왼쪽에 Claude Code, 오른쪽에 파일 트리
- **패시브 파일 트리**: 항상 펼쳐진 읽기 전용 프로젝트 구조 표시
- **CWD 추적**: Claude Code의 현재 작업 디렉토리를 ● 마커로 강조
- **OSC 7 + vterm 감지**: 이스케이프 시퀀스를 통한 디렉토리 변경 자동 감지
- **gitignore 지원**: `.gitignore` 패턴 적용
- **파일 아이콘**: 파일 유형별 시각적 아이콘 표시
- **제로 간섭**: 모든 키 입력이 Claude Code로 직접 전달

## 설치

### npm / bun

```bash
npm install -g cltree
# or
bun install -g cltree
```

### Homebrew (macOS / Linux)

```bash
brew tap jsleemaster/tap
brew install cltree
```

## 사용법

```bash
cltree
```

## 기여

기여를 환영합니다! 개발 환경 설정 및 가이드라인은 [CONTRIBUTING.md](CONTRIBUTING.md)를 참고하세요.

## 라이선스

이 프로젝트는 MIT 라이선스로 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참고하세요.

## 감사의 글

- [ratatui](https://github.com/ratatui-org/ratatui) - 터미널 UI 프레임워크
- [Claude Code](https://claude.com) - Anthropic의 AI 코딩 어시스턴트
- ranger, nnn 등 터미널 파일 매니저에서 영감을 받았습니다
