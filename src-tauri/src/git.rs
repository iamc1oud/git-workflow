use git2::{BranchType, Repository, Sort, StatusOptions};
use std::collections::HashMap;

use crate::models::{Branch, Commit, Contributor, Language, StatusFile, WorkingTreeStatus};

// ── Status ────────────────────────────────────────────────────────────────────

pub fn get_status(path: &str) -> WorkingTreeStatus {
    let empty = WorkingTreeStatus {
        clean: true,
        staged: 0,
        modified: 0,
        untracked: 0,
        ahead: 0,
        behind: 0,
        files: vec![],
    };

    let repo = match Repository::open(path) {
        Ok(r) => r,
        Err(_) => return empty,
    };

    let mut opts = StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(false);

    let statuses = match repo.statuses(Some(&mut opts)) {
        Ok(s) => s,
        Err(_) => return empty,
    };

    let mut staged = 0u32;
    let mut modified = 0u32;
    let mut untracked = 0u32;
    let mut files: Vec<StatusFile> = vec![];

    for entry in statuses.iter() {
        let s = entry.status();
        let file_path = entry.path().unwrap_or("").to_string();

        let is_staged = s.intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED,
        );
        let is_modified = s.intersects(
            git2::Status::WT_MODIFIED | git2::Status::WT_DELETED | git2::Status::WT_RENAMED,
        );
        let is_untracked = s.contains(git2::Status::WT_NEW);

        if is_staged {
            staged += 1;
            files.push(StatusFile { path: file_path, kind: "staged".into() });
        } else if is_untracked {
            untracked += 1;
            files.push(StatusFile { path: file_path, kind: "untracked".into() });
        } else if is_modified {
            modified += 1;
            files.push(StatusFile { path: file_path, kind: "modified".into() });
        }
    }

    let (ahead, behind) = ahead_behind_head(&repo);
    let clean = staged == 0 && modified == 0 && untracked == 0;

    WorkingTreeStatus { clean, staged, modified, untracked, ahead, behind, files }
}

fn ahead_behind_head(repo: &Repository) -> (u32, u32) {
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return (0, 0),
    };
    let branch = head.shorthand().unwrap_or("HEAD");
    let local_ref = format!("refs/heads/{}", branch);
    let remote_ref = format!("refs/remotes/origin/{}", branch);

    let local_oid = match repo.refname_to_id(&local_ref) {
        Ok(o) => o,
        Err(_) => return (0, 0),
    };
    let remote_oid = match repo.refname_to_id(&remote_ref) {
        Ok(o) => o,
        Err(_) => return (0, 0),
    };

    match repo.graph_ahead_behind(local_oid, remote_oid) {
        Ok((a, b)) => (a as u32, b as u32),
        Err(_) => (0, 0),
    }
}

// ── Commits ───────────────────────────────────────────────────────────────────

pub fn get_commits(path: &str, branch: &str) -> Vec<Commit> {
    let repo = match Repository::open(path) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let mut revwalk = match repo.revwalk() {
        Ok(rw) => rw,
        Err(_) => return vec![],
    };

    // Push the specified branch tip; fall back to HEAD
    let branch_ref = format!("refs/heads/{}", branch);
    if repo.refname_to_id(&branch_ref).and_then(|oid| revwalk.push(oid)).is_err() {
        if revwalk.push_head().is_err() {
            return vec![];
        }
    }

    revwalk.set_sorting(Sort::TIME).ok();

    let mut commits = vec![];

    for (i, oid_result) in revwalk.enumerate() {
        if i >= 50 { break; }

        let oid = match oid_result {
            Ok(o) => o,
            Err(_) => continue,
        };
        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let msg = commit.summary().unwrap_or("").to_string();
        let author = commit.author().name().unwrap_or("Unknown").to_string();
        let ts = commit.time().seconds();
        let date = ts_to_iso(ts);

        let (additions, deletions) = if i < 30 {
            diff_stats(&repo, &commit)
        } else {
            (0, 0)
        };

        commits.push(Commit {
            hash: format!("{:.7}", oid),
            msg,
            author,
            date: Some(date),
            additions,
            deletions,
        });
    }

    commits
}

fn diff_stats(repo: &Repository, commit: &git2::Commit) -> (u32, u32) {
    let tree = match commit.tree() {
        Ok(t) => t,
        Err(_) => return (0, 0),
    };
    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
    let diff = match repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None) {
        Ok(d) => d,
        Err(_) => return (0, 0),
    };
    match diff.stats() {
        Ok(s) => (s.insertions() as u32, s.deletions() as u32),
        Err(_) => (0, 0),
    }
}

// ── Branches ──────────────────────────────────────────────────────────────────

pub fn get_branches(path: &str) -> Vec<Branch> {
    let repo = match Repository::open(path) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let iter = match repo.branches(Some(BranchType::Local)) {
        Ok(b) => b,
        Err(_) => return vec![],
    };

    let mut result = vec![];

    for branch_result in iter {
        let (branch, _) = match branch_result {
            Ok(b) => b,
            Err(_) => continue,
        };

        let name = match branch.name() {
            Ok(Some(n)) => n.to_string(),
            _ => continue,
        };
        let current = branch.is_head();

        let (last_msg, when) = match branch.get().peel_to_commit() {
            Ok(c) => (c.summary().unwrap_or("").to_string(), Some(ts_to_iso(c.time().seconds()))),
            Err(_) => (String::new(), None),
        };

        let (ahead, behind) = match branch.upstream() {
            Ok(upstream) => {
                let local_oid = branch.get().peel_to_commit().map(|c| c.id());
                let remote_oid = upstream.get().peel_to_commit().map(|c| c.id());
                match (local_oid, remote_oid) {
                    (Ok(l), Ok(r)) => repo
                        .graph_ahead_behind(l, r)
                        .map(|(a, b)| (a as u32, b as u32))
                        .unwrap_or((0, 0)),
                    _ => (0, 0),
                }
            }
            Err(_) => (0, 0),
        };

        result.push(Branch { name, current, ahead, behind, last_msg, when });
    }

    result
}

// ── Contributors ──────────────────────────────────────────────────────────────

pub fn get_contributors(commits: &[Commit]) -> Vec<Contributor> {
    struct Acc {
        commits: u32,
        additions: u32,
        deletions: u32,
        last_active: Option<String>,
    }

    let mut map: HashMap<String, Acc> = HashMap::new();

    for c in commits {
        let acc = map.entry(c.author.clone()).or_insert(Acc {
            commits: 0,
            additions: 0,
            deletions: 0,
            last_active: c.date.clone(),
        });
        acc.commits += 1;
        acc.additions += c.additions;
        acc.deletions += c.deletions;
        // commits are newest-first; first entry IS the last_active date
    }

    let mut result: Vec<Contributor> = map
        .into_iter()
        .map(|(name, acc)| Contributor {
            handle: make_handle(&name),
            name,
            commits: acc.commits,
            additions: acc.additions,
            deletions: acc.deletions,
            last_active: acc.last_active,
        })
        .collect();

    result.sort_by(|a, b| b.commits.cmp(&a.commits));
    result
}

fn make_handle(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(12)
        .collect()
}

// ── Remote ────────────────────────────────────────────────────────────────────

pub fn get_remote_url(path: &str) -> (String, String) {
    let repo = match Repository::open(path) {
        Ok(r) => r,
        Err(_) => return (String::new(), String::new()),
    };
    let remote = match repo.find_remote("origin") {
        Ok(r) => r,
        Err(_) => return (String::new(), String::new()),
    };
    let url = remote.url().unwrap_or("").to_string();
    let host = if url.contains("github.com") {
        "github"
    } else if url.contains("gitlab.com") {
        "gitlab"
    } else if url.contains("bitbucket.org") {
        "bitbucket"
    } else {
        "git"
    };
    let display = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("git@")
        .replace(':', "/")
        .trim_end_matches(".git")
        .to_string();
    (display, host.to_string())
}

// ── Misc helpers ──────────────────────────────────────────────────────────────

pub fn get_current_branch(path: &str) -> String {
    let repo = match Repository::open(path) {
        Ok(r) => r,
        Err(_) => return "HEAD".into(),
    };
    let name = match repo.head() {
        Ok(h) => h.shorthand().unwrap_or("HEAD").to_string(),
        Err(_) => "HEAD".into(),
    };
    name
}

pub fn get_default_branch(path: &str) -> String {
    let repo = match Repository::open(path) {
        Ok(r) => r,
        Err(_) => return "main".into(),
    };
    // Try origin/HEAD symbolic ref
    if let Ok(r) = repo.find_reference("refs/remotes/origin/HEAD") {
        if let Some(target) = r.symbolic_target() {
            return target.trim_start_matches("refs/remotes/origin/").to_string();
        }
    }
    if repo.refname_to_id("refs/heads/main").is_ok() { "main".into() }
    else if repo.refname_to_id("refs/heads/master").is_ok() { "master".into() }
    else { "main".into() }
}

pub fn get_last_commit_time(path: &str) -> Option<String> {
    let repo = Repository::open(path).ok()?;
    let head = repo.head().ok()?;
    let commit = head.peel_to_commit().ok()?;
    Some(ts_to_iso(commit.time().seconds()))
}

pub fn get_readme(path: &str) -> String {
    let base = std::path::Path::new(path);
    for name in &["README.md", "readme.md", "Readme.md", "README.MD", "README"] {
        let p = base.join(name);
        if p.exists() {
            return std::fs::read_to_string(p).unwrap_or_default();
        }
    }
    String::new()
}

fn ts_to_iso(secs: i64) -> String {
    use chrono::{TimeZone, Utc};
    Utc.timestamp_opt(secs, 0)
        .single()
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default()
}

// ── Language detection ────────────────────────────────────────────────────────

// (language name, hex color, file extensions)
const LANG_MAP: &[(&str, &str, &[&str])] = &[
    ("Rust",        "#DEA584", &["rs"]),
    ("Go",          "#00ADD8", &["go"]),
    ("TypeScript",  "#3178C6", &["ts", "tsx"]),
    ("JavaScript",  "#F1E05A", &["js", "jsx", "mjs", "cjs"]),
    ("Python",      "#3572A5", &["py"]),
    ("Swift",       "#F05138", &["swift"]),
    ("Kotlin",      "#A97BFF", &["kt", "kts"]),
    ("Java",        "#B07219", &["java"]),
    ("HCL",         "#844FBA", &["tf", "hcl"]),
    ("Shell",       "#89E051", &["sh", "bash", "zsh"]),
    ("CSS",         "#563D7C", &["css"]),
    ("SCSS",        "#C6538C", &["scss", "sass"]),
    ("HTML",        "#E34C26", &["html", "htm"]),
    ("Vue",         "#41B883", &["vue"]),
    ("Ruby",        "#701516", &["rb"]),
    ("YAML",        "#CB171E", &["yml", "yaml"]),
    ("SQL",         "#E38C00", &["sql"]),
    ("Markdown",    "#083FA1", &["md", "mdx"]),
    ("Dockerfile",  "#384D54", &["dockerfile"]),
    ("TOML",        "#9C4221", &["toml"]),
    ("JSON",        "#6B7280", &["json"]),
];

const SKIP_DIRS: &[&str] = &[
    ".git", "node_modules", "vendor", "target", "dist", "build",
    ".next", "__pycache__", ".gradle", "Pods", "DerivedData", ".build",
];

pub fn detect_languages(path: &str) -> Vec<Language> {
    use walkdir::WalkDir;

    let mut counts: HashMap<&str, u64> = HashMap::new();
    let mut total: u64 = 0;

    for entry in WalkDir::new(path)
        .max_depth(8)
        .into_iter()
        .filter_entry(|e| {
            e.file_type().is_file()
                || e.file_name()
                    .to_str()
                    .map(|n| !SKIP_DIRS.iter().any(|&s| s == n))
                    .unwrap_or(true)
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let fname = entry.file_name().to_str().unwrap_or("").to_lowercase();

        // "Dockerfile" has no extension
        let lang = if fname == "dockerfile" {
            Some("Dockerfile")
        } else {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_lowercase())
                .and_then(|ext| {
                    LANG_MAP
                        .iter()
                        .find(|(_, _, exts)| exts.iter().any(|&e| e == ext))
                        .map(|(name, _, _)| *name)
                })
        };

        if let Some(name) = lang {
            let size = entry.metadata().map(|m| m.len()).unwrap_or(1);
            *counts.entry(name).or_insert(0) += size;
            total += size;
        }
    }

    if total == 0 {
        return vec![];
    }

    let mut langs: Vec<(String, u8, String)> = counts
        .into_iter()
        .map(|(name, bytes)| {
            let pct = ((bytes as f64 / total as f64) * 100.0).round() as u8;
            let color = LANG_MAP
                .iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, c, _)| c.to_string())
                .unwrap_or_else(|| "#888".into());
            (name.to_string(), pct, color)
        })
        .filter(|(_, pct, _)| *pct > 0)
        .collect();

    langs.sort_by(|a, b| b.1.cmp(&a.1));
    langs.truncate(6);

    // Normalize sum to 100 by adjusting the top entry
    let sum: u8 = langs.iter().map(|(_, p, _)| p).sum();
    if sum > 0 && !langs.is_empty() {
        let diff = 100i16 - sum as i16;
        langs[0].1 = (langs[0].1 as i16 + diff).max(1) as u8;
    }

    langs
        .into_iter()
        .map(|(name, pct, color)| Language { name, pct, color })
        .collect()
}

// ── Branch checkout ───────────────────────────────────────────────────────────

pub fn checkout_branch(path: &str, branch: &str) -> Result<(), String> {
    let repo = Repository::open(path).map_err(|e| e.message().to_string())?;
    let obj = repo
        .revparse_single(&format!("refs/heads/{}", branch))
        .map_err(|e| e.message().to_string())?;
    repo.checkout_tree(&obj, None)
        .map_err(|e| e.message().to_string())?;
    repo.set_head(&format!("refs/heads/{}", branch))
        .map_err(|e| e.message().to_string())?;
    Ok(())
}

// ── Directory scan ────────────────────────────────────────────────────────────

pub fn scan_dir_for_repos(parent: &str) -> Vec<String> {
    use walkdir::WalkDir;

    let mut repos = vec![];

    for entry in WalkDir::new(parent)
        .max_depth(4)
        .into_iter()
        .filter_entry(|e| {
            e.file_name()
                .to_str()
                .map(|n| !["node_modules", "vendor", "target"].iter().any(|&s| s == n))
                .unwrap_or(true)
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() && entry.file_name() == ".git" {
            if let Some(parent) = entry.path().parent() {
                repos.push(parent.to_string_lossy().to_string());
            }
        }
    }

    repos
}
