use dioxus::prelude::*;

use crate::{
    icons::icon_html,
    invoke,
    models::*,
    shared::Toast,
};

#[component]
pub fn Dashboard(
    repos: Vec<RepoSummary>,
    folders: Vec<Folder>,
    toasts: Signal<Vec<Toast>>,
    on_open: EventHandler<RepoSummary>,
) -> Element {
    let total = repos.len();
    let dirty: Vec<_> = repos.iter().filter(|r| !r.status.clean).cloned().collect();
    let stale: Vec<_> = repos.iter().filter(|r| is_stale(&r.last_commit_at)).cloned().collect();
    let favorites: Vec<_> = repos.iter().filter(|r| r.favorite).cloned().collect();

    let recent: Vec<RepoSummary> = {
        let mut v = repos.clone();
        v.sort_by(|a, b| b.last_commit_at.cmp(&a.last_commit_at));
        v.into_iter().take(8).collect()
    };

    rsx! {
        div { class: "dash scroll",
            // Stat cards
            div { class: "dash-cards",
                DashCard { label: "Total Repos", value: total.to_string(), icon: "archive", color: "#7C6BFF" }
                DashCard { label: "Uncommitted", value: dirty.len().to_string(), icon: "diff", color: "#F59E0B" }
                DashCard { label: "Stale (30d+)", value: stale.len().to_string(), icon: "clock", color: "#EF4444" }
                DashCard { label: "Favorites", value: favorites.len().to_string(), icon: "star", color: "#22C55E" }
            }

            div { class: "dash-cols",
                // Attention column
                div { class: "dash-col",
                    if !dirty.is_empty() {
                        div { class: "dash-sec",
                            div { class: "dash-sec-h",
                                span { dangerous_inner_html: "{icon_html(\"diff\", 14)}" }
                                "Needs Commit"
                                span { class: "badge", "{dirty.len()}" }
                            }
                            for r in dirty.iter().take(6) {
                                {
                                    let r2 = r.clone();
                                    let folder = folders.iter().find(|f| f.id == r.folder_id);
                                    let color = folder.map(|f| f.color.clone()).unwrap_or_else(|| "#7C6BFF".into());
                                    let n = r.status.staged + r.status.modified + r.status.untracked;
                                    rsx! {
                                        AttentionRow {
                                            repo: r2,
                                            color,
                                            detail: format!("{n} change{}", if n == 1 { "" } else { "s" }),
                                            on_open,
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !stale.is_empty() {
                        div { class: "dash-sec",
                            div { class: "dash-sec-h",
                                span { dangerous_inner_html: "{icon_html(\"clock\", 14)}" }
                                "Stale"
                                span { class: "badge danger", "{stale.len()}" }
                            }
                            for r in stale.iter().take(6) {
                                {
                                    let r2 = r.clone();
                                    let folder = folders.iter().find(|f| f.id == r.folder_id);
                                    let color = folder.map(|f| f.color.clone()).unwrap_or_else(|| "#7C6BFF".into());
                                    let when = from_now(&r.last_commit_at);
                                    rsx! {
                                        AttentionRow {
                                            repo: r2,
                                            color,
                                            detail: format!("last: {when}"),
                                            on_open,
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if dirty.is_empty() && stale.is_empty() {
                        div { class: "dash-sec",
                            div { class: "dash-ok",
                                span { dangerous_inner_html: "{icon_html(\"check\", 22)}" }
                                "All repos clean"
                            }
                        }
                    }
                }

                // Recent activity column
                div { class: "dash-col",
                    div { class: "dash-sec",
                        div { class: "dash-sec-h",
                            span { dangerous_inner_html: "{icon_html(\"activity\", 14)}" }
                            "Recent Activity"
                        }
                        for r in recent.iter() {
                            {
                                let r2 = r.clone();
                                let folder = folders.iter().find(|f| f.id == r.folder_id);
                                let color = folder.map(|f| f.color.clone()).unwrap_or_else(|| "#7C6BFF".into());
                                let when = from_now(&r.last_commit_at);
                                let glyph = glyph_for(&r.name);
                                rsx! {
                                    div {
                                        class: "activity-row",
                                        onclick: move |_| on_open.call(r2.clone()),
                                        div { class: "repo-glyph sm", style: "background:{color}", "{glyph}" }
                                        div { class: "act-info",
                                            span { class: "act-name", "{r.name}" }
                                            span { class: "act-branch mono",
                                                span { dangerous_inner_html: "{icon_html(\"branch\", 10)}" }
                                                "{r.branch}"
                                            }
                                        }
                                        span { class: "act-when", "{when}" }
                                    }
                                }
                            }
                        }
                        if recent.is_empty() {
                            div { class: "empty-state", p { "No repos yet" } }
                        }
                    }
                }
            }
        }
    }
}

// ── Stat card ─────────────────────────────────────────────────────────────────

#[component]
fn DashCard(label: &'static str, value: String, icon: &'static str, color: &'static str) -> Element {
    rsx! {
        div { class: "dash-card",
            div { class: "dc-ico", style: "color:{color}", dangerous_inner_html: "{icon_html(icon, 22)}" }
            div { class: "dc-body",
                span { class: "dc-val", "{value}" }
                span { class: "dc-lbl", "{label}" }
            }
        }
    }
}

// ── Attention row ─────────────────────────────────────────────────────────────

#[component]
fn AttentionRow(
    repo: RepoSummary,
    color: String,
    detail: String,
    on_open: EventHandler<RepoSummary>,
) -> Element {
    let glyph = glyph_for(&repo.name);
    let repo2 = repo.clone();
    rsx! {
        div {
            class: "attention-row",
            onclick: move |_| on_open.call(repo2.clone()),
            div { class: "repo-glyph sm", style: "background:{color}", "{glyph}" }
            span { class: "att-name", "{repo.name}" }
            span { class: "att-detail", "{detail}" }
        }
    }
}

// ── Settings modal ────────────────────────────────────────────────────────────

#[component]
pub fn SettingsModal(
    editors: Vec<Editor>,
    on_close: EventHandler,
    toasts: Signal<Vec<Toast>>,
) -> Element {
    let mut new_name = use_signal(|| String::new());
    let mut new_cmd = use_signal(|| String::new());

    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_close.call(()),
            div { class: "modal", onclick: |e| e.stop_propagation(),
                div { class: "modal-head",
                    "Editor / IDE Settings"
                    button { class: "tb-btn", onclick: move |_| on_close.call(()),
                        span { dangerous_inner_html: "{icon_html(\"x\", 16)}" }
                    }
                }
                div { class: "modal-body",
                    div { class: "eds-list",
                        for ed in editors.iter() {
                            div { class: "ed-row",
                                span {
                                    class: "ed-mark",
                                    style: "background:{ed.accent}",
                                    "{ed.mark}"
                                }
                                span { class: "ed-name", "{ed.name}" }
                                span { class: "mono ed-cmd", "{ed.cmd}" }
                                if ed.detected {
                                    span { class: "badge", "auto" }
                                }
                            }
                        }
                    }
                    div { class: "ed-add",
                        div { class: "dash-sec-h", "Add Custom Editor" }
                        input {
                            class: "inp",
                            placeholder: "Name (e.g. Helix)",
                            value: "{new_name}",
                            oninput: move |e| new_name.set(e.value().clone()),
                        }
                        input {
                            class: "inp mono",
                            placeholder: "Command (e.g. hx)",
                            value: "{new_cmd}",
                            oninput: move |e| new_cmd.set(e.value().clone()),
                        }
                        button {
                            class: "btn-primary",
                            onclick: move |_| {
                                let name = new_name.read().trim().to_string();
                                let cmd = new_cmd.read().trim().to_string();
                                if name.is_empty() || cmd.is_empty() { return; }
                                let id = format!("custom_{}", name.to_lowercase().replace(' ', "_"));
                                let mark = {
                                    let up = name.to_uppercase();
                                    up.chars().take(2).collect::<String>()
                                };
                                spawn(async move {
                                    invoke::add_custom_editor(id, name.clone(), String::new(), cmd.clone()).await;
                                    let mut t = toasts.write();
                                    t.push(Toast::new("Editor added", &name));
                                });
                                new_name.set(String::new());
                                new_cmd.set(String::new());
                            },
                            "Add"
                        }
                    }
                }
            }
        }
    }
}
