use std::sync::Mutex;

use tauri::State;

use crate::{
    git, ide,
    models::*,
    store::{RepoRecord, Store},
};


pub struct AppState {
    pub store: Store,
}

// Build a full RepoSummary from a persisted record + live git data.
fn build_summary(record: &RepoRecord) -> RepoSummary {
    let path = &record.path;
    let name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&record.id)
        .to_string();
    let (remote, remote_host) = git::get_remote_url(path);

    RepoSummary {
        id: record.id.clone(),
        name,
        folder_id: record.folder_id.clone(),
        path: path.clone(),
        description: record.description.clone(),
        remote,
        remote_host,
        default_branch: git::get_default_branch(path),
        branch: git::get_current_branch(path),
        favorite: record.favorite,
        last_fetched: record.last_fetched.clone(),
        last_commit_at: git::get_last_commit_time(path),
        languages: git::detect_languages(path),
        status: git::get_status(path),
    }
}

// ── Folder commands ───────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_folders(state: State<Mutex<AppState>>) -> Vec<Folder> {
    state.lock().unwrap().store.data.folders.clone()
}

#[tauri::command]
pub fn add_folder(
    name: String,
    icon: String,
    color: String,
    state: State<Mutex<AppState>>,
) -> Folder {
    state.lock().unwrap().store.add_folder(name, icon, color)
}

#[tauri::command]
pub fn remove_folder(folder_id: String, state: State<Mutex<AppState>>) {
    state.lock().unwrap().store.remove_folder(&folder_id);
}

// ── Repo commands ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_repos(folder_id: String, state: State<Mutex<AppState>>) -> Vec<RepoSummary> {
    let records: Vec<RepoRecord> = {
        let guard = state.lock().unwrap();
        guard
            .store
            .data
            .repos
            .iter()
            .filter(|r| r.folder_id == folder_id)
            .cloned()
            .collect()
    };
    records.iter().map(build_summary).collect()
}

#[tauri::command]
pub fn list_all_repos(state: State<Mutex<AppState>>) -> Vec<RepoSummary> {
    let records: Vec<RepoRecord> = state.lock().unwrap().store.data.repos.clone();
    records.iter().map(build_summary).collect()
}

#[tauri::command]
pub fn add_repo(
    folder_id: String,
    path: String,
    description: String,
    state: State<Mutex<AppState>>,
) -> Result<RepoSummary, String> {
    // Reject if path is not a valid git repository
    git2::Repository::open(&path)
        .map_err(|_| format!("'{}' is not a git repository", path))?;

    let record = state
        .lock()
        .unwrap()
        .store
        .add_repo(folder_id, path, description)
        .ok_or_else(|| "Failed to add repository".to_string())?;

    Ok(build_summary(&record))
}

#[tauri::command]
pub fn remove_repo(repo_id: String, state: State<Mutex<AppState>>) {
    state.lock().unwrap().store.remove_repo(&repo_id);
}

#[tauri::command]
pub fn get_repo_detail(path: String, state: State<Mutex<AppState>>) -> Option<RepoDetail> {
    let record = {
        let guard = state.lock().unwrap();
        guard.store.data.repos.iter().find(|r| r.path == path).cloned()?
    };

    let summary = build_summary(&record);
    let commits = git::get_commits(&path, &summary.branch);
    let branches = git::get_branches(&path);
    let contributors = git::get_contributors(&commits);
    let readme = git::get_readme(&path);

    Some(RepoDetail { summary, branches, commits, contributors, readme })
}

#[tauri::command]
pub fn get_commits(path: String, branch: String) -> Vec<Commit> {
    git::get_commits(&path, &branch)
}

#[tauri::command]
pub fn list_recent_activity(state: State<Mutex<AppState>>) -> Vec<ActivityItem> {
    let records: Vec<RepoRecord> = state.lock().unwrap().store.data.repos.clone();
    let summaries: Vec<RepoSummary> = records.iter().map(build_summary).collect();

    let mut items: Vec<ActivityItem> = Vec::new();
    for summary in &summaries {
        let commits = git::get_commits(&summary.path, &summary.branch);
        for commit in commits.into_iter().take(15) {
            items.push(ActivityItem {
                repo_id: summary.id.clone(),
                repo_name: summary.name.clone(),
                hash: commit.hash,
                msg: commit.msg,
                author: commit.author,
                date: commit.date,
                additions: commit.additions,
                deletions: commit.deletions,
            });
        }
    }

    items.sort_by(|a, b| b.date.cmp(&a.date));
    items.truncate(150);
    items
}

#[tauri::command]
pub fn get_status(path: String) -> WorkingTreeStatus {
    git::get_status(&path)
}

#[tauri::command]
pub fn checkout_branch(path: String, branch: String) -> SyncResult {
    match git::checkout_branch(&path, &branch) {
        Ok(_) => SyncResult { success: true, message: format!("Switched to '{}'", branch) },
        Err(e) => SyncResult { success: false, message: e },
    }
}

#[tauri::command]
pub fn toggle_favorite(repo_id: String, state: State<Mutex<AppState>>) -> bool {
    state.lock().unwrap().store.toggle_favorite(&repo_id)
}

#[tauri::command]
pub fn update_description(
    repo_id: String,
    description: String,
    state: State<Mutex<AppState>>,
) {
    state.lock().unwrap().store.update_description(&repo_id, description);
}

// ── IDE / editor commands ─────────────────────────────────────────────────────

#[tauri::command]
pub fn detect_ides(state: State<Mutex<AppState>>) -> Vec<Editor> {
    let custom = state.lock().unwrap().store.data.custom_editors.clone();
    let mut all = ide::detect_ides();
    all.extend(custom);
    all
}

#[tauri::command]
pub fn add_custom_editor(
    id: String,
    name: String,
    path: String,
    cmd: String,
    state: State<Mutex<AppState>>,
) {
    let mark = {
        let upper: String = name.chars().filter(|c| c.is_uppercase()).take(2).collect();
        if upper.is_empty() {
            name.chars().take(2).collect::<String>().to_uppercase()
        } else {
            upper
        }
    };
    let editor = Editor {
        id,
        name,
        detected: true,
        path,
        accent: "#7C6BFF".into(),
        mark,
        cmd,
    };
    state.lock().unwrap().store.add_editor(editor);
}

#[tauri::command]
pub fn remove_custom_editor(editor_id: String, state: State<Mutex<AppState>>) {
    state.lock().unwrap().store.remove_editor(&editor_id);
}

#[tauri::command]
pub fn open_in(
    ide_id: String,
    path: String,
    state: State<Mutex<AppState>>,
) -> Result<(), String> {
    // Only allow opening registered repos
    {
        let guard = state.lock().unwrap();
        if !guard.store.data.repos.iter().any(|r| r.path == path) {
            return Err(format!("Path '{}' is not a registered repository", path));
        }
    }

    let custom = state.lock().unwrap().store.data.custom_editors.clone();
    let system = ide::detect_ides();

    let editor = system
        .iter()
        .chain(custom.iter())
        .find(|e| e.id == ide_id)
        .ok_or_else(|| format!("Editor '{}' not found", ide_id))?
        .clone();

    // Split cmd into program + static args (e.g. "open -a Terminal" → ["open", "-a", "Terminal"]).
    // Pass path as a separate argv element — no shell, no interpolation.
    let mut parts = editor.cmd.split_whitespace();
    let program = parts.next().ok_or("Editor has empty command")?;
    let mut cmd = std::process::Command::new(program);
    for arg in parts {
        cmd.arg(arg);
    }
    cmd.arg(&path).spawn().map_err(|e| e.to_string())?;

    Ok(())
}

// ── Git action commands ───────────────────────────────────────────────────────

#[tauri::command]
pub fn fetch(path: String, state: State<Mutex<AppState>>) -> Result<SyncResult, String> {
    // Only operate on registered repos — prevents running git on arbitrary paths
    // (including repos with malicious .git/config hooks)
    if !state.lock().unwrap().store.data.repos.iter().any(|r| r.path == path) {
        return Err(format!("Path '{}' is not a registered repository", path));
    }

    let output = std::process::Command::new("git")
        .args(["fetch", "--all", "--prune"])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    state.lock().unwrap().store.update_last_fetched(&path);

    Ok(SyncResult {
        success: output.status.success(),
        message: if output.status.success() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).trim().to_string()
        },
    })
}

#[tauri::command]
pub fn pull(path: String, state: State<Mutex<AppState>>) -> Result<SyncResult, String> {
    if !state.lock().unwrap().store.data.repos.iter().any(|r| r.path == path) {
        return Err(format!("Path '{}' is not a registered repository", path));
    }

    let output = std::process::Command::new("git")
        .arg("pull")
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    Ok(SyncResult {
        success: output.status.success(),
        message: if output.status.success() {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).trim().to_string()
        },
    })
}

// ── Scan ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn scan_dir(parent: String) -> Vec<String> {
    git::scan_dir_for_repos(&parent)
}

// ── Folder picker ─────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .pick_folder(move |path| {
            let _ = tx.send(path);
        });
    rx.await.ok().flatten().map(|p| p.to_string())
}

// ── Shell helpers ─────────────────────────────────────────────────────────────

fn safe_dir(path: &str) -> Option<std::path::PathBuf> {
    // Reject flag-like paths, then canonicalize and confirm it's a directory.
    if path.starts_with('-') { return None; }
    let canon = std::fs::canonicalize(path).ok()?;
    if canon.is_dir() { Some(canon) } else { None }
}

#[tauri::command]
pub fn open_terminal(path: String) {
    let Some(dir) = safe_dir(&path) else { return };
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open")
        .args(["-a", "Terminal"])
        .arg(&dir)
        .spawn();
    // Windows: no shell interpolation — set working dir, let cmd start there.
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd")
        .args(["/c", "start", "", "cmd"])
        .current_dir(&dir)
        .spawn();
    // Linux: open a shell in the directory, never pass path as an argument.
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xterm")
        .args(["-e", std::env::var("SHELL").as_deref().unwrap_or("/bin/sh")])
        .current_dir(&dir)
        .spawn();
}

#[tauri::command]
pub fn open_folder(path: String) {
    let Some(dir) = safe_dir(&path) else { return };
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(&dir).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer").arg(&dir).spawn();
    // xdg-open: pass "--" to prevent path being parsed as a flag.
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg("--").arg(&dir).spawn();
}
