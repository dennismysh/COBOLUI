# COBALT

**COBOL Application Build & Layout Toolkit**

COBALT is a declarative UI framework that uses COBOL's DATA DIVISION as a component tree definition language. A Rust toolchain parses COBOL source files, produces a platform-neutral intermediate representation (IR), and renders to multiple backends.

## Architecture

```
COBOL Source (.cbl)
    │
    ├─ DATA DIVISION → Component Tree IR
    │      ├─ SCREEN SECTION    → Layout nodes
    │      ├─ WORKING-STORAGE   → State bindings
    │      └─ COPY ... REPLACING → Props
    │
    └─ PROCEDURE DIVISION → Event Handlers

Component Tree IR (Rust structs)
    │
    ├─ cobalt-term  → crossterm/ratatui  (terminal)
    ├─ html-renderer → HTML + CSS + JS   (web/WASM)  [planned]
    └─ compose-renderer → Kotlin/Compose (Android)   [planned]
```

## Quick Start

```bash
# Build from source
cargo build

# Validate a COBOL file
cargo run -p cobalt-cli -- check examples/hello.cbl

# View the IR as JSON
cargo run -p cobalt-cli -- build examples/hello.cbl

# Run in terminal (interactive TUI)
cargo run -p cobalt-cli -- run examples/hello.cbl
```

## COBOL UI Grammar

COBALT uses standard COBOL syntax with semantic conventions:

| Level | Role | Equivalent |
|-------|------|------------|
| 01 | Root screen / page | Top-level component |
| 05 | Container / section | Layout component |
| 10 | Leaf element | Input, button, label |
| 15 | Element modifier | Attributes / props |

| Clause | UI Meaning |
|--------|------------|
| `PIC X(n)` | Text field, max n chars |
| `PIC 9(n)` | Numeric input, n digits |
| `VALUE "..."` | Default value / label |
| `USING var` | Two-way data binding |
| `ON-ACTION PERFORM p` | Event handler |

## Example

```cobol
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  APP-STATE.
           05  USER-NAME      PIC X(40) VALUE "".

       SCREEN SECTION.
       01  MAIN-SCREEN.
           05  HEADER.
               10  TITLE      PIC X(30) VALUE "Hello, COBALT!".
           05  CONTENT.
               10  NAME-FIELD PIC X(40) USING USER-NAME.
               10  GREET-BTN  VALUE "Greet" ON-ACTION PERFORM HANDLE-GREET.
```

## Crate Structure

| Crate | Purpose |
|-------|---------|
| `cobalt-ir` | Intermediate representation types |
| `cobalt-parser` | COBOL → IR parser (pest PEG grammar) |
| `cobalt-render` | Renderer trait and event loop |
| `cobalt-term` | Terminal renderer (ratatui/crossterm) |
| `cobalt-cli` | CLI binary (`cobalt build/run/check`) |

## Controls (Terminal)

| Key | Action |
|-----|--------|
| Tab / Shift+Tab | Navigate between elements |
| Enter | Activate button |
| Type | Input text/numbers in focused field |
| Esc | Quit |

## License

MIT
