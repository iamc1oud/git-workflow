#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::{
    dashboard::{Dashboard, SettingsModal},
    icons::icon_html,
    invoke,
    models::*,
    palette::CommandPalette,
    repolist::RepoList,
    shared::Toast,
    sidebar::Sidebar,
};

static CSS: Asset = asset!("/assets/styles.css");

pub fn App() -> Element {
    // ── Global state ──────────────────────────────────────────────────────────
    let mut folders: Signal<Vec<Folder>> = use_signal(Vec::new);
    let mut repos: Signal<Vec<RepoSummary>> = use_signal(Vec::new);
    let mut editors: Signal<Vec<Editor>> = use_signal(Vec::new);
    let mut sel: Signal<Selection> = use_signal(|| Selection::smart("dashboard"));
    let mut theme: Signal<String> = use_signal(|| "dark".to_string());
    let mut show_palette = use_signal(|| false);
    let mut show_settings = use_signal(|| false);
    let mut toasts: Signal<Vec<Toast>> = use_signal(Vec::new);

    // ── Bootstrap ─────────────────────────────────────────────────────────────
    use_effect(move || {
        spawn(async move {
            folders.set(invoke::list_folders().await);
            repos.set(invoke::list_all_repos().await);
            editors.set(invoke::detect_ides().await);
        });
    });

    // ── Theme effect ──────────────────────────────────────────────────────────
    use_effect(move || {
        let t = theme.read().clone();
        let _ = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
            .map(|el| el.set_attribute("data-theme", &t));
    });

    // ── Global Cmd+K / Ctrl+K shortcut ───────────────────────────────────────
    use_effect(move || {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;
        let closure = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
            if (e.meta_key() || e.ctrl_key()) && e.key() == "k" {
                e.prevent_default();
                let cur = *show_palette.peek();
                show_palette.set(!cur);
            }
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);
        if let Some(win) = web_sys::window() {
            let _ = win.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
        }
        closure.forget();
    });

    // ── Refresh callback ─────────────────────────────────────────────────────
    let refresh = move |_| {
        spawn(async move {
            folders.set(invoke::list_folders().await);
            repos.set(invoke::list_all_repos().await);
        });
    };

    // ── Derived: selected repo (when NavKind::Repo) ───────────────────────────
    let selected_repo: Option<RepoSummary> = {
        let s = sel.read().clone();
        if s.kind == NavKind::Repo {
            repos.read().iter().find(|r| r.id == s.id).cloned()
        } else {
            None
        }
    };

    // ── Repos visible in list (based on selection) ────────────────────────────
    let visible_repos: Vec<RepoSummary> = {
        let s = sel.read().clone();
        let all = repos.read().clone();
        match s.kind {
            NavKind::Smart => match s.id.as_str() {
                "all" => all,
                "uncommitted" => all.into_iter().filter(|r| !r.status.clean).collect(),
                "stale" => all.into_iter().filter(|r| is_stale(&r.last_commit_at)).collect(),
                _ => all,
            },
            NavKind::Folder => all.into_iter().filter(|r| r.folder_id == s.id).collect(),
            NavKind::Repo => all,
        }
    };

    let is_dashboard = {
        let s = sel.read();
        s.kind == NavKind::Smart && s.id == "dashboard"
    };

    let is_activity = {
        let s = sel.read();
        s.kind == NavKind::Smart && s.id == "activity"
    };

    // ── Toast auto-dismiss ────────────────────────────────────────────────────
    // Use a manual timer via js_sys; dismiss oldest after 3s
    // Simple approach: each toast rendered with a dismiss button + auto-expire via spawn
    let mut dismiss_toast = move |id: String| {
        toasts.write().retain(|t| t.id != id);
    };

    rsx! {
        // CSS asset
        document::Link { rel: "stylesheet", href: CSS }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.googleapis.com"
        }

        div { class: "win",
            // ── Titlebar ──────────────────────────────────────────────────────
            div { class: "titlebar",
                div { class: "traffic",
                    div { class: "dot red" }
                    div { class: "dot yellow" }
                    div { class: "dot green" }
                }
                div { class: "tb-brand",
                    span { class: "tb-logo", "</>" }
                    span { class: "tb-title", "CodeFinder" }
                }
                // Search trigger (⌘K)
                button {
                    class: "tb-search",
                    onclick: move |_| show_palette.set(true),
                    span { dangerous_inner_html: "{icon_html(\"search\", 13)}" }
                    span { "Search repos & commands" }
                    span { class: "tb-kbd", "⌘K" }
                }
                div { style: "flex:1" }
            }

            // ── Main layout ───────────────────────────────────────────────────
            div { class: "layout",
                // Sidebar
                Sidebar {
                    sel,
                    folders,
                    repos,
                    theme,
                    on_toggle_theme: move |_| {
                        let t = theme.read().clone();
                        theme.set(if t == "dark" { "light".into() } else { "dark".into() });
                    },
                    on_open_settings: move |_| show_settings.set(true),
                    toasts,
                }

                // Center: list or dashboard
                if is_dashboard {
                    Dashboard {
                        repos: repos.read().clone(),
                        folders: folders.read().clone(),
                        toasts,
                        on_open: move |r: RepoSummary| {
                            sel.set(Selection::repo(&r.id));
                        },
                    }
                } else if is_activity {
                    ActivityView { repos: repos.read().clone() }
                } else {
                    RepoList {
                        repos: visible_repos,
                        folders: folders.read().clone(),
                        sel,
                        on_open: move |r: RepoSummary| {
                            sel.set(Selection::repo(&r.id));
                        },
                    }
                }

                // Detail panel (when a repo is selected)
                if let Some(repo) = selected_repo {
                    crate::detail::DetailPanel {
                        repo,
                        editors: editors.read().clone(),
                        sel,
                        toasts,
                    }
                }
            }

            // ── Toasts ─────────────────────────────────────────────────────────
            div { class: "toast-wrap",
                for toast in toasts.read().iter() {
                    {
                        let tid = toast.id.clone();
                        let tmsg = toast.msg.clone();
                        let tmono = toast.mono.clone();
                        rsx! {
                            div { class: "toast",
                                span { class: "toast-msg", "{tmsg}" }
                                if !tmono.is_empty() {
                                    span { class: "toast-mono mono", "{tmono}" }
                                }
                                button {
                                    class: "toast-x",
                                    onclick: move |_| dismiss_toast(tid.clone()),
                                    "×"
                                }
                            }
                        }
                    }
                }
            }

            // ── Command palette ────────────────────────────────────────────────
            if *show_palette.read() {
                CommandPalette {
                    repos: repos.read().clone(),
                    folders: folders.read().clone(),
                    on_close: move |_| show_palette.set(false),
                    on_open_repo: move |r: RepoSummary| {
                        sel.set(Selection::repo(&r.id));
                        show_palette.set(false);
                    },
                    on_refresh: refresh,
                    toasts,
                }
            }

            // ── Settings modal ─────────────────────────────────────────────────
            if *show_settings.read() {
                SettingsModal {
                    editors: editors.read().clone(),
                    on_close: move |_| show_settings.set(false),
                    toasts,
                }
            }
        }
    }
}

// ── Activity view ─────────────────────────────────────────────────────────────

#[component]
fn ActivityView(repos: Vec<RepoSummary>) -> Element {
    let mut sorted = repos;
    sorted.sort_by(|a, b| b.last_commit_at.cmp(&a.last_commit_at));

    rsx! {
        div { class: "list",
            div { class: "list-bar",
                div { class: "list-count", "Activity Feed" }
            }
            div { class: "activity-feed",
                for r in sorted.iter() {
                    div { class: "activity-row",
                        div { class: "repo-glyph sm", style: "background:#7C6BFF",
                            "{glyph_for(&r.name)}"
                        }
                        div { class: "act-info",
                            span { class: "act-name", "{r.name}" }
                            span { class: "act-branch mono",
                                span { dangerous_inner_html: "{icon_html(\"branch\", 10)}" }
                                "{r.branch}"
                            }
                        }
                        span { class: "act-when", "{from_now(&r.last_commit_at)}" }
                    }
                }
                if sorted.is_empty() {
                    div { class: "empty-state",
                        span { dangerous_inner_html: "{icon_html(\"activity\", 36)}" }
                        p { "No repos yet" }
                    }
                }
            }
        }
    }
}
