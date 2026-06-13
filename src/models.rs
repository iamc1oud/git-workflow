use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Language {
    pub name: String,
    pub pct: u8,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusFile {
    pub path: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkingTreeStatus {
    pub clean: bool,
    pub staged: u32,
    pub modified: u32,
    pub untracked: u32,
    pub ahead: u32,
    pub behind: u32,
    pub files: Vec<StatusFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepoSummary {
    pub id: String,
    pub name: String,
    pub folder_id: String,
    pub path: String,
    pub description: String,
    pub remote: String,
    pub remote_host: String,
    pub default_branch: String,
    pub branch: String,
    pub favorite: bool,
    pub last_fetched: Option<String>,
    pub last_commit_at: Option<String>,
    pub languages: Vec<Language>,
    pub status: WorkingTreeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Branch {
    pub name: String,
    pub current: bool,
    pub ahead: u32,
    pub behind: u32,
    pub last_msg: String,
    pub when: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Commit {
    pub hash: String,
    pub msg: String,
    pub author: String,
    pub date: Option<String>,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Contributor {
    pub name: String,
    pub handle: String,
    pub commits: u32,
    pub additions: u32,
    pub deletions: u32,
    pub last_active: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepoDetail {
    pub id: String,
    pub name: String,
    pub folder_id: String,
    pub path: String,
    pub description: String,
    pub remote: String,
    pub remote_host: String,
    pub default_branch: String,
    pub branch: String,
    pub favorite: bool,
    pub last_fetched: Option<String>,
    pub last_commit_at: Option<String>,
    pub languages: Vec<Language>,
    pub status: WorkingTreeStatus,
    pub branches: Vec<Branch>,
    pub commits: Vec<Commit>,
    pub contributors: Vec<Contributor>,
    pub readme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Editor {
    pub id: String,
    pub name: String,
    pub detected: bool,
    pub path: String,
    pub accent: String,
    pub mark: String,
    pub cmd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
}

// ── Navigation state ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum NavKind {
    Smart,
    Folder,
    Repo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selection {
    pub kind: NavKind,
    pub id: String,
}

impl Selection {
    pub fn smart(id: &str) -> Self {
        Self { kind: NavKind::Smart, id: id.to_string() }
    }
    pub fn folder(id: &str) -> Self {
        Self { kind: NavKind::Folder, id: id.to_string() }
    }
    pub fn repo(id: &str) -> Self {
        Self { kind: NavKind::Repo, id: id.to_string() }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn glyph_for(name: &str) -> String {
    let parts: Vec<&str> = name.split(|c: char| !c.is_alphanumeric()).filter(|s| !s.is_empty()).collect();
    let m = if parts.len() > 1 {
        format!("{}{}", parts[0].chars().next().unwrap_or(' '), parts[1].chars().next().unwrap_or(' '))
    } else {
        name.chars().take(2).collect()
    };
    m.to_uppercase()
}

pub fn status_kind(repo: &RepoSummary) -> (&'static str, String) {
    let s = &repo.status;
    if !s.clean {
        let n = s.staged + s.modified + s.untracked;
        return ("dirty", format!("{} change{}", n, if n == 1 { "" } else { "s" }));
    }
    if s.ahead > 0 { return ("ahead", format!("{} ahead", s.ahead)); }
    if s.behind > 0 { return ("behind", format!("{} behind", s.behind)); }
    ("clean", "clean".into())
}

pub fn is_stale(last_commit_at: &Option<String>) -> bool {
    let iso = match last_commit_at {
        Some(s) => s,
        None => return true,
    };
    let now = js_sys::Date::now();
    let dt = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(iso));
    let diff_days = (now - dt.get_time()) / 86_400_000.0;
    diff_days > 30.0
}

pub fn from_now(iso: &Option<String>) -> String {
    let iso = match iso {
        Some(s) if !s.is_empty() => s,
        _ => return String::new(),
    };
    let now = js_sys::Date::now();
    let dt = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(iso));
    let diff = (now - dt.get_time()) / 1000.0;
    if diff < 60.0 { return "just now".into(); }
    let m = diff / 60.0;
    if m < 60.0 { return format!("{}m ago", m as u32); }
    let h = m / 60.0;
    if h < 24.0 { return format!("{}h ago", h as u32); }
    let d = h / 24.0;
    if d < 7.0 { return format!("{}d ago", d as u32); }
    let w = d / 7.0;
    if w < 5.0 { return format!("{}w ago", w as u32); }
    let mo = d / 30.0;
    if mo < 12.0 { return format!("{}mo ago", mo as u32); }
    format!("{}y ago", (d / 365.0) as u32)
}
