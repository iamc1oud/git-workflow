use dioxus::prelude::*;

use crate::{
    icons::icon_html,
    invoke,
    models::*,
    shared::{AvatarStack, StatusPill, Toast},
};

#[component]
pub fn DetailPanel(
    repo: RepoSummary,
    editors: Vec<Editor>,
    sel: Signal<Selection>,
    toasts: Signal<Vec<Toast>>,
) -> Element {
    let mut tab = use_signal(|| "overview");
    let mut detail = use_signal(|| Option::<RepoDetail>::None);
    let mut loading = use_signal(|| true);

    let path = repo.path.clone();

    use_effect(move || {
        let p = path.clone();
        spawn(async move {
            loading.set(true);
            let d = invoke::get_repo_detail(p).await;
            detail.set(d);
            loading.set(false);
        });
    });

    let t = *tab.read();
    let (commit_count, branch_count, contrib_count) = {
        let d = detail.read();
        (
            d.as_ref().map_or(0, |d| d.commits.len()),
            d.as_ref().map_or(0, |d| d.branches.len()),
            d.as_ref().map_or(0, |d| d.contributors.len()),
        )
    };

    rsx! {
        div { class: "detail",
            // Header
            DetailHeader {
                repo: repo.clone(),
                editors: editors.clone(),
                toasts,
            }

            // Tabs
            div { class: "tabs",
                for (id, label, icon, count) in [
                    ("overview",  "Overview",     "eye",    0usize),
                    ("commits",   "Commits",       "commit", commit_count),
                    ("branches",  "Branches",      "branch", branch_count),
                    ("contribs",  "Contributors",  "users",  contrib_count),
                    ("readme",    "README",         "book",   0),
                ] {
                    button {
                        class: if t == id { "tab active" } else { "tab" },
                        onclick: move |_| tab.set(id),
                        span { class: "tab-ico", dangerous_inner_html: "{icon_html(icon, 14)}" }
                        "{label}"
                        if count > 0 {
                            span { class: "tcount", "{count}" }
                        }
                    }
                }
            }

            // Body
            div { class: "tab-body scroll",
                if *loading.read() {
                    div { class: "loading-state",
                        span { class: "spinner" }
                    }
                } else if let Some(d) = detail.read().clone() {
                    match t {
                        "commits" => rsx! { CommitsTab { commits: d.commits.clone(), branch: repo.branch.clone() } },
                        "branches" => rsx! { BranchesTab {
                            branches: d.branches.clone(),
                            default_branch: repo.default_branch.clone(),
                            repo_path: repo.path.clone(),
                            toasts,
                        } },
                        "contribs" => rsx! { ContribsTab { contributors: d.contributors.clone() } },
                        "readme" => rsx! { ReadmeTab { readme: d.readme.clone() } },
                        _ => rsx! { OverviewTab { detail: d, repo: repo.clone(), tab } },
                    }
                } else {
                    div { class: "empty-state",
                        span { dangerous_inner_html: "{icon_html(\"x\", 32)}" }
                        p { "Failed to load repo" }
                    }
                }
            }
        }
    }
}

// ── Header ────────────────────────────────────────────────────────────────────

#[component]
fn DetailHeader(
    repo: RepoSummary,
    editors: Vec<Editor>,
    toasts: Signal<Vec<Toast>>,
) -> Element {
    let glyph = glyph_for(&repo.name);
    let repo2 = repo.clone();
    let mut editor_dropdown_open = use_signal(|| false);
    let mut fetching = use_signal(|| false);
    let mut pulling = use_signal(|| false);
    let primary_ed = editors.first().cloned();
    let has_more_editors = editors.len() > 1;
    let fetch_path = repo.path.clone();
    let pull_path = repo.path.clone();

    rsx! {
        div { class: "detail-head",
            // ── Top: glyph + info ─────────────────────────────────────────────
            div { class: "dh-top",
                div { class: "dh-glyph", style: "background:#7C6BFF", "{glyph}" }
                div { class: "dh-titles",
                    // Name row: name + star + status pill
                    div { class: "dh-name",
                        span { "{repo.name}" }
                        if repo.favorite {
                            span { class: "dh-star", dangerous_inner_html: "{icon_html(\"star\", 16)}" }
                        }
                        StatusPill { repo: repo2.clone() }
                    }
                    // Description
                    if !repo.description.is_empty() {
                        div { class: "dh-desc", "{repo.description}" }
                    }
                    // Path
                    if !repo.path.is_empty() {
                        div { class: "dh-path",
                            span { class: "icon-inline", dangerous_inner_html: "{icon_html(\"folder\", 11)}" }
                            span { class: "mono", style: "overflow:hidden;text-overflow:ellipsis;white-space:nowrap", "{repo.path}" }
                        }
                    }
                }
            }

            // ── Actions row ───────────────────────────────────────────────────
            div { class: "dh-actions",
                // Open in editor (split button)
                if let Some(ref primary_ed) = primary_ed {
                    div { class: "split",
                        button {
                            class: "btn-primary",
                            style: "background:{primary_ed.accent};border-color:{primary_ed.accent}",
                            title: "Open in {primary_ed.name}",
                            onclick: {
                                let path = repo.path.clone();
                                let eid = primary_ed.id.clone();
                                move |_| {
                                    let p = path.clone();
                                    let e = eid.clone();
                                    spawn(async move { invoke::open_in(e, p).await; });
                                }
                            },
                            span { dangerous_inner_html: "{primary_ed.mark}" }
                            "Open in {primary_ed.name}"
                        }
                        if has_more_editors {
                            button {
                                class: "btn-primary",
                                style: "background:{primary_ed.accent};border-color:{primary_ed.accent}",
                                title: "More editors",
                                onclick: move |_| editor_dropdown_open.toggle(),
                                span { dangerous_inner_html: "{icon_html(\"chevronDown\", 14)}" }
                            }
                            if *editor_dropdown_open.read() {
                                div { class: "dropdown",
                                    for ed in editors.iter().skip(1) {
                                        button {
                                            class: "dropdown-item",
                                            onclick: {
                                                let path = repo.path.clone();
                                                let eid = ed.id.clone();
                                                move |_| {
                                                    let p = path.clone();
                                                    let e = eid.clone();
                                                    spawn(async move { invoke::open_in(e, p).await; });
                                                    editor_dropdown_open.set(false);
                                                }
                                            },
                                            span { dangerous_inner_html: "{ed.mark}" }
                                            "{ed.name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                button {
                    class: if *fetching.read() { "btn btn-syncing" } else { "btn" },
                    title: "Fetch",
                    disabled: *fetching.read() || *pulling.read(),
                    onclick: {
                        let p = fetch_path.clone();
                        move |_| {
                            let p = p.clone();
                            spawn(async move {
                                fetching.set(true);
                                if let Some(r) = invoke::fetch_repo(p).await {
                                    let mut t = toasts.write();
                                    t.push(Toast::new(
                                        if r.success { "Fetched" } else { "Fetch failed" },
                                        &r.message,
                                    ));
                                }
                                fetching.set(false);
                            });
                        }
                    },
                    span { class: if *fetching.read() { "spin" } else { "" }, dangerous_inner_html: "{icon_html(\"refresh\", 16)}" }
                    if *fetching.read() { "Fetching…" } else { "Fetch" }
                }
                button {
                    class: if *pulling.read() { "btn btn-syncing" } else { "btn" },
                    title: "Pull",
                    disabled: *fetching.read() || *pulling.read(),
                    onclick: {
                        let p = pull_path.clone();
                        move |_| {
                            let p = p.clone();
                            spawn(async move {
                                pulling.set(true);
                                if let Some(r) = invoke::pull_repo(p).await {
                                    let mut t = toasts.write();
                                    t.push(Toast::new(
                                        if r.success { "Pulled" } else { "Pull failed" },
                                        &r.message,
                                    ));
                                }
                                pulling.set(false);
                            });
                        }
                    },
                    span { class: if *pulling.read() { "spin" } else { "" }, dangerous_inner_html: "{icon_html(\"arrowDown\", 14)}" }
                    if *pulling.read() { "Pulling…" } else { "Pull" }
                }
                button {
                    class: "tb-btn",
                    title: "Open in Terminal",
                    onclick: {
                        let path = repo.path.clone();
                        move |_| {
                            let p = path.clone();
                            spawn(async move { invoke::open_terminal(p).await; });
                        }
                    },
                    span { dangerous_inner_html: "{icon_html(\"terminal\", 16)}" }
                }
                button {
                    class: "tb-btn",
                    title: "Open Folder",
                    onclick: {
                        let path = repo.path.clone();
                        move |_| {
                            let p = path.clone();
                            spawn(async move { invoke::open_folder(p).await; });
                        }
                    },
                    span { dangerous_inner_html: "{icon_html(\"folderOpen\", 16)}" }
                }
            }
        }
    }
}

// ── Overview tab ──────────────────────────────────────────────────────────────

#[component]
fn OverviewTab(detail: RepoDetail, repo: RepoSummary, tab: Signal<&'static str>) -> Element {
    let s = &repo.status;
    let more_files = if s.files.len() > 5 { s.files.len() - 5 } else { 0 };
    let total_commits: u32 = detail.contributors.iter().map(|c| c.commits).sum();
    let fetched = from_now(&repo.last_fetched);
    let fetched_text = if fetched.is_empty() { "Never".to_string() } else { fetched };
    let contrib_count = detail.contributors.len();

    rsx! {
        div { class: "overview",
            // Top grid: Working Tree + Sync
            div { class: "ov-top-grid",
                // Working Tree
                div { class: "ov-card",
                    div { class: "ov-card-hdr",
                        span { class: "ov-card-ico", dangerous_inner_html: "{icon_html(\"diff\", 13)}" }
                        span { "WORKING TREE" }
                        span { class: "ov-branch-tag",
                            span { dangerous_inner_html: "{icon_html(\"branch\", 11)}" }
                            span { class: "mono", "{repo.branch}" }
                        }
                    }
                    div { class: "ov-card-body",
                        div { class: "wt-chips",
                            if s.staged > 0 {
                                span { class: "wt-chip wt-staged", "• {s.staged} staged" }
                            }
                            if s.modified > 0 {
                                span { class: "wt-chip wt-modified", "• {s.modified} modified" }
                            }
                            if s.untracked > 0 {
                                span { class: "wt-chip wt-untracked", "• {s.untracked} untracked" }
                            }
                            if s.staged == 0 && s.modified == 0 && s.untracked == 0 {
                                span { class: "wt-chip wt-clean", "• Clean" }
                            }
                        }
                        if !s.files.is_empty() {
                            div { class: "wt-files",
                                for f in s.files.iter().take(5) {
                                    div { class: "wt-file-row",
                                        span { class: "wt-badge wt-{f.kind}",
                                            "{f.kind.chars().next().unwrap_or('?').to_uppercase()}"
                                        }
                                        span { class: "mono wt-path", "{f.path}" }
                                    }
                                }
                            }
                            if more_files > 0 {
                                div { class: "wt-more", "+ {more_files} more" }
                            }
                        }
                    }
                }
                // Sync
                div { class: "ov-card",
                    div { class: "ov-card-hdr",
                        span { class: "ov-card-ico", dangerous_inner_html: "{icon_html(\"refresh\", 13)}" }
                        "SYNC"
                    }
                    div { class: "sync-rows",
                        div { class: "sync-row",
                            span { class: "sync-lbl", "Ahead of remote" }
                            if s.ahead > 0 {
                                span { class: "sync-val sync-ahead",
                                    span { dangerous_inner_html: "{icon_html(\"arrowUp\", 12)}" }
                                    "{s.ahead} commits"
                                }
                            } else {
                                span { class: "sync-val sync-dash", "—" }
                            }
                        }
                        div { class: "sync-row",
                            span { class: "sync-lbl", "Behind remote" }
                            if s.behind > 0 {
                                span { class: "sync-val sync-behind",
                                    span { dangerous_inner_html: "{icon_html(\"arrowDown\", 12)}" }
                                    "{s.behind} commits"
                                }
                            } else {
                                span { class: "sync-val sync-dash", "—" }
                            }
                        }
                        div { class: "sync-row",
                            span { class: "sync-lbl", "Last fetched" }
                            span { class: "sync-val", "{fetched_text}" }
                        }
                        if !repo.remote.is_empty() {
                            div { class: "sync-row",
                                span { class: "sync-lbl", "Remote" }
                                span { class: "sync-val sync-remote",
                                    "{repo.remote}"
                                    span { class: "ov-card-ico", dangerous_inner_html: "{icon_html(\"external\", 11)}" }
                                }
                            }
                        }
                    }
                }
            }

            // Languages
            if !detail.languages.is_empty() {
                div { class: "ov-card",
                    div { class: "ov-card-hdr",
                        span { class: "ov-card-ico", dangerous_inner_html: "{icon_html(\"code\", 13)}" }
                        "LANGUAGES"
                    }
                    div { class: "ov-card-body",
                        div { class: "lang-bar full",
                            for lang in detail.languages.iter() {
                                div {
                                    class: "lang-seg",
                                    style: "width:{lang.pct}%;background:{lang.color}",
                                    title: "{lang.name} {lang.pct}%"
                                }
                            }
                        }
                        div { class: "lang-legend",
                            for lang in detail.languages.iter() {
                                div { class: "lang-item",
                                    span { class: "lang-dot", style: "background:{lang.color}" }
                                    "{lang.name} {lang.pct}%"
                                }
                            }
                        }
                    }
                }
            }

            // Recent Commits
            if !detail.commits.is_empty() {
                div { class: "ov-card",
                    div { class: "ov-card-hdr",
                        span { class: "ov-card-ico", dangerous_inner_html: "{icon_html(\"commit\", 13)}" }
                        "RECENT COMMITS"
                        button {
                            class: "ov-view-all",
                            onclick: move |_| tab.set("commits"),
                            "View all"
                            span { dangerous_inner_html: "{icon_html(\"chevronRight\", 11)}" }
                        }
                    }
                    div { class: "ov-commit-list",
                        for c in detail.commits.iter().take(5) {
                            OvCommitRow { commit: c.clone() }
                        }
                    }
                }
            }

            // Contributors
            if !detail.contributors.is_empty() {
                div { class: "ov-card",
                    div { class: "ov-card-hdr",
                        span { class: "ov-card-ico", dangerous_inner_html: "{icon_html(\"users\", 13)}" }
                        "TOP CONTRIBUTORS"
                        button {
                            class: "ov-view-all",
                            onclick: move |_| tab.set("contribs"),
                            "View all"
                            span { dangerous_inner_html: "{icon_html(\"chevronRight\", 11)}" }
                        }
                    }
                    div { class: "ov-contrib-preview",
                        AvatarStack {
                            contributors: detail.contributors.clone(),
                            max: 6,
                            size: 32,
                        }
                        span { class: "ov-contrib-meta",
                            strong { "{contrib_count}" }
                            " contributors  "
                            strong { "{total_commits}" }
                            " commits"
                        }
                    }
                }
            }
        }
    }
}

// ── Overview commit row (with avatar) ────────────────────────────────────────

#[component]
fn OvCommitRow(commit: Commit) -> Element {
    let hash_short = commit.hash.chars().take(7).collect::<String>();
    let when = from_now(&commit.date);
    let initials = commit.author
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    let colors = ["#6366F1","#EC4899","#14B8A6","#F59E0B","#8B5CF6","#06B6D4","#F43F5E","#22C55E"];
    let av_color = colors[commit.author.bytes().fold(0usize, |a, b| a.wrapping_add(b as usize)) % colors.len()];

    rsx! {
        div { class: "ov-commit-row",
            div { class: "ov-commit-av", style: "background:{av_color}", "{initials}" }
            div { class: "ov-commit-body",
                div { class: "ov-commit-msg", "{commit.msg}" }
                div { class: "ov-commit-meta",
                    span { "{commit.author}" }
                    span { class: "ov-hash-chip", "{hash_short}" }
                    if !when.is_empty() { span { "· {when}" } }
                }
            }
            if commit.additions > 0 || commit.deletions > 0 {
                div { class: "ov-commit-stats",
                    if commit.additions > 0 { span { class: "adds", "+{commit.additions}" } }
                    if commit.deletions > 0 { span { class: "dels", "-{commit.deletions}" } }
                }
            }
        }
    }
}

// ── Commits tab ───────────────────────────────────────────────────────────────

#[component]
fn CommitsTab(commits: Vec<Commit>, branch: String) -> Element {
    let count = commits.len();
    rsx! {
        div { class: "commits-tab",
            div { class: "commits-tab-hdr",
                span { class: "ct-hdr-ico", dangerous_inner_html: "{icon_html(\"commit\", 14)}" }
                span { class: "ct-count mono", "{count}" }
                span { class: "ct-label", "COMMITS ON" }
                span { class: "ct-branch mono", "{branch}" }
            }
            div { class: "ct-list",
                if commits.is_empty() {
                    div { class: "empty-state", p { "No commits" } }
                }
                for c in commits.iter() {
                    CommitRow { commit: c.clone() }
                }
            }
        }
    }
}

#[component]
fn CommitRow(commit: Commit) -> Element {
    let hash_short = commit.hash.chars().take(7).collect::<String>();
    let when = from_now(&commit.date);
    let initials = commit.author
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    let colors = ["#6366F1","#EC4899","#14B8A6","#F59E0B","#8B5CF6","#06B6D4","#F43F5E","#22C55E"];
    let av_color = colors[commit.author.bytes().fold(0usize, |a, b| a.wrapping_add(b as usize)) % colors.len()];

    rsx! {
        div { class: "ct-row",
            div { class: "ct-avatar", style: "background:{av_color}", "{initials}" }
            div { class: "ct-body",
                div { class: "ct-msg", "{commit.msg}" }
                div { class: "ct-meta",
                    span { "{commit.author}" }
                    span { class: "ct-hash mono", "{hash_short}" }
                    if !when.is_empty() {
                        span { "· {when}" }
                    }
                }
            }
            div { class: "ct-stats",
                span { class: "adds", "+{commit.additions}" }
                span { class: "dels", "-{commit.deletions}" }
            }
        }
    }
}

// ── Branches tab ─────────────────────────────────────────────────────────────

#[component]
fn BranchesTab(
    branches: Vec<Branch>,
    default_branch: String,
    repo_path: String,
    toasts: Signal<Vec<Toast>>,
) -> Element {
    let count = branches.len();
    rsx! {
        div { class: "branches-tab",
            div { class: "branches-tab-hdr",
                span { class: "bt-hdr-ico", dangerous_inner_html: "{icon_html(\"branch\", 14)}" }
                span { class: "bt-count mono", "{count}" }
                span { class: "bt-label", "BRANCHES" }
            }
            div { class: "bt-list",
                if branches.is_empty() {
                    div { class: "empty-state", p { "No branches" } }
                }
                for b in branches.iter() {
                    {
                        let is_default = b.name == default_branch;
                        let when = from_now(&b.when);
                        let bname = b.name.clone();
                        let bpath = repo_path.clone();
                        let is_current = b.current;
                        let ahead = b.ahead;
                        let behind = b.behind;
                        let last_msg = b.last_msg.clone();
                        rsx! {
                            div { class: "bt-row",
                                // Icon box
                                div { class: "bt-icon-box",
                                    span { dangerous_inner_html: "{icon_html(\"branch\", 16)}" }
                                }
                                // Info
                                div { class: "bt-info",
                                    div { class: "bt-name-row",
                                        span { class: "bt-name mono", "{bname}" }
                                        if is_current {
                                            span { class: "bt-badge bt-badge-current", "current" }
                                        }
                                        if is_default {
                                            span { class: "bt-badge bt-badge-default", "default" }
                                        }
                                    }
                                    if !last_msg.is_empty() {
                                        div { class: "bt-sub",
                                            span { "{last_msg}" }
                                            if !when.is_empty() {
                                                span { class: "bt-dot", "·" }
                                                span { "{when}" }
                                            }
                                        }
                                    }
                                }
                                // Right: ahead/behind + checkout
                                div { class: "bt-right",
                                    if ahead > 0 || behind > 0 {
                                        div { class: "bt-sync",
                                            if ahead > 0 {
                                                span { class: "bt-ahead", "↑{ahead}" }
                                            }
                                            if behind > 0 {
                                                span { class: "bt-behind", "↓{behind}" }
                                            }
                                        }
                                    }
                                    if !is_current {
                                        button {
                                            class: "bt-checkout-btn",
                                            onclick: move |_| {
                                                let p = bpath.clone();
                                                let br = bname.clone();
                                                spawn(async move {
                                                    if let Some(r) = invoke::checkout_branch(p, br).await {
                                                        let mut t = toasts.write();
                                                        t.push(Toast::new(
                                                            if r.success { "Checked out" } else { "Checkout failed" },
                                                            &r.message,
                                                        ));
                                                    }
                                                });
                                            },
                                            "Checkout"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Contributors tab ──────────────────────────────────────────────────────────

fn fmt_stat(n: u32) -> String {
    if n >= 1000 { format!("{:.1}k", n as f32 / 1000.0) } else { n.to_string() }
}

#[component]
fn ContribsTab(contributors: Vec<Contributor>) -> Element {
    let count = contributors.len();
    rsx! {
        div { class: "contribs-tab",
            div { class: "contribs-tab-hdr",
                span { class: "cv-hdr-ico", dangerous_inner_html: "{icon_html(\"users\", 14)}" }
                span { class: "cv-count mono", "{count}" }
                span { class: "cv-label", "CONTRIBUTORS" }
            }
            div { class: "cv-list",
                if contributors.is_empty() {
                    div { class: "empty-state", p { "No contributors" } }
                }
                for c in contributors.iter() {
                    {
                        let colors = ["#6366F1","#EC4899","#14B8A6","#F59E0B","#8B5CF6","#06B6D4","#F43F5E","#22C55E"];
                        let av_color = colors[c.handle.bytes().fold(0usize, |a, b| a.wrapping_add(b as usize)) % colors.len()];
                        let initials = c.name.split_whitespace()
                            .filter_map(|w| w.chars().next())
                            .take(2)
                            .collect::<String>()
                            .to_uppercase();
                        let when = from_now(&c.last_active);
                        let adds_fmt = fmt_stat(c.additions);
                        let dels_fmt = fmt_stat(c.deletions);
                        let total = (c.additions + c.deletions).max(1);
                        let add_pct = (c.additions as f32 / total as f32 * 100.0) as u32;
                        let del_pct = 100u32.saturating_sub(add_pct);
                        rsx! {
                            div { class: "cv-row",
                                div { class: "cv-avatar", style: "background:{av_color}", "{initials}" }
                                div { class: "cv-info",
                                    div { class: "cv-name-row",
                                        span { class: "cv-name", "{c.name}" }
                                        span { class: "cv-handle mono", "@{c.handle}" }
                                    }
                                    div { class: "cv-meta",
                                        span { "{c.commits} commits" }
                                        span { class: "cv-dot", "·" }
                                        span { "active" }
                                        if !when.is_empty() {
                                            span { class: "cv-dot", "·" }
                                            span { "{when}" }
                                        }
                                    }
                                }
                                div { class: "cv-right",
                                    div { class: "cv-stats",
                                        span { class: "adds", "+{adds_fmt}" }
                                        span { class: "dels", "-{dels_fmt}" }
                                    }
                                    div { class: "cv-bar",
                                        div { class: "cv-bar-add", style: "width:{add_pct}%" }
                                        div { class: "cv-bar-del", style: "width:{del_pct}%" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── README tab ────────────────────────────────────────────────────────────────

fn md_to_html(src: &str) -> String {
    use pulldown_cmark::{html, Event, Options, Parser};
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    // Strip raw HTML blocks and inline HTML to prevent XSS from untrusted README files.
    let parser = Parser::new_ext(src, opts)
        .filter(|e| !matches!(e, Event::Html(_) | Event::InlineHtml(_)));
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

#[component]
fn ReadmeTab(readme: String) -> Element {
    if readme.is_empty() {
        return rsx! {
            div { class: "empty-state",
                span { dangerous_inner_html: "{icon_html(\"book\", 32)}" }
                p { "No README found" }
            }
        };
    }
    let html = md_to_html(&readme);
    rsx! {
        div { class: "readme-wrap",
            div { class: "ov-card",
                div { class: "ov-card-hdr",
                    span { class: "ov-card-ico", dangerous_inner_html: "{icon_html(\"book\", 13)}" }
                    "README.MD"
                }
                div { class: "md readme-content", dangerous_inner_html: "{html}" }
            }
        }
    }
}

