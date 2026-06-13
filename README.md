# CodeFinder

A native macOS desktop app for managing local Git repositories. Organize repos into folders, view git state at a glance, and open any repo in any IDE — all with one keystroke.

Built with **Tauri 2** + **Dioxus 0.7** (Rust throughout).

---

## Features

**Repository Management**
- Add repositories by path or scan a directory for git repos
- Organize repos into color-coded folders
- Star favorites
- Grid and list view modes
- Full-text search across repo names

**Dashboard**
- Stat cards: total repos, uncommitted, stale (30d+), favorites
- Attention list: repos with uncommitted changes or no recent commits
- Recent activity feed

**Activity Feed**
- Commits across all repositories, grouped by Today / Yesterday / This Week / Older
- Author avatars, commit message, repo name, hash, diff stats

**Repo Detail Panel**
- *Overview* — working tree status, sync state (ahead/behind), language breakdown, recent commits, contributor preview
- *Commits* — full commit history with author avatars, hash badges, additions/deletions
- *Branches* — branch list with current/default badges, ahead/behind indicators, one-click checkout
- *Contributors* — commit counts, addition/deletion totals with proportional bar charts
- *README* — rendered Markdown

**Git Operations**
- Fetch and Pull from remote
- Branch checkout (via libgit2, no shell)
- Working tree status with staged/modified/untracked breakdown

**IDE Integration**
- Auto-detects VS Code, Cursor, Xcode, Zed, IntelliJ IDEA, and more from `/Applications`
- Add custom editors by app path
- "Open in [Editor]" split button on every repo

**UX**
- `⌘K` command palette — jump to any repo or run actions
- Dark / light theme toggle
- Toast notifications for async operations

---

## Stack

| Layer | Tech |
|-------|------|
| Shell | Tauri 2 |
| UI | Dioxus 0.7 (Rust → WASM) |
| Git | `git2` 0.19 (libgit2, vendored) |
| Markdown | `pulldown-cmark` |
| State | Dioxus signals |
| Target | macOS 13+ |

---

## Project Structure

```
src/              # Dioxus UI (compiles to WASM)
  activity.rs     # Activity feed view
  app.rs          # Root component, global state
  dashboard.rs    # Dashboard + Settings modal
  detail.rs       # Repo detail panel (all tabs)
  icons.rs        # SVG icon registry
  invoke.rs       # Tauri IPC calls
  models.rs       # Shared data types + helpers
  palette.rs      # ⌘K command palette
  repolist.rs     # Repo list / grid
  shared.rs       # Avatar, StatusPill, Toast
  sidebar.rs      # Left navigation

src-tauri/src/    # Rust backend
  commands.rs     # Tauri command handlers
  git.rs          # libgit2 operations
  ide.rs          # IDE detection
  models.rs       # Serde structs
  store.rs        # JSON persistence

assets/
  styles.css      # All UI styles
```

---

## Development

```bash
# Install prerequisites
cargo install tauri-cli dioxus-cli

# Run in development (hot-reload UI + native backend)
cargo tauri dev

# Build release bundle
cargo tauri build
```

Requires Rust stable and Xcode Command Line Tools on macOS.

---

## Data Persistence

Repo list and folder config persist to `~/Library/Application Support/com.codefinder.app/data.json` via Tauri's app data directory API.
