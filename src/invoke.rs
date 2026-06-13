use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::models::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

async fn call<T: for<'de> serde::Deserialize<'de>>(cmd: &str, args: impl Serialize) -> Option<T> {
    let args = serde_wasm_bindgen::to_value(&args).ok()?;
    let result = invoke(cmd, args).await;
    serde_wasm_bindgen::from_value(result).ok()
}

pub async fn list_folders() -> Vec<Folder> {
    call::<Vec<Folder>>("list_folders", ()).await.unwrap_or_default()
}

pub async fn list_all_repos() -> Vec<RepoSummary> {
    call::<Vec<RepoSummary>>("list_all_repos", ()).await.unwrap_or_default()
}

pub async fn get_repo_detail(path: String) -> Option<RepoDetail> {
    #[derive(Serialize)]
    struct Args { path: String }
    call("get_repo_detail", Args { path }).await
}

pub async fn add_folder(name: String, icon: String, color: String) -> Option<Folder> {
    #[derive(Serialize)]
    struct Args { name: String, icon: String, color: String }
    call("add_folder", Args { name, icon, color }).await
}

pub async fn add_repo(folder_id: String, path: String, description: String) -> Option<RepoSummary> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args { folder_id: String, path: String, description: String }
    call("add_repo", Args { folder_id, path, description }).await
}

pub async fn toggle_favorite(repo_id: String) -> bool {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args { repo_id: String }
    call("toggle_favorite", Args { repo_id }).await.unwrap_or(false)
}

pub async fn detect_ides() -> Vec<Editor> {
    call::<Vec<Editor>>("detect_ides", ()).await.unwrap_or_default()
}

pub async fn open_in(ide_id: String, path: String) {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args { ide_id: String, path: String }
    let _ = call::<()>("open_in", Args { ide_id, path }).await;
}

pub async fn fetch_repo(path: String) -> Option<SyncResult> {
    #[derive(Serialize)]
    struct Args { path: String }
    call("fetch", Args { path }).await
}

pub async fn pull_repo(path: String) -> Option<SyncResult> {
    #[derive(Serialize)]
    struct Args { path: String }
    call("pull", Args { path }).await
}

pub async fn pick_folder() -> Option<String> {
    call("pick_folder", ()).await
}

pub async fn scan_dir(parent: String) -> Vec<String> {
    #[derive(Serialize)]
    struct Args { parent: String }
    call("scan_dir", Args { parent }).await.unwrap_or_default()
}

pub async fn add_custom_editor(id: String, name: String, path: String, cmd: String) {
    #[derive(Serialize)]
    struct Args { id: String, name: String, path: String, cmd: String }
    let _ = call::<()>("add_custom_editor", Args { id, name, path, cmd }).await;
}

pub async fn open_terminal(path: String) {
    #[derive(Serialize)]
    struct Args { path: String }
    let _ = call::<()>("open_terminal", Args { path }).await;
}

pub async fn open_folder(path: String) {
    #[derive(Serialize)]
    struct Args { path: String }
    let _ = call::<()>("open_folder", Args { path }).await;
}
