use dioxus::prelude::*;

use crate::{
    icons::icon_html,
    models::*,
    shared::StatusPill,
};

#[component]
pub fn RepoList(
    repos: Vec<RepoSummary>,
    folders: Vec<Folder>,
    sel: Signal<Selection>,
    on_open: EventHandler<RepoSummary>,
    search: Signal<String>,
) -> Element {
    let mut view = use_signal(|| "list"); // "list" | "grid"
    let mut sort = use_signal(|| "name"); // "name" | "recent" | "status"

    let q = search.read().to_lowercase();
    let v = *view.read();
    let s = *sort.read();

    let mut filtered: Vec<RepoSummary> = repos
        .iter()
        .filter(|r| q.is_empty() || r.name.to_lowercase().contains(&q) || r.path.to_lowercase().contains(&q))
        .cloned()
        .collect();

    filtered.sort_by(|a, b| match s {
        "recent" => b.last_commit_at.cmp(&a.last_commit_at),
        "status" => {
            let sa = if !a.status.clean { 0 } else { 1 };
            let sb = if !b.status.clean { 0 } else { 1 };
            sa.cmp(&sb).then(a.name.cmp(&b.name))
        }
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    let repo_count = filtered.len();
    let plural = if repo_count == 1 { "repo" } else { "repos" };

    rsx! {
        div { class: "list",
            // Count
            div { class: "dash-h ", "All Repositories" }
            div { class: "list-sub", "{repo_count} {plural}" }

            // Toolbar
            div { class: "list-bar",
                // search moved to appbar
                div { class: "bar-actions",
                    select {
                        class: "sort-sel",
                        onchange: move |e| sort.set(Box::leak(e.value().into_boxed_str())),
                        option { value: "name", "Name" }
                        option { value: "recent", "Recent" }
                        option { value: "status", "Status" }
                    }
                    button {
                        class: if v == "list" { "tb-btn active" } else { "tb-btn" },
                        onclick: move |_| view.set("list"),
                        span { dangerous_inner_html: "{icon_html(\"list\", 15)}" }
                    }
                    button {
                        class: if v == "grid" { "tb-btn active" } else { "tb-btn" },
                        onclick: move |_| view.set("grid"),
                        span { dangerous_inner_html: "{icon_html(\"grid\", 15)}" }
                    }
                }
            }

            

            // Items
            if v == "grid" {
                div { class: "repo-grid",
                    for r in filtered.iter() {
                        {
                            let r2 = r.clone();
                            let folder = folders.iter().find(|f| f.id == r.folder_id).cloned();
                            let color = folder.as_ref().map(|f| f.color.clone()).unwrap_or_else(|| "#7C6BFF".into());
                            let is_sel = sel.read().kind == NavKind::Repo && sel.read().id == r.id;
                            rsx! { RepoGridCard { repo: r2, color, selected: is_sel, on_open } }
                        }
                    }
                }
            } else {
                div { class: "repo-list-items",
                    for r in filtered.iter() {
                        {
                            let r2 = r.clone();
                            let folder = folders.iter().find(|f| f.id == r.folder_id).cloned();
                            let color = folder.as_ref().map(|f| f.color.clone()).unwrap_or_else(|| "#7C6BFF".into());
                            let folder_name = folder.map(|f| f.name.clone()).unwrap_or_default();
                            let is_sel = sel.read().kind == NavKind::Repo && sel.read().id == r.id;
                            rsx! { RepoListRow { repo: r2, color, folder_name, selected: is_sel, on_open } }
                        }
                    }
                }
            }

            if filtered.is_empty() {
                div { class: "empty-state",
                    span { dangerous_inner_html: "{icon_html(\"archive\", 36)}" }
                    p { "No repos match" }
                }
            }
        }
    }
}

// ── List row ──────────────────────────────────────────────────────────────────

#[component]
fn RepoListRow(
    repo: RepoSummary,
    color: String,
    folder_name: String,
    selected: bool,
    on_open: EventHandler<RepoSummary>,
) -> Element {
    let glyph = glyph_for(&repo.name);
    let when = from_now(&repo.last_commit_at);
    let repo2 = repo.clone();
    let repo3 = repo.clone();
    let dirty = !repo.status.clean;

    rsx! {
        div {
            class: if selected { "repo sel" } else { "repo" },
            onclick: move |_| on_open.call(repo2.clone()),
            // Glyph
            div {
                class: "repo-glyph",
                style: "background:{color}",
                if dirty { span { class: "dirty-dot" } }
                "{glyph}"
            }
            // Middle: name + meta
            div { class: "repo-mid",
                div { class: "repo-name",
                    span { class: "nm", "{repo.name}" }
                }
                div { class: "repo-meta",
                    span { class: "repo-branch mono",
                        span { dangerous_inner_html: "{icon_html(\"branch\", 11)}" }
                        "{repo.branch}"
                    }
                    if !folder_name.is_empty() {
                        span { class: "sep", "·" }
                        span { class: "repo-folder", "{folder_name}" }
                    }
                }
            }
            // Right: time + status pill
            div { class: "repo-right",
                if !when.is_empty() {
                    span { class: "repo-time", "{when}" }
                }
                StatusPill { repo: repo3 }
            }
        }
    }
}

// ── Grid card ─────────────────────────────────────────────────────────────────

#[component]
fn RepoGridCard(
    repo: RepoSummary,
    color: String,
    selected: bool,
    on_open: EventHandler<RepoSummary>,
) -> Element {
    let glyph = glyph_for(&repo.name);
    let when = from_now(&repo.last_commit_at);
    let repo2 = repo.clone();
    let repo3 = repo.clone();
    let dirty = !repo.status.clean;

    rsx! {
        div {
            class: if selected { "repo-card selected" } else { "repo-card" },
            onclick: move |_| on_open.call(repo2.clone()),
            div { class: "card-head",
                div {
                    class: "repo-glyph lg",
                    style: "background:{color}",
                    if dirty {
                        span { class: "dirty-dot" }
                    }
                    "{glyph}"
                }
                StatusPill { repo: repo3 }
            }
            div { class: "card-name", "{repo.name}" }
            if !repo.description.is_empty() {
                div { class: "card-desc", "{repo.description}" }
            }
            div { class: "card-foot",
                span { class: "mono", style: "font-size:11px;color:var(--text-3)",
                    span { dangerous_inner_html: "{icon_html(\"branch\", 11)}" }
                    "{repo.branch}"
                }
                span { class: "card-when", "{when}" }
            }
            if !repo.languages.is_empty() {
                div { class: "lang-bar",
                    for lang in repo.languages.iter().take(4) {
                        span { style: "width:{lang.pct}%;background:{lang.color}" }
                    }
                }
            }
        }
    }
}
