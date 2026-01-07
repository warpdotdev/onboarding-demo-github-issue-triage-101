# issue-triage

A terminal UI for browsing and triaging GitHub issues.

![Demo](demo.gif)

## Features

- Browse issues from any public GitHub repository
- Preview issue body and comments inline
- Filter issues by title or label
- Open issues in browser with one keypress
- Vim-style navigation (j/k)

## Prerequisites

- [GitHub CLI (`gh`)](https://cli.github.com/) installed and authenticated
- Rust toolchain

## Installation

```bash
cargo install --path .
```

Or run directly:

```bash
cargo run -- owner/repo
```

## Usage

```bash
# Browse issues from any public repo
issue-triage facebook/react
issue-triage rust-lang/rust
issue-triage microsoft/vscode

# Fetch more issues (default: 100)
issue-triage facebook/react --limit 500
```

## Key Bindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Open issue in browser |
| `/` | Start filtering |
| `Esc` | Clear filter / exit filter mode |
| `r` | Refresh issues from GitHub |
| `q` | Quit |

## How It Works

1. Runs `gh issue list` to fetch issues as JSON
2. Parses and displays in a split-pane TUI
3. Filter works on issue titles and label names
4. `Enter` runs `gh issue view --web` to open in browser

## License

MIT
