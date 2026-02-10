# Claude Explorer - Project Context

Claude Code CLI 옆에 파일 트리를 보여주는 TUI 앱. Rust + ratatui 기반.

## 프로젝트 구조

```
claude-explorer/
├── src/
│   ├── main.rs              # 진입점, 터미널 초기화, 이벤트 루프
│   ├── app.rs               # App 상태, 키 입력 핸들링
│   ├── event.rs             # 비동기 이벤트 핸들러 (키, 마우스, 리사이즈)
│   ├── lib.rs               # 라이브러리 모듈 선언
│   ├── terminal.rs          # PTY 관리, Claude Code 프로세스 실행
│   ├── vterm.rs             # 가상 터미널 버퍼 (ANSI 파싱, 셀 기반 렌더링)
│   ├── tree/
│   │   ├── mod.rs           # FileTree 구조체, 트리 빌드 로직
│   │   └── file_node.rs     # FileNode 구조체, 파일 아이콘 매핑
│   └── ui/
│       ├── mod.rs           # draw() 함수, 레이아웃 분할
│       ├── file_tree_widget.rs   # 트리 렌더링 위젯
│       └── terminal_widget.rs    # vterm 버퍼 → ratatui 위젯 변환
├── Cargo.toml
├── README.md
├── LICENSE                  # MIT
└── CLAUDE.md               # 이 파일
```

## 핵심 의존성

- `ratatui` (0.30): TUI 프레임워크
- `crossterm` (0.29): 크로스플랫폼 터미널 제어
- `portable-pty` (0.9): PTY 생성 및 프로세스 관리
- `tokio` (1.42): 비동기 런타임
- `notify` (8.2) + `notify-debouncer-mini` (0.7): 파일시스템 감시
- `ignore` (0.4): gitignore 지원 파일 워킹
- `clap` (4.5): CLI 인자 파싱

## 아키텍처

```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                             │
│  - Terminal 초기화 (raw mode, alternate screen)             │
│  - App 생성                                                 │
│  - EventHandler 생성                                        │
│  - 메인 루프: draw() → event.next() → handle               │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│    app.rs     │    │   event.rs    │    │    ui/mod     │
│               │    │               │    │               │
│ - App struct  │    │ - Event enum  │    │ - draw()      │
│ - FileTree    │◄───│ - Tick        │    │ - Layout      │
│ - TerminalPane│    │ - Key/Mouse   │    │               │
│ - CWD 추적    │    │ - Resize      │    │               │
└───────────────┘    └───────────────┘    └───────────────┘
        │                                         │
        ▼                                         ▼
┌───────────────┐    ┌───────────────┐   ┌───────────────┐
│  terminal.rs  │    │   vterm.rs    │   │   widgets     │
│               │    │               │   │               │
│ - PTY 생성    │    │ - 셀 버퍼    │   │ - TreeWidget  │
│ - Claude 실행 │───▶│ - ANSI 파싱  │   │ - TermWidget  │
│ - I/O 처리    │    │ - CWD 감지   │   │               │
└───────────────┘    └───────────────┘   └───────────────┘
```

## 빌드 & 실행

```bash
# 개발 모드 실행
cargo run

# 릴리스 빌드
cargo build --release

# 특정 경로에서 실행
cargo run -- --path /some/project

# 테스트
cargo test

# 린트
cargo clippy

# 포맷팅
cargo fmt
```

## 코딩 컨벤션

- Rust 2021 에디션
- `cargo fmt` 스타일 준수
- 에러 처리: `anyhow::Result` 사용 (라이브러리 경계에서는 `thiserror`)
- 주석: 한글 가능, 공개 API는 영문 doc comment
- 네이밍: snake_case (함수/변수), PascalCase (타입)

## 주요 구현 포인트

### 1. PTY 통합 (terminal.rs)
- `portable-pty`로 pseudo-terminal 생성
- Claude Code를 자식 프로세스로 spawn
- 백그라운드 스레드에서 출력 읽기 (non-blocking)
- 키 입력을 PTY master로 전송

### 2. 가상 터미널 (vterm.rs)
- 셀 기반 터미널 버퍼 (행/열 그리드)
- SGR 이스케이프 시퀀스 파싱 (색상, 볼드 등)
- 커서 이동, 삽입/삭제, 스크롤 처리
- OSC 7 이스케이프 시퀀스로 CWD 감지
- vterm 버퍼 스캔으로 CWD 폴백 감지

### 3. 파일 트리 (tree/mod.rs)
- `ignore` crate로 gitignore 지원
- 디렉토리 우선 정렬
- 항상 전체 확장 (읽기 전용 패시브 디스플레이)
- CWD 마커(●) 표시

### 4. 이벤트 루프 (event.rs + main.rs)
- tokio select! 기반 비동기
- 모든 키 입력을 PTY로 직접 전달
- 파일 변경 감시 (notify) 통합
- CWD 변경 시 디바운스된 트리 갱신

## 테스트 전략

```bash
# 단위 테스트
cargo test

# 특정 모듈 테스트
cargo test tree::
cargo test ui::
```

## 알려진 이슈 / TODO

- [ ] PTY 리사이즈 동기화 개선
- [ ] 파일 변경 감지 (notify) 통합
- [ ] 마우스 클릭으로 파일 선택
- [ ] 설정 파일 지원 (~/.config/claude-explorer/config.toml)
- [ ] 테마 커스터마이징

## 디버깅 팁

```bash
# 로그 출력 (stderr는 TUI에 영향 안 줌)
RUST_LOG=debug cargo run 2> debug.log

# PTY 없이 트리만 테스트
cargo run -- --no-terminal  # TODO: 구현 필요
```

## Agent Teams 작업 가이드

### 독립 작업 영역
- **config 시스템**: `src/config.rs` (신규), `src/app.rs`
- **마우스 이벤트**: `src/event.rs`, `src/app.rs`, `src/ui/file_tree_widget.rs`
- **테스트**: `tests/` 디렉토리
- **테마**: `src/ui/` 전체

### 주의사항
- `src/app.rs`는 여러 기능이 터치할 수 있음 → 작은 단위로 작업
- 새 모듈 추가 시 `src/main.rs`의 mod 선언 필요
- `cargo test`로 항상 검증

## 릴리스 체크리스트

1. `cargo fmt && cargo clippy`
2. `cargo test`
3. README 업데이트
4. Cargo.toml 버전 업데이트
5. `cargo publish --dry-run`
6. Git tag 생성
