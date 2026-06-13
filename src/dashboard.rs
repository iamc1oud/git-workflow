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
        div { 
            class: "dash scroll",
            // Header
            div { 
                class: "dash-header",
                h1 { class: "dash-h", "Hey Buddy 👋" }
                p { 
                    class: "dash-sub", 
                    "{total} repositories across 5 folders • {dirty.len()} need attention" }
            }

            // Stat cards
            div { class: "stat-grid",
                StatCard { label: "Total Repos", value: total.to_string(), icon: "archive", color: "#7C6BFF" }
                StatCard { label: "Uncommitted", value: dirty.len().to_string(), icon: "diff", color: "#F59E0B" }
                StatCard { label: "Stale (30d+)", value: stale.len().to_string(), icon: "clock", color: "#EF4444" }
                StatCard { label: "Favorites", value: favorites.len().to_string(), icon: "star", color: "#22C55E" }
            }

            // Dashboard columns
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
                            div { class: "empty-state dash-empty",
                                span { dangerous_inner_html: "{icon_html(\"check\", 24)}" }
                                p { "All repos clean" }
                                p { class: "muted", "Keep up the great work!" }
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
                            span { class: "badge", "{recent.len()}" }
                        }
                        if !recent.is_empty() {
                            for r in recent.iter() {
                                {
                                    let r2 = r.clone();
                                    let folder = folders.iter().find(|f| f.id == r.folder_id);
                                    let color = folder.map(|f| f.color.clone()).unwrap_or_else(|| "#7C6BFF".into());
                                    let when = from_now(&r.last_commit_at);
                                    let glyph = glyph_for(&r.name);
                                    rsx! {
                                        ActivityRow {
                                            repo: r2,
                                            color,
                                            glyph,
                                            when,
                                            on_open,
                                        }
                                    }
                                }
                            }
                        } else {
                            div { class: "empty-state dash-empty",
                                span { dangerous_inner_html: "{icon_html(\"inbox\", 24)}" }
                                p { "No activity" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Stat card ─────────────────────────────────────────────────────────────────

#[component]
fn StatCard(label: &'static str, value: String, icon: &'static str, color: &'static str) -> Element {
    rsx! {
        div { class: "stat",
            div { class: "top",
                span { class: "stat ic", style: "background:{color}", dangerous_inner_html: "{icon_html(icon, 16)}" }
            }
            div { class: "num", "{value}" }
            span { class: "lbl", "{label}" }
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
            class: "attn",
            onclick: move |_| on_open.call(repo2.clone()),
            div { class: "repo-glyph", style: "width:32px;height:32px;border-radius:8px;font-size:12px;background:{color}", "{glyph}" }
            div { style: "flex:1;min-width:0",
                div { style: "font-size:13px;font-weight:600;white-space:nowrap;overflow:hidden;text-overflow:ellipsis", "{repo.name}" }
                div { style: "font-size:11px;color:var(--text-3);margin-top:2px", "{detail}" }
            }
            span { style: "font-size:11px;color:var(--text-3);white-space:nowrap", "{from_now(&repo.last_commit_at)}" }
        }
    }
}

// ── Activity row ──────────────────────────────────────────────────────────────

#[component]
fn ActivityRow(
    repo: RepoSummary,
    color: String,
    glyph: String,
    when: String,
    on_open: EventHandler<RepoSummary>,
) -> Element {
    let repo2 = repo.clone();
    rsx! {
        div {
            class: "act",
            onclick: move |_| on_open.call(repo2.clone()),
            div { class: "repo-glyph", style: "width:32px;height:32px;border-radius:8px;font-size:12px;background:{color}", "{glyph}" }
            div { style: "flex:1;min-width:0",
                div { style: "font-size:13px;font-weight:600;white-space:nowrap;overflow:hidden;text-overflow:ellipsis", "{repo.name}" }
                div { class: "act-branch", style: "font-size:10px;margin-top:2px",
                    span { dangerous_inner_html: "{icon_html(\"branch\", 10)}" }
                    "{repo.branch}"
                }
            }
            span { class: "act-when", "{when}" }
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
    let mut new_path = use_signal(|| String::new());

    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_close.call(()),
            div { class: "modal sm-modal", onclick: |e| e.stop_propagation(),

                // ── Header ────────────────────────────────────────────────────
                div { class: "modal-h",
                    div { class: "sm-hdr-icon",
                        span { dangerous_inner_html: "{icon_html(\"code\", 18)}" }
                    }
                    span { class: "modal-h-title", "Editors & Tools" }
                    button { class: "sm-close", onclick: move |_| on_close.call(()),
                        span { dangerous_inner_html: "{icon_html(\"x\", 16)}" }
                    }
                }

                // ── Body ──────────────────────────────────────────────────────
                div { class: "modal-b",

                    // Subtitle
                    div { class: "sm-subtitle",
                        span { class: "sm-subtitle-ico", dangerous_inner_html: "{icon_html(\"sparkle\", 13)}" }
                        span {
                            "Auto-detected from "
                            span { class: "mono sm-path-chip", "/Applications" }
                            ". Toggle off to hide, or add any editor by its path."
                        }
                    }

                    // Editor list
                    for ed in editors.iter() {
                        div { class: "ide-row",
                            // Icon box
                            div { class: "sm-ed-icon", style: "background:{ed.accent}",
                                "{ed.mark}"
                            }
                            // Info
                            div { style: "flex:1;min-width:0",
                                div { class: "ide-row nm", "{ed.name}" }
                                div { class: "ide-row pa",
                                    if ed.path.is_empty() || !ed.detected {
                                        "Not installed — add manually"
                                    } else {
                                        "{ed.path}"
                                    }
                                }
                            }
                            // Toggle
                            div { class: "ide-row rt",
                                if ed.detected {
                                    button { class: "sm-toggle sm-toggle-on",
                                        span { "✓" }
                                        "Enabled"
                                    }
                                } else {
                                    button { class: "sm-toggle sm-toggle-off",
                                        "Enable"
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Footer: add editor ────────────────────────────────────────
                div { class: "modal-foot",
                    div { class: "sm-add-label", "ADD EDITOR BY PATH" }
                    div { class: "sm-add-row",
                        input {
                            class: "field sm-name-inp",
                            placeholder: "Name",
                            value: "{new_name}",
                            oninput: move |e| new_name.set(e.value().clone()),
                        }
                        input {
                            class: "field sm-path-inp mono",
                            placeholder: "/Applications/MyEditor.app",
                            value: "{new_path}",
                            oninput: move |e| new_path.set(e.value().clone()),
                        }
                        button {
                            class: "btn-primary sm-add-btn",
                            onclick: move |_| {
                                let name = new_name.read().trim().to_string();
                                let path = new_path.read().trim().to_string();
                                if name.is_empty() || path.is_empty() { return; }
                                let id = format!("custom_{}", name.to_lowercase().replace(' ', "_"));
                                let cmd = std::path::Path::new(&path)
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or(&name)
                                    .to_lowercase()
                                    .replace(' ', "-");
                                spawn(async move {
                                    invoke::add_custom_editor(id, name.clone(), path, cmd).await;
                                    let mut t = toasts.write();
                                    t.push(Toast::new("Editor added", &name));
                                });
                                new_name.set(String::new());
                                new_path.set(String::new());
                            },
                            span { dangerous_inner_html: "{icon_html(\"plus\", 14)}" }
                            "Add"
                        }
                    }
                }
            }
        }
    }
}
