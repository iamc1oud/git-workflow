use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::models::{Editor, Folder};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoRecord {
    pub id: String,
    pub folder_id: String,
    pub path: String,
    pub favorite: bool,
    pub description: String,
    pub last_fetched: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppData {
    pub folders: Vec<Folder>,
    pub repos: Vec<RepoRecord>,
    pub custom_editors: Vec<Editor>,
}

pub struct Store {
    data_dir: PathBuf,
    pub data: AppData,
}

impl Store {
    pub fn load(data_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&data_dir).ok();
        let data = Self::read(&data_dir);
        Self { data_dir, data }
    }

    fn read(dir: &PathBuf) -> AppData {
        let path = dir.join("appdata.json");
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = self.data_dir.join("appdata.json");
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            std::fs::write(path, json).ok();
        }
    }

    // ── Folders ──────────────────────────────────────────────────────────────

    pub fn add_folder(&mut self, name: String, icon: String, color: String) -> Folder {
        let id = format!("{}-{}", slugify(&name), self.data.folders.len());
        let folder = Folder { id, name, icon, color };
        self.data.folders.push(folder.clone());
        self.save();
        folder
    }

    pub fn remove_folder(&mut self, folder_id: &str) {
        self.data.folders.retain(|f| f.id != folder_id);
        self.data.repos.retain(|r| r.folder_id != folder_id);
        self.save();
    }

    // ── Repos ─────────────────────────────────────────────────────────────────

    pub fn add_repo(
        &mut self,
        folder_id: String,
        path: String,
        description: String,
    ) -> Option<RepoRecord> {
        // Idempotent: return existing if already tracked
        if let Some(existing) = self.data.repos.iter().find(|r| r.path == path) {
            return Some(existing.clone());
        }
        let name = std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repo")
            .to_string();
        let id = format!("{}-{}", slugify(&name), self.data.repos.len());
        let record = RepoRecord { id, folder_id, path, favorite: false, description, last_fetched: None };
        self.data.repos.push(record.clone());
        self.save();
        Some(record)
    }

    pub fn remove_repo(&mut self, repo_id: &str) {
        self.data.repos.retain(|r| r.id != repo_id);
        self.save();
    }

    pub fn toggle_favorite(&mut self, repo_id: &str) -> bool {
        let new_fav = self
            .data
            .repos
            .iter_mut()
            .find(|r| r.id == repo_id)
            .map(|r| {
                r.favorite = !r.favorite;
                r.favorite
            });
        if let Some(fav) = new_fav {
            self.save();
            return fav;
        }
        false
    }

    pub fn update_last_fetched(&mut self, path: &str) {
        use chrono::Utc;
        let now = Utc::now().to_rfc3339();
        if let Some(r) = self.data.repos.iter_mut().find(|r| r.path == path) {
            r.last_fetched = Some(now);
            self.save();
        }
    }

    pub fn update_description(&mut self, repo_id: &str, description: String) {
        if let Some(r) = self.data.repos.iter_mut().find(|r| r.id == repo_id) {
            r.description = description;
            self.save();
        }
    }

    // ── Editors ───────────────────────────────────────────────────────────────

    pub fn add_editor(&mut self, editor: Editor) {
        self.data.custom_editors.push(editor);
        self.save();
    }

    pub fn remove_editor(&mut self, editor_id: &str) {
        self.data.custom_editors.retain(|e| e.id != editor_id);
        self.save();
    }
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
