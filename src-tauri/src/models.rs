use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub name: String,
    pub pct: u8,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusFile {
    pub path: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingTreeStatus {
    pub clean: bool,
    pub staged: u32,
    pub modified: u32,
    pub untracked: u32,
    pub ahead: u32,
    pub behind: u32,
    pub files: Vec<StatusFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub current: bool,
    pub ahead: u32,
    pub behind: u32,
    pub last_msg: String,
    pub when: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,
    pub msg: String,
    pub author: String,
    pub date: Option<String>,
    pub additions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contributor {
    pub name: String,
    pub handle: String,
    pub commits: u32,
    pub additions: u32,
    pub deletions: u32,
    pub last_active: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoDetail {
    #[serde(flatten)]
    pub summary: RepoSummary,
    pub branches: Vec<Branch>,
    pub commits: Vec<Commit>,
    pub contributors: Vec<Contributor>,
    pub readme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Editor {
    pub id: String,
    pub name: String,
    pub detected: bool,
    pub path: String,
    pub accent: String,
    pub mark: String,
    pub cmd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub repo_id: String,
    pub repo_name: String,
    pub hash: String,
    pub msg: String,
    pub author: String,
    pub date: Option<String>,
    pub additions: u32,
    pub deletions: u32,
}
