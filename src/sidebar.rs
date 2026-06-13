use dioxus::prelude::*;

use crate::{
    icons::icon_html,
    models::*,
    shared::{Icon, Toast},
};

#[component]
pub fn Sidebar(
    sel: Signal<Selection>,
    folders: Signal<Vec<Folder>>,
    repos: Signal<Vec<RepoSummary>>,
    theme: Signal<String>,
    on_open_settings: EventHandler,
    on_toggle_theme: EventHandler,
    toasts: Signal<Vec<Toast>>,
) -> Element {
    let folders_val = folders.read().clone();
    let repos_val = repos.read().clone();

    let uncommitted: Vec<_> = repos_val.iter().filter(|r| !r.status.clean).collect();
    let stale: Vec<_> = repos_val.iter().filter(|r| is_stale(&r.last_commit_at)).collect();
    let favs: Vec<_> = repos_val.iter().filter(|r| r.favorite).collect();
    let current_sel = sel.read().clone();

    let is_active_smart = |id: &str| {
        current_sel.kind == NavKind::Smart && current_sel.id == id
    };
    let is_active_folder = |id: &str| {
        current_sel.kind == NavKind::Folder && current_sel.id == id
    };
    let is_active_repo = |id: &str| {
        current_sel.kind == NavKind::Repo && current_sel.id == id
    };

    rsx! {
        aside { class: "nav",
            div { class: "nav-scroll scroll",

                // Smart groups
                div { class: "nav-sec",
                    NavItem {
                        active: is_active_smart("dashboard"),
                        onclick: move |_| sel.set(Selection::smart("dashboard")),
                        icon: "dashboard", label: "Dashboard",
                    }
                    NavItem {
                        active: is_active_smart("all"),
                        onclick: move |_| sel.set(Selection::smart("all")),
                        icon: "archive", label: "All Repositories",
                        count: Some(repos_val.len().to_string()),
                    }
                    NavItem {
                        active: is_active_smart("activity"),
                        onclick: move |_| sel.set(Selection::smart("activity")),
                        icon: "activity", label: "Activity",
                    }
                }

                // Needs attention
                div { class: "nav-sec",
                    div { class: "nav-sec-h", "Needs Attention" }
                    NavItem {
                        active: is_active_smart("uncommitted"),
                        onclick: move |_| sel.set(Selection::smart("uncommitted")),
                        icon: "diff", label: "Uncommitted",
                        badge: Some(uncommitted.len().to_string()),
                        badge_danger: false,
                    }
                    NavItem {
                        active: is_active_smart("stale"),
                        onclick: move |_| sel.set(Selection::smart("stale")),
                        icon: "clock", label: "Stale",
                        badge: if stale.is_empty() { None } else { Some(stale.len().to_string()) },
                        badge_danger: true,
                    }
                }

                // Folders
                div { class: "nav-sec",
                    div { class: "nav-sec-h",
                        "Folders"
                        button {
                            class: "add",
                            title: "New folder",
                            onclick: move |_| {
                                // trigger new folder dialog via toast for now
                                let mut t = toasts.write();
                                t.push(Toast::new("New folder", "Use ⌘K → New folder"));
                                // auto-remove after 2.4s handled in App
                            },
                            span { dangerous_inner_html: "{icon_html(\"plus\", 14)}" }
                        }
                    }
                    for folder in folders_val.iter() {
                        {
                            let folder_id = folder.id.clone();
                            let folder_color = folder.color.clone();
                            let folder_name = folder.name.clone();
                            let count = repos_val.iter().filter(|r| r.folder_id == folder_id).count();
                            let dirty = repos_val.iter().filter(|r| r.folder_id == folder_id && !r.status.clean).count();
                            let active = is_active_folder(&folder_id);
                            rsx! {
                                div {
                                    class: if active { "nav-item active" } else { "nav-item" },
                                    onclick: move |_| sel.set(Selection::folder(&folder_id)),
                                    span { class: "fdot", style: "background:{folder_color}" }
                                    span { class: "label", "{folder_name}" }
                                    if dirty > 0 {
                                        span { class: "badge", "{dirty}" }
                                    }
                                    span { class: "count", "{count}" }
                                }
                            }
                        }
                    }
                }

                // Favorites
                if !favs.is_empty() {
                    div { class: "nav-sec",
                        div { class: "nav-sec-h", "Favorites" }
                        for r in favs.iter() {
                            {
                                let rid = r.id.clone();
                                let rname = r.name.clone();
                                let glyph = glyph_for(&rname);
                                let folder = folders_val.iter().find(|f| f.id == r.folder_id);
                                let color = folder.map(|f| f.color.clone()).unwrap_or_else(|| "#7C6BFF".into());
                                let dirty = !r.status.clean;
                                let active = is_active_repo(&rid);
                                rsx! {
                                    div {
                                        class: if active { "nav-item active" } else { "nav-item" },
                                        onclick: move |_| sel.set(Selection::repo(&rid)),
                                        span {
                                            class: "repo-glyph",
                                            style: "width:18px;height:18px;border-radius:5px;font-size:9px;background:{color}",
                                            "{glyph}"
                                        }
                                        span { class: "label mono", style: "font-size:12.5px", "{rname}" }
                                        if dirty {
                                            span { class: "dirty-dot", style: "position:static;margin-left:auto" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Footer
            div { class: "nav-foot",
                button {
                    class: "tb-btn",
                    title: "Toggle theme",
                    onclick: move |_| on_toggle_theme.call(()),
                    {
                        let t = theme.read().clone();
                        let icon = if t == "dark" { "sun" } else { "moon" };
                        rsx! { span { dangerous_inner_html: "{icon_html(icon, 16)}" } }
                    }
                }
                button {
                    class: "tb-btn",
                    title: "Manage editors",
                    onclick: move |_| on_open_settings.call(()),
                    span { dangerous_inner_html: "{icon_html(\"settings\", 16)}" }
                }
                div { style: "flex:1" }
                span { style: "font-size:11px;color:var(--text-4);font-weight:600", "v1.0" }
            }
        }
    }
}

// ── NavItem helper ────────────────────────────────────────────────────────────

#[component]
fn NavItem(
    active: bool,
    onclick: EventHandler<MouseEvent>,
    icon: &'static str,
    label: &'static str,
    count: Option<String>,
    badge: Option<String>,
    badge_danger: Option<bool>,
) -> Element {
    let badge_class = if badge_danger.unwrap_or(false) { "badge danger" } else { "badge" };
    rsx! {
        div {
            class: if active { "nav-item active" } else { "nav-item" },
            onclick: move |e| onclick.call(e),
            span { class: "ico", dangerous_inner_html: "{icon_html(icon, 16)}" }
            span { class: "label", "{label}" }
            if let Some(ref b) = badge {
                span { class: "{badge_class}", "{b}" }
            }
            if let Some(ref c) = count {
                if badge.is_none() {
                    span { class: "count", "{c}" }
                }
            }
        }
    }
}
