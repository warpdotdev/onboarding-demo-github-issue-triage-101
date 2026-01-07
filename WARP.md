# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Build & Run

```bash
# Run with a repo argument (required)
cargo run -- owner/repo
cargo run -- facebook/react --limit 500

# Install locally
cargo install --path .

# Check/lint
cargo check
cargo fmt
```

## Architecture

Rust TUI app using ratatui + crossterm. Fetches GitHub issues via `gh` CLI.

### Modules

- `main.rs` - CLI parsing (clap), terminal setup, event loop
- `app.rs` - App state: issues, selection, filter, input mode
- `github.rs` - `gh issue list` wrapper, JSON parsing, `open` for browser
- `ui.rs` - ratatui rendering: header, split-pane (list + preview), help bar

### Data Flow

1. `main` parses args, creates `App`, calls `app.refresh()`
2. `refresh()` shells out to `gh issue list -R repo --json ...`
3. Event loop: `terminal.draw(ui::draw)` + handle key events
4. Filter is live-applied via `app.filtered_issues()` (title/label match)

### Key Patterns

- `InputMode` enum switches between Normal (navigation) and Filter (typing)
- Preview pane shows selected issue's body + comments
- Labels rendered with parsed hex colors from GitHub
