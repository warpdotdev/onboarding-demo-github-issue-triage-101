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

- Rust toolchain
- (Optional) GitHub personal access token for accessing private repos or increasing rate limits

## Installation

```bash
cargo install --path .
```

Or run directly:

```bash
cargo run -- owner/repo
```

## (Optional) Authentication

Set a GitHub personal access token for private repos or higher API rate limits. [Follow this guide to create a personal access token.](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)

Then, export your token into your shell environment before starting the TUI:

```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxx
```

Without a token, the app uses unauthenticated requests (60 requests/hour limit, public repos only).

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

1. Fetches issues via GitHub REST API (using `octocrab`)
2. Parses and displays in a split-pane TUI
3. Filter works on issue titles and label names
4. `Enter` opens the issue in your browser

## License

MIT
