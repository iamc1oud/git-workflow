# CodeFinder — Technical Spec

## What

macOS desktop app. Organize local git repos into custom folders. View git state at a glance. Open any repo in any IDE. One-keystroke everything.

**Target:** macOS 13+. Single binary ~5–10 MB.

---

## Stack

| Layer | Tech | Notes |
|-------|------|-------|
| Shell | Tauri 2 | Native webview, IPC bridge, OS permissions |
| UI | Dioxus (Rust) | Replaces React prototype. Same CSS, same layout. |
| Backend | Rust | FS scan, git ops, IDE detect |
| Git | `git2` crate (libgit2) | No shell-out. Structured data. |
| Packaging | `tauri-cli` | `.app` bundle |
| State | Dioxus signals | `use_signal`, `use_memo` — maps 1:1 to React hooks |

> Design prototype is React/JSX (browser). Port to Dioxus. CSS unchanged.

---

## Data Model

```rust
struct Folder { id: String, name: String, icon: String, color: String }

struct RepoSummary {
    id: String, name: String, folder_id: String, path: String,
    description: String, remote: String, remote_host: String,
    default_branch: String, branch: String, favorite: bool,
    last_fetched: DateTime, last_commit_at: DateTime,
    languages: Vec<Language>, status: WorkingTreeStatus,
}

struct RepoDetail {
    // all RepoSummary fields +
    branches: Vec<Branch>,
    commits: Vec<Commit>,
    contributors: Vec<Contributor>,
    readme: String,
}

struct WorkingTreeStatus {
    clean: bool, staged: u32, modified: u32, untracked: u32,
    ahead: u32, behind: u32, files: Vec<StatusFile>,
}

struct StatusFile { path: String, kind: FileStatus }
enum FileStatus { Modified, Staged, Untracked }

struct Branch { name: String, current: bool, ahead: u32, behind: u32, last_msg: String, when: DateTime }
struct Commit { hash: String, msg: String, author: String, date: DateTime, additions: u32, deletions: u32 }
struct Contributor { name: String, handle: String, commits: u32, additions: u32, deletions: u32, last_active: DateTime }
struct Language { name: String, pct: u8, color: String }

struct Editor { id: String, name: String, detected: bool, path: String, cmd: String }
```

All structs: `#[derive(Serialize, Deserialize)]` → cross IPC bridge as JSON.

---

## Tauri Commands (IPC)

```rust
#[tauri::command] fn list_folders() -> Vec<Folder>;
#[tauri::command] fn add_folder(name: String, icon: String, color: String) -> Folder;
#[tauri::command] fn list_repos(folder_id: String) -> Vec<RepoSummary>;
#[tauri::command] fn add_repo(folder_id: String, path: String) -> RepoSummary;
#[tauri::command] fn get_repo_detail(path: String) -> RepoDetail;
#[tauri::command] fn get_commits(path: String, branch: String) -> Vec<Commit>;
#[tauri::command] fn get_status(path: String) -> WorkingTreeStatus;
#[tauri::command] fn detect_ides() -> Vec<Editor>;
#[tauri::command] fn open_in(ide_id: String, path: String) -> Result<(), String>;
#[tauri::command] fn fetch(path: String) -> Result<SyncResult, String>;
#[tauri::command] fn pull(path: String) -> Result<SyncResult, String>;
#[tauri::command] fn scan_dir(parent: String) -> Vec<String>; // find all git repos under path
```

---

## UI Layout

```
┌─────────────────────────────────────────────────────────┐
│  TITLEBAR  [traffic lights] CodeFinder  [⌘K search]  ☀️ ⚙️ │
├──────────┬───────────────┬──────────────────────────────┤
│ SIDEBAR  │  REPO LIST    │  DETAIL PANEL                │
│          │               │                              │
│ Smart    │ Filter/Sort   │  Header: name, path, actions │
│ Groups   │ List/Grid     │  Tabs: Overview | Commits    │
│          │               │        Branches | Contributors│
│ Folders  │  RepoRow or   │        README                │
│          │  RepoCard     │                              │
│ Favorites│               │  Overview: WorkTree + Sync  │
│          │               │  + Languages + recent commits│
└──────────┴───────────────┴──────────────────────────────┘
```

**Full-width modes:** Dashboard (stat cards + attention list + activity + folder grid) | Activity feed (cross-repo commit timeline, grouped Today/Yesterday/This week/Earlier)

---

## Navigation State

```
Selection { kind: Smart | Folder | Repo, id: String }

Smart IDs:
  "dashboard"   → Dashboard full-width
  "activity"    → Activity feed full-width
  "all"         → all repos
  "uncommitted" → repos where !status.clean
  "stale"       → repos where last_commit_at > 30 days ago
```

---

## Component Map (React → Dioxus)

| React component | Dioxus component | File |
|----------------|-----------------|------|
| `App` | `App` | app.rs |
| `Sidebar` | `Sidebar` | nav.rs |
| `RepoList` + `RepoRow` + `RepoCard` | `RepoList` | repolist.rs |
| `Detail` + tabs | `Detail` | detail.rs |
| `Dashboard` | `Dashboard` | dashboard.rs |
| `ActivityFeed` | `ActivityFeed` | dashboard.rs |
| `CommandPalette` | `CommandPalette` | palette.rs |
| `SettingsModal` | `SettingsModal` | settings.rs |
| `Icon` | `Icon` | lib.rs |
| `Avatar`, `AvatarStack` | `Avatar`, `AvatarStack` | lib.rs |
| `StatusPill` | `StatusPill` | lib.rs |
| `OpenButton` (split) | `OpenButton` | detail.rs |

---

## State (App-level)

```rust
// Dioxus signals
let sel = use_signal(|| Selection { kind: Smart, id: "dashboard" });
let selected_repo_id = use_signal(|| None::<String>);
let theme = use_signal(|| "dark");
let list_view = use_signal(|| "list");
let palette_open = use_signal(|| false);
let settings_open = use_signal(|| false);
let ides = use_signal(|| vec![]);  // loaded via detect_ides command
let toasts = use_signal(|| vec![]);
let accent = use_signal(|| "#7C6BFF");
```

---

## Repo Status Logic

```rust
enum StatusKind { Dirty, Ahead, Behind, Clean }

fn status_of(repo: &RepoSummary) -> (StatusKind, String) {
    if !repo.status.clean {
        let n = repo.status.staged + repo.status.modified + repo.status.untracked;
        return (Dirty, format!("{n} change{}", if n == 1 { "" } else { "s" }));
    }
    if repo.status.ahead > 0 { return (Ahead, format!("{} ahead", repo.status.ahead)); }
    if repo.status.behind > 0 { return (Behind, format!("{} behind", repo.status.behind)); }
    (Clean, "clean".into())
}
```

---

## Keyboard

| Key | Action |
|-----|--------|
| `⌘K` | Open command palette |
| `↑↓` | Navigate repo list / palette |
| `Enter` | Select repo / run command |
| `Esc` | Close palette / modal |

Palette commands: go-dashboard, go-activity, go-uncommitted, fetch-all, toggle-theme, new-folder, manage-editors

---

## IDE Detection (macOS)

Scan `/Applications/` for known `.app` bundles:

| ID | App | CLI cmd |
|----|-----|---------|
| vscode | Visual Studio Code.app | `code` |
| cursor | Cursor.app | `cursor` |
| xcode | Xcode.app | `xed` |
| zed | Zed.app | `zed` |
| jetbrains | IntelliJ IDEA.app | `idea` |
| terminal | Terminal.app | `open -a Terminal` |
| finder | Finder.app | `open` |

User can add custom editor by path. Persisted in app config.

---

## Persistence (app config)

Store in `~/.config/codefinder/` (via Tauri path API):
- `folders.json` — folder definitions
- `repos.json` — repo → folder mapping + favorites
- `editors.json` — custom editors list
- `prefs.json` — theme, accent, default view

---

## Roadmap

- [ ] Drag-drop repos between folders
- [ ] Auto-import: scan parent dir, add all git repos (uses `scan_dir` command)
- [ ] Live FS watch via `notify` crate → refresh status on change
- [ ] Per-repo default editor override
- [ ] Tags/labels alongside folders
- [ ] Search commit messages cross-repo

---

## macOS Permissions

- First launch: macOS prompts **Files and Folders** access
- For arbitrary paths: **Full Disk Access** (System Settings → Privacy & Security)
- Both requested via Tauri's `fs` permission scope in `tauri.conf.json`
