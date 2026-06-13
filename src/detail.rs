use dioxus::prelude::*;

use crate::{
    icons::icon_html,
    invoke,
    models::*,
    shared::{Avatar, AvatarStack, Icon, StatusPill, Toast},
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
    let path2 = path.clone();

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
                on_fetch: {
                    let path = path2.clone();
                    move |_| {
                        let p = path.clone();
                        spawn(async move {
                            if let Some(r) = invoke::fetch_repo(p.clone()).await {
                                let mut t = toasts.write();
                                t.push(Toast::new(
                                    if r.success { "Fetched" } else { "Fetch failed" },
                                    &r.message,
                                ));
                            }
                        });
                    }
                },
                on_pull: {
                    let p = path2.clone();
                    move |_| {
                        let p = p.clone();
                        spawn(async move {
                            if let Some(r) = invoke::pull_repo(p).await {
                                let mut t = toasts.write();
                                t.push(Toast::new(
                                    if r.success { "Pulled" } else { "Pull failed" },
                                    &r.message,
                                ));
                            }
                        });
                    }
                },
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
                        _ => rsx! { OverviewTab { detail: d, repo: repo.clone() } },
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
    on_fetch: EventHandler,
    on_pull: EventHandler,
) -> Element {
    let glyph = glyph_for(&repo.name);
    let repo2 = repo.clone();
    let repo3 = repo.clone();

    rsx! {
        div { class: "detail-head",
            div { class: "dh-left",
                div { class: "repo-glyph lg", style: "background:#7C6BFF",
                    "{glyph}"
                }
                div { class: "dh-info",
                    div { class: "dh-name", "{repo.name}" }
                    div { class: "dh-sub",
                        if !repo.remote.is_empty() {
                            a {
                                class: "dh-remote",
                                href: "#",
                                onclick: |e| e.prevent_default(),
                                span { dangerous_inner_html: "{icon_html(\"external\", 11)}" }
                                "{repo.remote}"
                            }
                        }
                    }
                }
            }
            div { class: "dh-actions",
                StatusPill { repo: repo2 }
                // Open in editors
                for ed in editors.iter() {
                    {
                        let path = repo.path.clone();
                        let eid = ed.id.clone();
                        let ename = ed.name.clone();
                        let accent = ed.accent.clone();
                        let mark = ed.mark.clone();
                        rsx! {
                            button {
                                class: "open-btn",
                                style: "background:{accent}",
                                title: "Open in {ename}",
                                onclick: move |_| {
                                    let p = path.clone();
                                    let e = eid.clone();
                                    spawn(async move { invoke::open_in(e, p).await; });
                                },
                                "{mark}"
                            }
                        }
                    }
                }
                button {
                    class: "tb-btn",
                    title: "Fetch",
                    onclick: move |_| on_fetch.call(()),
                    span { dangerous_inner_html: "{icon_html(\"refresh\", 16)}" }
                }
                button {
                    class: "btn-primary",
                    title: "Pull",
                    onclick: move |_| on_pull.call(()),
                    span { dangerous_inner_html: "{icon_html(\"arrowDown\", 14)}" }
                    "Pull"
                }
            }
        }
    }
}

// ── Overview tab ──────────────────────────────────────────────────────────────

#[component]
fn OverviewTab(detail: RepoDetail, repo: RepoSummary) -> Element {
    let s = &repo.status;
    rsx! {
        div { class: "overview",
            // Stat cards row
            div { class: "stat-row",
                StatCard { label: "Branch", value: repo.branch.clone(), icon: "branch" }
                StatCard { label: "Staged", value: s.staged.to_string(), icon: "diff" }
                StatCard { label: "Modified", value: s.modified.to_string(), icon: "diff" }
                StatCard { label: "Untracked", value: s.untracked.to_string(), icon: "diff" }
                if s.ahead > 0 || s.behind > 0 {
                    StatCard { label: "Ahead", value: s.ahead.to_string(), icon: "arrowUp" }
                    StatCard { label: "Behind", value: s.behind.to_string(), icon: "arrowDown" }
                }
            }

            // Language bar
            if !detail.languages.is_empty() {
                div { class: "section",
                    div { class: "sec-head", "Languages" }
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

            // Changed files
            if !s.files.is_empty() {
                div { class: "section",
                    div { class: "sec-head", "Changed Files" }
                    div { class: "file-list",
                        for f in s.files.iter().take(20) {
                            div { class: "file-row",
                                span { class: "file-kind {f.kind}", "{f.kind.chars().next().unwrap_or('?').to_uppercase()}" }
                                span { class: "mono file-path", "{f.path}" }
                            }
                        }
                    }
                }
            }

            // Recent commits preview
            if !detail.commits.is_empty() {
                div { class: "section",
                    div { class: "sec-head", "Recent Commits" }
                    for c in detail.commits.iter().take(5) {
                        CommitRow { commit: c.clone() }
                    }
                }
            }

            // Contributors preview
            if !detail.contributors.is_empty() {
                div { class: "section",
                    div { class: "sec-head", "Contributors" }
                    AvatarStack {
                        contributors: detail.contributors.clone(),
                        max: 6,
                        size: 32,
                    }
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
    rsx! {
        div { class: "readme-body prose",
            pre { class: "readme-raw", "{readme}" }
        }
    }
}

// ── Stat card ─────────────────────────────────────────────────────────────────

#[component]
fn StatCard(label: &'static str, value: String, icon: &'static str) -> Element {
    rsx! {
        div { class: "stat-card",
            span { class: "stat-ico", dangerous_inner_html: "{icon_html(icon, 15)}" }
            div { class: "stat-body",
                span { class: "stat-val", "{value}" }
                span { class: "stat-lbl", "{label}" }
            }
        }
    }
}
