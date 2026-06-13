use dioxus::prelude::*;

use crate::{
    icons::icon_html,
    invoke,
    models::*,
    shared::{Avatar, AvatarStack, StatusPill, Toast},
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
                for (id, label) in [("overview","Overview"),("commits","Commits"),("branches","Branches"),("contribs","Contributors"),("readme","README")] {
                    button {
                        class: if t == id { "tab active" } else { "tab" },
                        onclick: move |_| tab.set(id),
                        "{label}"
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
                        "commits" => rsx! { CommitsTab { commits: d.commits.clone() } },
                        "branches" => rsx! { BranchesTab { branches: d.branches.clone() } },
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
fn CommitsTab(commits: Vec<Commit>) -> Element {
    rsx! {
        div { class: "commits-list",
            for c in commits.iter() {
                CommitRow { commit: c.clone() }
            }
            if commits.is_empty() {
                div { class: "empty-state", p { "No commits" } }
            }
        }
    }
}

#[component]
fn CommitRow(commit: Commit) -> Element {
    let hash_short = commit.hash.chars().take(7).collect::<String>();
    let when = from_now(&commit.date);
    rsx! {
        div { class: "commit",
            div { class: "commit-left",
                span { class: "commit-hash mono", "{hash_short}" }
                div { class: "commit-info",
                    span { class: "commit-msg", "{commit.msg}" }
                    span { class: "commit-meta",
                        "{commit.author}"
                        if !when.is_empty() { " · {when}" }
                    }
                }
            }
            div { class: "commit-stats",
                if commit.additions > 0 {
                    span { class: "adds", "+{commit.additions}" }
                }
                if commit.deletions > 0 {
                    span { class: "dels", "-{commit.deletions}" }
                }
            }
        }
    }
}

// ── Branches tab ─────────────────────────────────────────────────────────────

#[component]
fn BranchesTab(branches: Vec<Branch>) -> Element {
    rsx! {
        div { class: "branches-list",
            for b in branches.iter() {
                div { class: if b.current { "branch-row current" } else { "branch-row" },
                    span { dangerous_inner_html: "{icon_html(\"branch\", 14)}" }
                    span { class: "branch-name mono", "{b.name}" }
                    if b.current {
                        span { class: "badge", "current" }
                    }
                    div { class: "branch-right",
                        if b.ahead > 0 { span { class: "adds", "↑{b.ahead}" } }
                        if b.behind > 0 { span { class: "dels", "↓{b.behind}" } }
                        span { class: "branch-msg", "{b.last_msg}" }
                        span { class: "branch-when", "{from_now(&b.when)}" }
                    }
                }
            }
            if branches.is_empty() {
                div { class: "empty-state", p { "No branches" } }
            }
        }
    }
}

// ── Contributors tab ──────────────────────────────────────────────────────────

#[component]
fn ContribsTab(contributors: Vec<Contributor>) -> Element {
    rsx! {
        div { class: "contribs-list",
            for c in contributors.iter() {
                {
                    let initials = {
                        c.name.split_whitespace()
                            .filter_map(|w| w.chars().next())
                            .take(2)
                            .collect::<String>()
                            .to_uppercase()
                    };
                    let color = {
                        let colors = ["#6366F1","#EC4899","#14B8A6","#F59E0B","#8B5CF6","#06B6D4","#F43F5E","#22C55E"];
                        let idx = c.handle.bytes().fold(0usize, |a, b| a.wrapping_add(b as usize)) % colors.len();
                        colors[idx].to_string()
                    };
                    let when = from_now(&c.last_active);
                    rsx! {
                        div { class: "contrib-row",
                            Avatar { initials, color, name: c.name.clone(), size: 36 }
                            div { class: "contrib-info",
                                span { class: "contrib-name", "{c.name}" }
                                span { class: "contrib-handle mono", "@{c.handle}" }
                            }
                            div { class: "contrib-stats",
                                span { class: "stat-chip", span { dangerous_inner_html: "{icon_html(\"commit\", 12)}" } "{c.commits}" }
                                span { class: "adds", "+{c.additions}" }
                                span { class: "dels", "-{c.deletions}" }
                                if !when.is_empty() { span { class: "contrib-when", "{when}" } }
                            }
                        }
                    }
                }
            }
            if contributors.is_empty() {
                div { class: "empty-state", p { "No contributors" } }
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

