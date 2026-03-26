# COBALT

**COBOL Application Build & Layout Toolkit**

COBALT is a declarative UI framework that transforms COBOL source files into interactive terminal user interfaces. It repurposes COBOL's hierarchical `DATA DIVISION` — originally designed for record layouts — as a modern component tree definition language. A Rust toolchain parses `.cbl` source files, produces a platform-neutral intermediate representation (IR), and renders to pluggable backends.

## Why COBALT?

COBOL's level-numbered data structures are a natural fit for describing nested UI layouts. COBALT leverages this by mapping:

- **Level numbers** to component hierarchy (containers, sections, leaf elements)
- **PIC clauses** to input types and constraints
- **WORKING-STORAGE** to reactive application state
- **PROCEDURE DIVISION paragraphs** to event handlers

The result: build terminal UIs using a familiar, structured language — no JavaScript, no HTML, no CSS.

## Architecture

```
COBOL Source (.cbl)
    │
    ├─ DATA DIVISION ──────────► Component Tree IR
    │      ├─ SCREEN SECTION       → Layout nodes (Container, Text, Numeric, Button)
    │      ├─ WORKING-STORAGE      → State bindings
    │      └─ COPY ... REPLACING   → Props
    │
    └─ PROCEDURE DIVISION ─────► Event Handlers
           └─ Paragraphs          → Named action callbacks

Component Tree IR (Rust structs / JSON)
    │
    ├─ cobalt-term  ──► crossterm + ratatui  (terminal TUI)
    ├─ html-renderer ─► HTML + CSS + JS      (web/WASM)    [planned]
    └─ compose-renderer ► Kotlin/Compose     (Android)     [planned]
```

## Tech Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Language | Rust | Edition 2021, MSRV 1.75 |
| Parser | [pest](https://pest.rs/) (PEG grammar) | 2.x |
| Terminal UI | [ratatui](https://ratatui.rs/) | 0.29 |
| Terminal I/O | [crossterm](https://github.com/crossterm-rs/crossterm) | 0.28 |
| CLI | [clap](https://docs.rs/clap) (derive) | 4.x |
| Serialization | serde + serde_json | 1.x |
| Error handling | thiserror + anyhow | 2.x / 1.x |

## Crate Structure

The project is organized as a Rust workspace with five focused crates:

```
crates/
├── cobalt-ir/        # Intermediate representation types (CobaltApp, Screen, Node, Style)
├── cobalt-parser/    # COBOL → IR parser powered by a PEG grammar (grammar.pest)
├── cobalt-render/    # Renderer trait and event loop framework
├── cobalt-term/      # Terminal renderer implementation (ratatui + crossterm)
└── cobalt-cli/       # CLI binary with build/run/check/new subcommands
```

| Crate | Purpose | Key Types / Exports |
|-------|---------|---------------------|
| `cobalt-ir` | Platform-neutral IR types | `CobaltApp`, `Screen`, `Node`, `StateField`, `Handler`, `Style` |
| `cobalt-parser` | Parses COBOL subset into IR | `parse_cobol() → CobaltApp` |
| `cobalt-render` | Abstract renderer contract | `Renderer` trait, `run_app()`, `EventRecord` |
| `cobalt-term` | Concrete terminal backend | `TermRenderer` (focus, input, styling) |
| `cobalt-cli` | User-facing CLI tool | `build`, `run`, `check`, `new` subcommands |

## Quick Start

### Prerequisites

- **Rust 1.75+** — install via [rustup](https://rustup.rs/)

### Build

```bash
# Clone the repository
git clone https://github.com/dennismysh/cobolui.git
cd cobolui

# Build all crates
cargo build

# Build in release mode
cargo build --release
```

### Run

```bash
# Validate a COBOL file for COBALT compatibility
cargo run -p cobalt-cli -- check examples/hello.cbl

# Parse and emit the IR as JSON (useful for debugging)
cargo run -p cobalt-cli -- build examples/hello.cbl

# Launch an interactive terminal UI
cargo run -p cobalt-cli -- run examples/hello.cbl
```

### Test

```bash
cargo test
```

## COBOL UI Grammar

COBALT uses a subset of standard COBOL syntax with semantic conventions for UI layout:

### Level Number Mapping

| Level | Role | UI Equivalent |
|-------|------|---------------|
| `01` | Root screen / page | Top-level component |
| `05` | Container / section | Layout group |
| `10` | Leaf element | Input, button, or label |
| `15` | Element modifier | Attributes / props |

### Supported Clauses

| Clause | UI Meaning | Example |
|--------|------------|---------|
| `PIC X(n)` | Text input field, max `n` characters | `PIC X(40)` |
| `PIC 9(n)` | Numeric input, `n` digits | `PIC 9(4)` |
| `PIC A(n)` | Alphabetic input, `n` characters | `PIC A(20)` |
| `PIC 9(n)V9(m)` | Decimal number (`V` = decimal point) | `PIC 9(3)V9(2)` |
| `VALUE "..."` | Default value or static label | `VALUE "Submit"` |
| `USING var` | Two-way data binding to WORKING-STORAGE | `USING USER-NAME` |
| `ON-ACTION PERFORM p` | Attach event handler paragraph | `ON-ACTION PERFORM HANDLE-CLICK` |
| `BACKGROUND-COLOR n` | Background color (palette index 0–7) | `BACKGROUND-COLOR 1` |
| `FOREGROUND-COLOR n` | Foreground color (palette index 0–7) | `FOREGROUND-COLOR 4` |

## Examples

### Hello World

```cobol
       IDENTIFICATION DIVISION.
       PROGRAM-ID. HELLO.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  APP-STATE.
           05  USER-NAME      PIC X(40) VALUE "".
           05  STATUS-MSG     PIC X(60) VALUE "Welcome to COBALT!".

       SCREEN SECTION.
       01  MAIN-SCREEN.
           05  HEADER.
               10  TITLE      PIC X(30) VALUE "Hello, COBALT!".
           05  CONTENT.
               10  NAME-FIELD PIC X(40) USING USER-NAME.
               10  GREET-BTN  VALUE "Greet" ON-ACTION PERFORM HANDLE-GREET.
           05  FOOTER.
               10  MSG-TEXT   PIC X(60) USING STATUS-MSG.

       PROCEDURE DIVISION.
       MAIN-LOOP.
           STOP RUN.

       HANDLE-GREET.
           DISPLAY "Hello!".
```

### Counter App

```cobol
       IDENTIFICATION DIVISION.
       PROGRAM-ID. COUNTER.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  APP-STATE.
           05  COUNTER-VAL    PIC 9(4) VALUE 0.
           05  STATUS-MSG     PIC X(60) VALUE "Ready".

       SCREEN SECTION.
       01  MAIN-SCREEN.
           05  HEADER.
               10  TITLE      PIC X(30) VALUE "Counter App".
           05  DISPLAY-AREA.
               10  COUNT-DISPLAY PIC 9(4) USING COUNTER-VAL.
           05  CONTROLS.
               10  INC-BTN    VALUE "+" ON-ACTION PERFORM HANDLE-INCREMENT.
               10  DEC-BTN    VALUE "-" ON-ACTION PERFORM HANDLE-DECREMENT.
               10  RESET-BTN  VALUE "Reset" ON-ACTION PERFORM HANDLE-RESET.
           05  STATUS-BAR.
               10  MSG-TEXT   PIC X(60) USING STATUS-MSG.

       PROCEDURE DIVISION.
       MAIN-LOOP.
           STOP RUN.

       HANDLE-INCREMENT.
           ADD 1 TO COUNTER-VAL.

       HANDLE-DECREMENT.
           SUBTRACT 1 FROM COUNTER-VAL.

       HANDLE-RESET.
           MOVE 0 TO COUNTER-VAL.
```

## Terminal Controls

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Navigate between focusable elements |
| `Enter` | Activate the focused button |
| Type characters | Input text/numbers in the focused field |
| `Backspace` | Delete last character in input |
| `Esc` | Quit the application |

## Project Configuration

COBALT projects can include a `cobalt.toml` for build settings and theming:

```toml
[project]
name = "my-cobalt-app"
version = "0.1.0"
entry = "src/screens/MAIN.cbl"

[build]
target = "terminal"    # "terminal", "web", "android"
out_dir = "dist"

[theme]
name = "default"

[theme.palette]
0 = "#1E293B"    # bg-primary
1 = "#3B82F6"    # accent
2 = "#FFFFFF"    # bg-surface
3 = "#F8FAFC"    # bg-muted
4 = "#1A1A1A"    # text-primary
5 = "#2563EB"    # link
6 = "#16A34A"    # success
7 = "#DC2626"    # error
```

## Roadmap

- [x] PEG-based COBOL parser (SCREEN SECTION, WORKING-STORAGE, PROCEDURE DIVISION)
- [x] Platform-neutral intermediate representation with JSON serialization
- [x] Terminal renderer with focus management, text/numeric input, and button support
- [x] CLI tool (`build`, `run`, `check`)
- [x] `cobalt new` project scaffolding
- [ ] Web/WASM renderer (HTML + CSS + JS)
- [ ] Android renderer (Jetpack Compose)
- [ ] Multi-screen navigation
- [ ] COPY ... REPLACING support for reusable components

## License

MIT — see [LICENSE](LICENSE) for details.

Copyright (c) 2026 Dennis Myshkovskiy
